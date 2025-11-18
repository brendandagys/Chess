use crate::helpers::board::game_state_to_fen;
use crate::helpers::user::{get_user_game, save_user_record};
use crate::types::api::{ApiMessage, ApiResponse};
use crate::types::board::{Board, BoardSetup, File, Position, Rank};
use crate::types::dynamo_db::GameRecord;
use crate::types::game::{
    ColorPreference, EngineDifficulty, GameEnding, GameState, GameStateAtPointInTime, GameTime,
    PlayerMove, State,
};
use crate::types::piece::{Color, Piece};
use crate::utils::api_gateway::post_to_connection;
use crate::utils::dynamo_db::{get_item, put_item};

use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequestContext;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use chess_engine::engine::{Engine, SearchResult};
use chrono::{TimeZone, Utc};
use lambda_runtime::Error;
use std::collections::HashMap;

use super::generic::generate_id;

pub async fn save_game(client: &Client, table: &str, game: &GameRecord) -> Result<(), Error> {
    put_item(client, table, game).await
}

pub async fn get_game(
    client: &Client,
    table: &str,
    game_id: &str,
) -> Result<Option<GameRecord>, Error> {
    let mut key = HashMap::new();
    key.insert("game_id".into(), AttributeValue::S(game_id.into()));
    get_item(client, table, key).await
}

/// Returns a tuple containing:
/// 1) The connection ID for the white player, if applicable.
/// 2) The username for the white player, if applicable.
/// 3) The connection ID for the black player, if applicable.
/// 4) The username for the black player, if applicable.
///
/// This tuple will be applied to a `GameRecord` struct
pub fn determine_player_color(
    color_preference: ColorPreference,
    username: &str,
    connection_id: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    match color_preference {
        ColorPreference::Black => (
            None,
            None,
            Some(connection_id.to_string()),
            Some(username.to_string()),
        ),
        ColorPreference::White => (
            Some(connection_id.to_string()),
            Some(username.to_string()),
            None,
            None,
        ),
        ColorPreference::Random => {
            let random_value = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
                % 2;

            if random_value == 0 {
                (
                    Some(connection_id.to_string()),
                    Some(username.to_string()),
                    None,
                    None,
                )
            } else {
                (
                    None,
                    None,
                    Some(connection_id.to_string()),
                    Some(username.to_string()),
                )
            }
        }
    }
}

/// We know that the game has at least one player assigned.
pub fn assign_player_to_existing_or_remaining_slot(
    game: &mut GameRecord,
    username: &str,
    connection_id: &str,
) -> Result<(), Error> {
    if let Some(white_username) = &game.white_username {
        if let Some(black_username) = &game.black_username {
            if black_username != username && white_username != username {
                return Err(Error::from(format!(
                    "Game (ID: `{}`) already has two players",
                    game.game_id
                )));
            }
        }
    }

    for (color, connection_id_option) in [
        ("white", &game.white_connection_id),
        ("black", &game.black_connection_id),
    ] {
        if let Some(existing_connection_id) = connection_id_option {
            if connection_id == existing_connection_id {
                return Err(Error::from(format!(
                    "You are already connected to this game as {color}",
                )));
            }
        }
    }

    match &game.white_username {
        Some(white_username) if white_username == username => {
            // NOTE: Commented to handle bug where disconnect function doesn't run

            // if let Some(white_connection_id) = &game.white_connection_id {
            //     if white_connection_id != "<disconnected>" {
            //         return Err(Error::from(format!(
            //             "{username} has already joined this game (ID: {}) as white",
            //             game.game_id
            //         )));
            //     }
            // }

            game.white_connection_id = Some(connection_id.to_string());
        }
        Some(_) => {
            // NOTE: Commented to handle bug where disconnect function doesn't run

            // if let Some(black_username) = &game.black_username {
            //     if let Some(black_connection_id) = &game.black_connection_id {
            //         if black_connection_id != "<disconnected>" {
            //             return Err(Error::from(format!(
            //                 "{} has already joined this game (ID: {}) as black",
            //                 black_username, game.game_id
            //             )));
            //         }
            //     }
            // }

            game.black_username = Some(username.to_string());
            game.black_connection_id = Some(connection_id.to_string());
        }
        None => {
            let black_username = game
                .black_username
                .as_deref()
                .expect("Black username should be set if white is not");

            if black_username == username {
                game.black_connection_id = Some(connection_id.to_string());
            } else {
                game.white_username = Some(username.to_string());
                game.white_connection_id = Some(connection_id.to_string());
            }
        }
    }

    Ok(())
}

