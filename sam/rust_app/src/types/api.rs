use serde::{Deserialize, Serialize};

use super::game::PlayerAction;

#[derive(Deserialize)]
pub struct GameRequest {
    pub route: String, // API Gateway uses this to forward to the appropriate Lambda handler
    pub data: PlayerAction,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApiMessageType {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiMessage {
    pub message: String,
    pub message_type: ApiMessageType,
}

impl From<&str> for ApiMessage {
    fn from(message: &str) -> Self {
        ApiMessage {
            message: message.to_string(),
            message_type: ApiMessageType::Error,
        }
    }
}

impl From<String> for ApiMessage {
    fn from(message: String) -> Self {
        ApiMessage {
            message,
            message_type: ApiMessageType::Error,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub status_code: u16,
    pub connection_id: Option<String>,
    pub messages: Vec<ApiMessage>,
    pub data: Option<T>,
}
