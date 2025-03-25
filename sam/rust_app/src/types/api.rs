use serde::{Deserialize, Serialize};

use super::game::PlayerAction;

#[derive(Deserialize)]
pub struct GameRequest {
    pub route: String,
    pub data: PlayerAction,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub status_code: u16,
    pub message: Option<String>,
    pub data: Option<T>,
}
