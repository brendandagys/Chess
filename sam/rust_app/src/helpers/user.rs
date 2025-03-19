use std::collections::HashMap;

use crate::{
    types::dynamo_db::UserRecord,
    utils::dynamo_db::{get_item, put_item},
};

use aws_sdk_dynamodb::{types::AttributeValue, Client};
use lambda_runtime::Error;

pub async fn save_user(client: &Client, table: &str, user: &UserRecord) -> Result<(), Error> {
    put_item(client, table, user).await
}

async fn get_user_record(
    client: &Client,
    table: &str,
    username: &str,
    sort_key: &str,
) -> Result<UserRecord, Error> {
    let mut key = HashMap::new();
    key.insert("username".into(), AttributeValue::S(username.into()));
    key.insert("sk".into(), AttributeValue::S(sort_key.into()));
    Ok(get_item(client, table, key).await?)
}

pub async fn get_user(client: &Client, table: &str, username: &str) -> Result<UserRecord, Error> {
    get_user_record(client, table, username, "INFO").await
}

pub async fn get_user_game(
    client: &Client,
    table: &str,
    username: &str,
    game_id: &str,
) -> Result<UserRecord, Error> {
    get_user_record(client, table, username, &format!("GAME-{game_id}")).await
}

pub fn create_user_game(game_id: &str, username: &str) -> UserRecord {
    let sort_key = format!("GAME-{game_id}");

    UserRecord {
        username: username.to_string(),
        sort_key,
        winner: None,
        created: chrono::Utc::now().to_rfc3339(),
    }
}
