use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use lambda_http::{http::StatusCode, Body, Error};
use serde::Serialize;

use crate::types::api::{ApiMessage, ApiResponse};

/// Sends a response back to the client
pub fn build_response<T: Serialize>(
    status_code: StatusCode,
    connection_id: Option<String>,
    messages: Option<Vec<ApiMessage>>,
    data: Option<T>,
) -> Result<ApiGatewayProxyResponse, Error> {
    let status_code = status_code.as_u16();

    let body = Body::from(serde_json::to_string(&ApiResponse {
        status_code,
        connection_id,
        messages: messages.unwrap_or_default(),
        data,
    })?);

    Ok(ApiGatewayProxyResponse {
        status_code: status_code.into(),
        body: Some(body),
        ..Default::default()
    })
}
