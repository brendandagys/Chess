use serde::{Deserialize, Serialize};

use super::game::PlayerAction;

#[derive(Deserialize)]
pub struct GameRequest {
    pub route: String, // API Gateway uses this to forward to the appropriate Lambda handler
    pub data: PlayerAction,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApiErrorType {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorMessage {
    pub message: String,
    pub error_type: ApiErrorType,
}

impl From<&str> for ApiErrorMessage {
    fn from(message: &str) -> Self {
        ApiErrorMessage {
            message: message.to_string(),
            error_type: ApiErrorType::Error,
        }
    }
}

impl From<String> for ApiErrorMessage {
    fn from(message: String) -> Self {
        ApiErrorMessage {
            message,
            error_type: ApiErrorType::Error,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub status_code: u16,
    pub messages: Vec<ApiErrorMessage>,
    pub data: Option<T>,
}