pub fn create_game(
    game_id: Option<&str>,
    username: &str,
    board_setup: Option<BoardSetup>,
    color_preference: ColorPreference,
    engine_difficulty: Option<EngineDifficulty>,
    seconds_per_player: Option<usize>,
    connection_id: &str,
) -> GameRecord {
    let game_id = game_id.map_or_else(generate_id, |id| id.to_string());

    let game_state = GameState::new(
        game_id.clone(),
        &board_setup.unwrap_or(BoardSetup::Standard),
        seconds_per_player,
    );

    let (white_connection_id, white_username, black_connection_id, black_username) =
        determine_player_color(color_preference, username, connection_id);

    GameRecord {
        game_id,
        white_connection_id,
        white_username,
        black_connection_id,
        black_username,
        engine_difficulty,
        game_state,
        created: chrono::Utc::now().to_rfc3339(),
    }
}

pub async fn mark_user_as_disconnected_and_notify_other_player(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    dynamo_db_client: &Client,
    game_table: &str,
    game: &mut GameRecord,
    username: &str,
) -> Result<(), Error> {
    match game.white_username == Some(username.to_string()) {
        true => {
            game.white_connection_id = Some("<disconnected>".to_string());
            save_game(dynamo_db_client, game_table, game).await?;

            if let Some(black_connection_id) = &game.black_connection_id {
                if black_connection_id != "<disconnected>"
                    && (post_to_connection(
                        sdk_config,
                        request_context,
                        black_connection_id,
                        &ApiResponse {
                            status_code: 200,
                            connection_id: Some(black_connection_id.clone()),
                            messages: vec![
                                format!("{username} has disconnected from the game").into()
                            ],
                            data: Some(&game),
                        },
                    )
                    .await?)
                        .is_some()
                {
                    tracing::info!(
                        "Notified black player of disconnection for game (ID: {})",
                        game.game_id
                    );
                }
            }
        }
        false => {
            game.black_connection_id = Some("<disconnected>".to_string());
            save_game(dynamo_db_client, game_table, game).await?;

            if let Some(white_connection_id) = &game.white_connection_id {
                if white_connection_id != "<disconnected>"
                    && (post_to_connection(
                        sdk_config,
                        request_context,
                        white_connection_id,
                        &ApiResponse {
                            status_code: 200,
                            connection_id: Some(white_connection_id.clone()),
                            messages: vec![
                                format!("{username} has disconnected from the game").into()
                            ],
                            data: Some(&game),
                        },
                    )
                    .await?)
                        .is_some()
                {
                    tracing::info!(
                        "Notified white player of disconnection for game (ID: {})",
                        game.game_id
                    );
                }
            }
        }
    }

    Ok(())
}

