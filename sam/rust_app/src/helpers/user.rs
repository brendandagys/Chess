use std::collections::HashMap;

use crate::{
    types::dynamo_db::UserRecord,
    utils::dynamo_db::{get_item, put_item, query_items},
};

use aws_sdk_dynamodb::{types::AttributeValue, Client};
use lambda_runtime::Error;

pub async fn save_user_record(
    client: &Client,
    table: &str,
    user_record: &UserRecord,
) -> Result<(), Error> {
    put_item(client, table, user_record).await
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

pub async fn get_user_info(
    client: &Client,
    table: &str,
    username: &str,
) -> Result<UserRecord, Error> {
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

pub async fn get_all_user_games(
    client: &Client,
    table: &str,
    username: &str,
) -> Result<Vec<UserRecord>, Error> {
    let key_condition_expression = "username
        = :username AND begins_with(sk, :game_prefix)"
        .to_string();

    let mut expression_attribute_values = HashMap::new();
    expression_attribute_values.insert(
        ":username".to_string(),
        AttributeValue::S(username.to_string()),
    );
    expression_attribute_values.insert(
        ":game_prefix".to_string(),
        AttributeValue::S("GAME-".to_string()),
    );

    let items = query_items(
        client,
        table,
        Some(key_condition_expression),
        None,
        Some(expression_attribute_values),
        None,
    )
    .await?;

    Ok(items)
}

pub async fn get_user_game_from_connection_id(
    client: &Client,
    table: &str,
    index: &str,
    connection_id: &str,
) -> Result<UserRecord, Error> {
    let key_condition_expression = "connection_id = :connection_id".to_string();

    let mut expression_attribute_values = HashMap::new();
    expression_attribute_values.insert(
        ":connection_id".to_string(),
        AttributeValue::S(connection_id.to_string()),
    );

    let items = query_items(
        client,
        table,
        Some(key_condition_expression),
        None,
        Some(expression_attribute_values),
        Some(index.to_string()),
    )
    .await?;

    items
        .into_iter()
        .next() // A connection should only have up to 1 game
        .ok_or_else(|| {
            Error::from(format!(
                "No user game found for the given connection ID: {connection_id}"
            ))
        })
}

pub fn create_user_game(game_id: &str, username: &str, connection_id: &str) -> UserRecord {
    let sort_key = format!("GAME-{game_id}");

    UserRecord {
        username: username.to_string(),
        sort_key,
        connection_id: Some(connection_id.to_string()),
        winner: None,
        created: chrono::Utc::now().to_rfc3339(),
    }
}
