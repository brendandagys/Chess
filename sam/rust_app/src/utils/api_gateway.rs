use aws_sdk_apigatewaymanagement::{operation::post_to_connection::PostToConnectionOutput, Client};
use serde::Serialize;

pub async fn post_to_connection<T>(
    client: &Client,
    connection_id: &str,
    data: &T,
) -> Result<PostToConnectionOutput, aws_sdk_apigatewaymanagement::Error>
where
    T: ?Sized + Serialize,
{
    let payload = serde_json::to_vec(&data).unwrap().into();

    let post_to_connection_output = client
        .post_to_connection()
        .connection_id(connection_id)
        .data(payload)
        .send()
        .await?;

    Ok(post_to_connection_output)
}
