use crate::types::dynamo_db::GameRecord;
use crate::types::game::GameState;
use crate::types::pieces::Color;
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

pub async fn get_game(client: &Client, table: &str, game_id: &str) -> Result<GameRecord, Error> {
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
    mut game: GameRecord,
    username: &str,
    connection_id: &str,
) -> Result<GameRecord, Error> {
    // Check if the game is already full
    if let Some(white_username) = &game.white_username {
        if let Some(black_username) = &game.black_username {
            if black_username != username && white_username != username {
                return Err(Error::from("Game is full"));
            }
        }
    }

    match &game.white_username {
        Some(white_username) if white_username == username => {
            game.white_connection_id = Some(connection_id.to_string());
        }
        Some(_) => {
            game.black_connection_id = Some(connection_id.to_string());
            game.black_username = Some(username.to_string());
        }
        None => {
            game.white_connection_id = Some(connection_id.to_string());
            game.white_username = Some(username.to_string());
        }
    }

    Ok(game)
}

pub fn create_game(
    game_id: Option<&str>,
    username: &str,
    color_preference: Option<Color>,
    connection_id: &str,
) -> GameRecord {
    let game_id = game_id.map_or_else(generate_id, |id| id.to_string());
    let game_state = GameState::new(game_id.clone());

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

pub async fn mark_user_as_disconnected_and_update_other_player(
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
                post_to_connection(sdk_config, &request_context, &black_connection_id, &game)
                    .await?;
                tracing::info!(
                    "Notified black player of disconnection for game (ID: {})",
                    game.game_id
                );
            }
        }
        false => {
            game.black_connection_id = Some("<disconnected>".to_string());
            save_game(dynamo_db_client, &game_table, &game).await?;

            if let Some(white_connection_id) = &game.white_connection_id {
                post_to_connection(sdk_config, &request_context, &white_connection_id, &game)
                    .await?;
                tracing::info!(
                    "Notified white player of disconnection for game (ID: {})",
                    game.game_id
                );
            }
        }
    }

    Ok(())
}

pub async fn notify_players_about_game_update(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    current_user_connection_id: &str,
    game: &GameRecord,
) -> Result<(), Error> {
    // Always notify current user
    post_to_connection(
        sdk_config,
        request_context,
        current_user_connection_id,
        &game,
    )
    .await?;

    // Notify opponent if they are connected
    if let Some(white_connection_id) = &game.white_connection_id {
        if white_connection_id != current_user_connection_id {
            post_to_connection(sdk_config, request_context, white_connection_id, &game).await?;
            tracing::info!("Sent game (ID: {}) update to white player", game.game_id);
        }
    }

    if let Some(black_connection_id) = &game.black_connection_id {
        if black_connection_id != current_user_connection_id {
            post_to_connection(sdk_config, request_context, black_connection_id, &game).await?;
            tracing::info!("Sent game (ID: {}) update to black player", game.game_id);
        }
    }

    Ok(())
}

pub fn can_player_make_move(
    game: &GameRecord,
    username: &str,
    connection_id: &str,
) -> Result<(), &'static str> {
    if !is_player_of_game(game, username) {
        return Err("User is not part of this game");
    }
    if !is_turn(game, username) {
        return Err("Not user's turn");
    }
    Ok(())
}

fn is_player_of_game(game: &GameRecord, _username: &str) -> bool {
    // Check if user is in game
    true
}

fn is_turn(game: &GameRecord, _username: &str) -> bool {
    // Confirm it's this player's turn
    true
}

pub fn check_game_state(game: &GameRecord) -> Result<(), &'static str> {
    // Ensure game is still active
    Ok(())
}

pub fn make_move(
    game: &mut GameRecord,
    username: &str,
    player_move: &str,
) -> Result<(), &'static str> {
    if !is_valid_game_move(game, username, player_move) {
        return Err("Invalid move");
    }
    does_move_deliver_check(game);
    does_move_deliver_checkmate(game);
    // Save and broadcast updated state
    Ok(())
}

fn is_valid_game_move(game: &GameRecord, _username: &str, _player_move: &str) -> bool {
    if !is_own_piece_at_origin(game, _player_move) {
        return false;
    }
    if !is_move_in_bounds(game, _player_move) {
        return false;
    }
    if does_move_create_self_check(game, _player_move) {
        return false;
    }
    true
}

fn is_own_piece_at_origin(game: &GameRecord, _player_move: &str) -> bool {
    // Verify piece belongs to the player
    true
}

fn is_move_in_bounds(game: &GameRecord, _player_move: &str) -> bool {
    // Confirm move is valid on the board
    true
}

fn does_move_create_self_check(game: &GameRecord, _player_move: &str) -> bool {
    // Check if the move would cause player's own king to be in check
    false
}

fn does_move_deliver_check(game: &mut GameRecord) {
    // Check if this move places opponent in check
}

fn does_move_deliver_checkmate(game: &mut GameRecord) {
    // Check if this move checkmates opponent
}
