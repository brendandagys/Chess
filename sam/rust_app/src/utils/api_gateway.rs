use aws_config::SdkConfig;
use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequestContext;
use aws_sdk_apigatewaymanagement::config;
use aws_sdk_apigatewaymanagement::error::DisplayErrorContext;
use aws_sdk_apigatewaymanagement::{operation::post_to_connection::PostToConnectionOutput, Client};
use lambda_runtime::Error;
use serde::Serialize;

fn build_api_gateway_management_client(
    sdk_config: &SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
) -> Client {
    let domain_name = request_context.domain_name.as_ref().unwrap();
    let stage = request_context.stage.as_ref().unwrap();

    let api_management_config = config::Builder::from(sdk_config)
        .endpoint_url(format!("https://{domain_name}/{stage}"))
        .build();

    Client::from_conf(api_management_config)
}

pub async fn post_to_connection<T>(
    sdk_config: &SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    connection_id: &str,
    data: &T,
) -> Result<PostToConnectionOutput, Error>
where
    T: ?Sized + Serialize,
{
    let payload = serde_json::to_vec(&data).unwrap().into();
    let client = build_api_gateway_management_client(sdk_config, request_context);

    match client
        .post_to_connection()
        .connection_id(connection_id)
        .data(payload)
        .send()
        .await
    {
        Ok(output) => Ok(output),
        Err(err) => {
            tracing::error!("Failed to post to connection: {}", err);
            tracing::error!("{}", DisplayErrorContext(&err));
            return Err(Error::from(err));
        }
    }
}
