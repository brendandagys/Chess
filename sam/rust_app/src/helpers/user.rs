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

pub async fn get_user(
    client: &Client,
    table: &str,
    pk: &str,
    created: &str,
) -> Result<UserRecord, Error> {
    let mut key = HashMap::new();
    key.insert("pk".into(), AttributeValue::S(pk.into()));
    key.insert("created".into(), AttributeValue::S(created.into()));
    Ok(get_item(client, table, key).await?)
}
