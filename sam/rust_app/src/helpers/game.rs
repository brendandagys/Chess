use crate::types::dynamo_db::GameRecord;
use crate::types::game::GameState;
use crate::types::pieces::Color;
use crate::utils::dynamo_db::{get_item, put_item};

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
    if game.white_username.is_none() {
        game.white_connection_id = Some(connection_id.to_string());
        game.white_username = Some(username.to_string());
    } else if game.black_username.is_none() {
        game.black_connection_id = Some(connection_id.to_string());
        game.black_username = Some(username.to_string());
    } else {
        return Err(Error::from("Game is full"));
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
