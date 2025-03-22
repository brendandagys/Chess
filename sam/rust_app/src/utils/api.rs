use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use lambda_http::{http::StatusCode, Body, Error};
use serde::Serialize;

use crate::types::api::ApiResponse;

/// Sends a response back to the client
pub fn build_response<T: Serialize>(
    status_code: Option<StatusCode>,
    message: Option<&str>,
    data: T,
) -> Result<ApiGatewayProxyResponse, Error> {
    let status_code = status_code.unwrap_or(StatusCode::OK).as_u16();

    let body = Body::from(serde_json::to_string(&ApiResponse {
        status_code,
        message: message.map(|m| m.to_string()),
        data,
    })?);

    Ok(ApiGatewayProxyResponse {
        status_code: status_code.into(),
        body: Some(body),
        ..Default::default()
    })
}
