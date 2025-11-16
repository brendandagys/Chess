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

    let body = serde_json::to_string(&ApiResponse {
        status_code,
        connection_id,
        messages: messages.unwrap_or_default(),
        data,
    })?;

    let mut response = ApiGatewayProxyResponse::default();
    response.status_code = status_code.into();
    response.body = Some(Body::from(body));

    Ok(response)
}