/// Notify a player, if they are connected
///
/// Originally used *only* to notify a human opponent (single WebSocket HTTP response would go to current player).
/// With the engine, we now need to notify the current player before the Lambda returns (i.e. between their move and the engine move).
pub async fn notify_player_about_game_update(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    current_user_connection_id: &str,
    game: &GameRecord,
    messages: Option<Vec<ApiMessage>>,
    current_player: bool, // Notify current player? Otherwise notify opponent.
) -> Result<(), Error> {
    if !current_player && game.engine_difficulty.is_some() {
        return Ok(());
    }

    let player_check = |s: &str| match current_player {
        true => s == current_user_connection_id,
        false => s != current_user_connection_id,
    };

    if let Some(white_connection_id) = &game.white_connection_id {
        if player_check(white_connection_id)
            && white_connection_id != "<disconnected>"
            && (post_to_connection(
                sdk_config,
                request_context,
                white_connection_id,
                &ApiResponse {
                    status_code: 200,
                    connection_id: Some(white_connection_id.clone()),
                    messages: messages.clone().unwrap_or_default(),
                    data: Some(game),
                },
            )
            .await?)
                .is_some()
        {
            tracing::info!("Sent game (ID: {}) update to white player", game.game_id);
        }
    }

    if let Some(black_connection_id) = &game.black_connection_id {
        if player_check(black_connection_id)
            && black_connection_id != "<disconnected>"
            && (post_to_connection(
                sdk_config,
                request_context,
                black_connection_id,
                &ApiResponse {
                    status_code: 200,
                    connection_id: Some(black_connection_id.clone()),
                    messages: messages.unwrap_or_default(),
                    data: Some(game),
                },
            )
            .await?)
                .is_some()
        {
            tracing::info!("Sent game (ID: {}) update to black player", game.game_id);
        }
    }

    Ok(())
}

pub fn is_game_over(game: &GameRecord) -> bool {
    matches!(game.game_state.current_state().state, State::Finished(_))
}

fn are_both_players_present(game: &GameRecord) -> bool {
    if game.engine_difficulty.is_some() {
        return true;
    }

    match (&game.white_connection_id, &game.black_connection_id) {
        (Some(white), Some(black)) => white != "<disconnected>" && black != "<disconnected>",
        _ => false,
    }
}

/// Confirm it is this player's turn
fn is_turn(game: &GameRecord, player_color: &Color) -> bool {
    *player_color == game.game_state.current_state().current_turn
}

pub fn can_player_make_a_move(game: &GameRecord, player_color: &Color) -> Result<(), &'static str> {
    if is_game_over(game) {
        return Err("Game is finished"); // TODO: add more detail
    }

    if !are_both_players_present(game) {
        return Err("Both players must be connected to make a move");
    }

    if !is_turn(game, player_color) {
        return Err("It is not your turn");
    }

    Ok(())
}

pub struct PlayerDetails {
    pub color: Color,
    pub username: String,
    pub opponent_username: Option<String>,
}

pub fn get_player_details_from_connection_id(
    game: &GameRecord,
    connection_id: &str,
) -> Option<PlayerDetails> {
    if let Some(white_connection_id) = &game.white_connection_id {
        if white_connection_id == connection_id {
            return Some(PlayerDetails {
                color: Color::White,
                username: game
                    .white_username
                    .clone()
                    .expect("White player must have a username"),
                opponent_username: game.black_username.clone(),
            });
        }
    }

    if let Some(black_connection_id) = &game.black_connection_id {
        if black_connection_id == connection_id {
            return Some(PlayerDetails {
                color: Color::Black,
                username: game
                    .black_username
                    .clone()
                    .expect("Black player must have a username"),
                opponent_username: game.white_username.clone(),
            });
        }
    }

    None
}

fn get_own_piece_from_position<'a>(
    board: &'a Board,
    position: &Position,
    player_color: &Color,
) -> Option<&'a Piece> {
    if let Some(piece) = board.get_piece_at_position(position) {
        if piece.color == *player_color {
            return Some(piece);
        }
    }

    None
}

fn does_move_create_self_check(
    board: &Board,
    player_move: &PlayerMove,
    player_color: &Color,
) -> bool {
    let mut hypothetical_board = board.clone();

    hypothetical_board.apply_move(player_move, false);
    hypothetical_board.is_king_in_check(player_color)
}

pub fn validate_move(
    board: &Board,
    player_move: &PlayerMove,
    player_color: &Color,
) -> Result<(), &'static str> {
    let Some(piece) = get_own_piece_from_position(board, &player_move.from, player_color) else {
        return Err("Move does not originate from one of your pieces");
    };

    if !board.is_valid_board_position(&player_move.to) {
        return Err("Move destination is out of bounds");
    }

    if !piece
        .possible_moves(board, &player_move.from, false)
        .contains(&player_move.to)
    {
        return Err("Invalid move");
    }

    if does_move_create_self_check(board, player_move, player_color) {
        return Err("Move would place your own king in check");
    }

    Ok(())
}

