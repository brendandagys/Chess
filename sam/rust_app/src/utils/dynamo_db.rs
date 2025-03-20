use aws_sdk_dynamodb::{types::AttributeValue, Client};
use lambda_runtime::Error;
use serde::{Deserialize, Serialize};
use serde_dynamo::aws_sdk_dynamodb_1::{from_item, from_items, to_item};
use std::collections::HashMap;

pub async fn get_item<'a, T: Deserialize<'a> + Serialize>(
    client: &Client,
    table_name: &str,
    key: HashMap<String, AttributeValue>,
) -> Result<T, Error> {
    let request = client
        .get_item()
        .table_name(table_name.to_string())
        .set_key(Some(key));

    let response = request.send().await?;

    let item = response
        .item
        .ok_or_else(|| Error::from("Record not found"))?;
    let typed_entity: T = from_item(item)?;

    Ok(typed_entity)
}

pub async fn query_items<'a, T: Deserialize<'a> + Serialize>(
    client: &Client,
    table_name: &str,
    key_condition_expression: Option<String>,
    expression_attribute_names: Option<HashMap<String, String>>,
    expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    index_name: Option<String>,
) -> Result<Vec<T>, Error> {
    let response = client
        .query()
        .table_name(table_name)
        .set_index_name(index_name)
        .set_key_condition_expression(key_condition_expression)
        .set_expression_attribute_names(expression_attribute_names)
        .set_expression_attribute_values(expression_attribute_values)
        .send()
        .await?;

    let items = response
        .items
        .ok_or_else(|| Error::from("Error obtaining DynamoDB items from query result"))?;

    let typed_entities: Vec<T> = from_items(items)?;

    Ok(typed_entities)
}

pub async fn put_item<'a, T: Deserialize<'a> + Serialize>(
    client: &Client,
    table_name: &str,
    typed_entity: &T,
) -> Result<(), Error> {
    let item = to_item(typed_entity)?;

    client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item))
        .send()
        .await?;

    Ok(())
}

pub async fn update_item<'a, T: Deserialize<'a> + Serialize>(
    client: &Client,
    table_name: &str,
    key: HashMap<String, AttributeValue>,
    update_expression: &str,
    expression_attribute_values: HashMap<String, AttributeValue>,
) -> Result<(), Error> {
    client
        .update_item()
        .table_name(table_name)
        .set_key(Some(key))
        .update_expression(update_expression)
        .set_expression_attribute_values(Some(expression_attribute_values))
        .send()
        .await?;

    Ok(())
}

pub async fn delete_item(
    client: &Client,
    table_name: &str,
    key: HashMap<String, AttributeValue>,
) -> Result<(), Error> {
    let request = client
        .delete_item()
        .table_name(table_name.to_string())
        .set_key(Some(key));

    request.send().await?;

    Ok(())
}
