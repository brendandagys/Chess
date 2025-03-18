use crate::types::dynamo_db::GameRecord;
use crate::types::game::GameState;
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
fn determine_player_slots(
    color: Option<&str>,
    connection_id: &str,
    username: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    match color {
        Some("black") => (
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

async fn create_and_save_game(
    client: &Client,
    table: &str,
    connection_id: &str,
    game_id: Option<&str>,
    username: &str,
    color: Option<&str>,
) -> Result<GameRecord, Error> {
    let timestamp = chrono::Utc::now().to_rfc3339();

    let (white_connection_id, white_username, black_connection_id, black_username) =
        determine_player_slots(color, connection_id, username);

    let game_id = game_id.map_or_else(generate_id, |id| id.to_string());
    let game_state = GameState::new(game_id.clone());

    let game_record = GameRecord {
        game_id,
        white_connection_id,
        white_username,
        black_connection_id,
        black_username,
        game_state,
        created: timestamp.clone(),
    };

    save_game(client, table, &game_record).await?;

    Ok(game_record)
}