/// Update the game time and ensure the game is started if both players have just joined
pub fn check_if_both_players_just_joined(game_record: &mut GameRecord) {
    if game_record.engine_difficulty.is_some()
        || game_record
            .white_connection_id
            .as_deref()
            .unwrap_or("<disconnected>")
            != "<disconnected>"
            && game_record
                .black_connection_id
                .as_deref()
                .unwrap_or("<disconnected>")
                != "<disconnected>"
    {
        if let Some(game_time) = &mut game_record.game_state.game_time {
            game_time.both_players_last_connected_at = Some(chrono::Utc::now().to_rfc3339());
        }

        let current_state = game_record.game_state.current_state_mut();

        if current_state.state == State::NotStarted {
            current_state.state = State::InProgress;
        }
    }
}

pub fn update_game_time_after_engine_move(game_state: &mut GameState, search_duration: u64) {
    let engine_color = game_state.current_state().current_turn;

    if let Some(game_time) = &mut game_state.game_time {
        let search_seconds = ((search_duration + 999) / 1000) as usize; // Round up
        let time_to_decrement = search_seconds.max(1);

        match engine_color {
            Color::White => {
                game_time.white_seconds_left = game_time
                    .white_seconds_left
                    .saturating_sub(time_to_decrement);
            }
            Color::Black => {
                game_time.black_seconds_left = game_time
                    .black_seconds_left
                    .saturating_sub(time_to_decrement);
            }
        }
    }
}

/// Update the game time remaining for both players after a move is made
fn update_game_time(game_time: &mut GameTime, game_state: &mut GameStateAtPointInTime) {
    let current_turn = game_state.current_turn;

    let very_old_date = Utc.with_ymd_and_hms(1900, 1, 1, 0, 0, 0).unwrap();

    let last_time_both_players_connected = &game_time
        .both_players_last_connected_at
        .as_ref()
        .map_or(very_old_date, |s| {
            s.parse()
                .expect("Invalid date format in `GameTime.both_players_last_connected_at`")
        });

    let last_move_at = &game_time.last_move_at.as_ref().map_or(very_old_date, |s| {
        s.parse()
            .expect("Invalid date format in `GameTime.last_move_at`")
    });

    let last_action = if last_time_both_players_connected > last_move_at {
        last_time_both_players_connected
    } else {
        last_move_at
    };

    let elapsed = chrono::Utc::now() - *last_action;

    let seconds_left = match current_turn {
        Color::White => &mut game_time.white_seconds_left,
        Color::Black => &mut game_time.black_seconds_left,
    };

    *seconds_left = seconds_left.saturating_sub(elapsed.num_seconds() as usize);

    if *seconds_left == 0 {
        game_state.state = State::Finished(GameEnding::OutOfTime(current_turn));
        return;
    }

    game_time.last_move_at = Some(chrono::Utc::now().to_rfc3339());
}

/// Called after a move is made. Checks if the opponent's king is in check or checkmate.
fn check_for_mates(game_state: &mut GameStateAtPointInTime) {
    let board = &game_state.board;
    let opponent_color = game_state.current_turn.opponent_color();

    if board.is_king_in_check(&opponent_color) {
        game_state.in_check = Some(opponent_color);

        let opponent_pieces_and_positions = board.get_all_pieces(Some(&opponent_color));

        for (opponent_piece, from_position) in opponent_pieces_and_positions {
            for to_position in opponent_piece.possible_moves(board, &from_position, true) {
                let mut hypothetical_board = board.clone();
                hypothetical_board.apply_move(
                    &PlayerMove {
                        from: from_position.clone(),
                        to: to_position.clone(),
                    },
                    // Value should not matter as .possible_moves() won't return castling moves due to check
                    true,
                );

                if !hypothetical_board.is_king_in_check(&opponent_color) {
                    game_state.current_turn = opponent_color;
                    return;
                }
            }
        }

        game_state.state = State::Finished(GameEnding::Checkmate(opponent_color));
        return;
    }

    game_state.in_check = None;
    game_state.current_turn = opponent_color;
}

