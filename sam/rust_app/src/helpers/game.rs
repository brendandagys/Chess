use crate::types::api::{ApiMessage, ApiResponse};
use crate::types::board::{Board, BoardSetup, Position};
use crate::types::dynamo_db::GameRecord;
use crate::types::game::{GameEnding, GameState, GameStateAtPointInTime, PlayerMove, State};
use crate::types::piece::{Color, Piece};
use crate::utils::api_gateway::post_to_connection;
use crate::utils::dynamo_db::{get_item, put_item};

use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequestContext;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
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
    Ok(get_item(client, table, key).await?)
}

/// RETURNS a tuple containing:
/// 1) The connection ID for the white player, if applicable.
/// 2) The username for the white player, if applicable.
/// 3) The connection ID for the black player, if applicable.
/// 4) The username for the black player, if applicable.
///
/// This tuple will be applied to a `GameRecord` struct
pub fn determine_player_color(
    color_preference: Option<Color>,
    username: &str,
    connection_id: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    match color_preference {
        Some(Color::Black) => (
            None,
            None,
            Some(connection_id.to_string()),
            Some(username.to_string()),
        ),
        _ => (
            Some(connection_id.to_string()),
            Some(username.to_string()),
            None,
            None,
        ),
    }
}

pub fn assign_player_to_remaining_slot(
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
            if let Some(white_connection_id) = &game.white_connection_id {
                if white_connection_id != "<disconnected>" {
                    return Err(Error::from(format!(
                        "{username} has already joined this game (ID: {}) as white",
                        game.game_id
                    )));
                }
            }

            game.white_connection_id = Some(connection_id.to_string());
        }
        Some(_) => {
            if let Some(black_username) = &game.black_username {
                if let Some(black_connection_id) = &game.black_connection_id {
                    if black_connection_id != "<disconnected>" {
                        return Err(Error::from(format!(
                            "{} has already joined this game (ID: {}) as black",
                            black_username, game.game_id
                        )));
                    }
                }
            }

            game.black_username = Some(username.to_string());
            game.black_connection_id = Some(connection_id.to_string());
        }
        None => {
            if let Some(black_username) = &game.black_username {
                if black_username == username {
                    return Err(Error::from(format!(
                        "{} has already joined this game (ID: {}) as black",
                        username, game.game_id
                    )));
                }
            }

            game.white_username = Some(username.to_string());
            game.white_connection_id = Some(connection_id.to_string());
        }
    }

    Ok(())
}

pub fn create_game(
    game_id: Option<&str>,
    username: &str,
    board_setup: Option<BoardSetup>,
    color_preference: Option<Color>,
    connection_id: &str,
) -> GameRecord {
    let game_id = game_id.map_or_else(generate_id, |id| id.to_string());

    let game_state = GameState::new(
        game_id.clone(),
        &board_setup.unwrap_or(BoardSetup::Standard),
    );

    let (white_connection_id, white_username, black_connection_id, black_username) =
        determine_player_color(color_preference, username, connection_id);

    GameRecord {
        game_id,
        white_connection_id,
        white_username,
        black_connection_id,
        black_username,
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
            save_game(dynamo_db_client, &game_table, &game).await?;

            if let Some(black_connection_id) = &game.black_connection_id {
                if black_connection_id != "<disconnected>" {
                    if let Some(_) = post_to_connection(
                        sdk_config,
                        &request_context,
                        &black_connection_id,
                        &ApiResponse {
                            status_code: 200,
                            connection_id: Some(black_connection_id.clone()),
                            messages: vec![
                                format!("{username} has disconnected from the game").into()
                            ],
                            data: Some(&game),
                        },
                    )
                    .await?
                    {
                        tracing::info!(
                            "Notified black player of disconnection for game (ID: {})",
                            game.game_id
                        );
                    }
                }
            }
        }
        false => {
            game.black_connection_id = Some("<disconnected>".to_string());
            save_game(dynamo_db_client, &game_table, &game).await?;

            if let Some(white_connection_id) = &game.white_connection_id {
                if white_connection_id != "<disconnected>" {
                    if let Some(_) = post_to_connection(
                        sdk_config,
                        &request_context,
                        &white_connection_id,
                        &ApiResponse {
                            status_code: 200,
                            connection_id: Some(white_connection_id.clone()),
                            messages: vec![
                                format!("{username} has disconnected from the game").into()
                            ],
                            data: Some(&game),
                        },
                    )
                    .await?
                    {
                        tracing::info!(
                            "Notified white player of disconnection for game (ID: {})",
                            game.game_id
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

/// Notify other player, if they are connected
pub async fn notify_other_player_about_game_update(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    current_user_connection_id: &str,
    game: &GameRecord,
    messages: Option<Vec<ApiMessage>>,
) -> Result<(), Error> {
    if let Some(white_connection_id) = &game.white_connection_id {
        if white_connection_id != current_user_connection_id
            && white_connection_id != "<disconnected>"
        {
            if let Some(_) = post_to_connection(
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
            .await?
            {
                tracing::info!("Sent game (ID: {}) update to white player", game.game_id);
            }
        }
    }

    if let Some(black_connection_id) = &game.black_connection_id {
        if black_connection_id != current_user_connection_id
            && black_connection_id != "<disconnected>"
        {
            if let Some(_) = post_to_connection(
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
            .await?
            {
                tracing::info!("Sent game (ID: {}) update to black player", game.game_id);
            }
        }
    }

    Ok(())
}

pub fn is_game_over(game: &GameRecord) -> bool {
    match game.game_state.current_state().state {
        State::Finished(_) => true,
        _ => false,
    }
}

fn are_both_players_present(game: &GameRecord) -> bool {
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

    hypothetical_board.apply_move(player_move);
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
        .possible_moves(board, &player_move.from)
        .contains(&player_move.to)
    {
        return Err("Invalid move");
    }

    if does_move_create_self_check(board, player_move, player_color) {
        return Err("Move would place your own king in check");
    }

    Ok(())
}

/// Called after a move is made. Checks if the opponent's king is in check or checkmate.
fn check_for_mates(game_state: &mut GameStateAtPointInTime) {
    let board = &game_state.board;
    let opponent_color = game_state.current_turn.opponent_color();

    if board.is_king_in_check(&opponent_color) {
        game_state.in_check = Some(opponent_color);

        let possible_moves_by_opponent_to_remove_check =
            board.get_all_pieces(Some(&opponent_color));

        for (opponent_piece, from_position) in possible_moves_by_opponent_to_remove_check {
            for to_position in opponent_piece.possible_moves(board, &from_position) {
                let mut hypothetical_board = board.clone();

                hypothetical_board.apply_move(&PlayerMove {
                    from: from_position.clone(),
                    to: to_position.clone(),
                });

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
    let current_state = game_state.current_state();
    let mut next_state = (*current_state).clone();

    if let Some(captured_piece) = next_state.board.apply_move(player_move) {
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

    check_for_mates(&mut next_state);

    if current_state.state == State::NotStarted {
        next_state.state = State::InProgress;
    }

    game_state.history.push(next_state);

    Ok(())
}