pub fn make_move(game_state: &mut GameState, player_move: &PlayerMove) -> Result<(), &'static str> {
    let mut next_state = game_state.current_state().clone();
    next_state.engine_result = None; // Clear previous engine result

    if let Some(game_time) = &mut game_state.game_time {
        update_game_time(game_time, &mut next_state);
    }

    match next_state.state {
        State::Finished(GameEnding::OutOfTime(_)) => {}
        _ => {
            if let Some(captured_piece) = next_state.board.apply_move(player_move, false) {
                match next_state.current_turn {
                    Color::White => {
                        next_state.captured_pieces.white.push(captured_piece);
                        next_state.captured_pieces.white_points += captured_piece.get_point_value();
                    }
                    Color::Black => {
                        next_state.captured_pieces.black.push(captured_piece);
                        next_state.captured_pieces.black_points += captured_piece.get_point_value();
                    }
                }
            }

            check_for_mates(&mut next_state); // Toggles turn
        }
    };

    game_state.history.push(next_state);

    Ok(())
}

pub fn get_next_move_from_engine_search_result(search_result: &SearchResult) -> PlayerMove {
    let from_square = search_result
        .best_move_from
        .expect("Engine did not return a move");
    let to_square = search_result
        .best_move_to
        .expect("Engine did not return a move");

    PlayerMove {
        from: Position {
            file: File((from_square.file() + 1) as usize),
            rank: Rank((from_square.rank() + 1) as usize),
        },
        to: Position {
            file: File((to_square.file() + 1) as usize),
            rank: Rank((to_square.rank() + 1) as usize),
        },
    }
}

pub fn get_engine_result(game_record: &GameRecord) -> SearchResult {
    let fen = game_state_to_fen(game_record.game_state.history.last().unwrap());

    let mut engine = Engine::new(
        None,
        None,
        None,
        None,
        Some(5000),
        None,
        None,
        None,
        // Some("../lpb-allbook.bin"),
        game_record.engine_difficulty.map(|d| d.into()),
    );

    engine.position = chess_engine::position::Position::from_fen(&fen)
        .unwrap_or_else(|_| panic!("Failed to load FEN: {fen}"));

    engine.think::<fn(u16, i32, &mut chess_engine::position::Position)>(None)
}

pub async fn get_engine_result_if_turn(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    game: &mut GameRecord,
    connection_id: &str,
) -> Result<Option<SearchResult>, Error> {
    let current_turn = game.game_state.current_state().current_turn;

    let engine_color = match (&game.white_username, &game.black_username) {
        (None, Some(_)) => Color::White,
        (Some(_), None) => Color::Black,
        _ => unreachable!("Engine games should have exactly one player"),
    };

    if engine_color == current_turn {
        notify_player_about_game_update(
            sdk_config,
            request_context,
            connection_id,
            game,
            None,
            true,
        )
        .await?;

        return Ok(Some(get_engine_result(game)));
    }

    Ok(None)
}

/// Update the user-game records for both players if the game has finished
pub async fn handle_if_game_is_finished(
    dynamo_db_client: &Client,
    user_table: &str,
    username: &str,
    opponent_username: Option<&str>,
    game_state: &GameState,
) -> Result<(), Error> {
    match game_state.current_state().state {
        State::Finished(GameEnding::Checkmate(losing_color))
        | State::Finished(GameEnding::OutOfTime(losing_color))
        | State::Finished(GameEnding::Resignation(losing_color)) => {
            let winner = Some(losing_color.opponent_color().to_string());

            let usernames = match opponent_username {
                Some(opponent) => vec![username, opponent],
                None => vec![username],
            };

            for username in usernames {
                let mut user_game =
                    get_user_game(dynamo_db_client, user_table, username, &game_state.game_id)
                        .await?
                        .unwrap_or_else(|| {
                            panic!(
                                "User game should exist for user {username} and game ID {}",
                                game_state.game_id
                            )
                        });

                user_game.winner = winner.clone();
                save_user_record(dynamo_db_client, user_table, &user_game).await?;
            }
        }
        _ => {}
    }

    Ok(())
}
