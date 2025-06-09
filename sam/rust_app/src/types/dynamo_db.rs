use crate::types::game::GameState;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GameRecord {
    pub game_id: String, // PK
    pub white_connection_id: Option<String>,
    pub white_username: Option<String>,
    pub black_connection_id: Option<String>,
    pub black_username: Option<String>,
    pub game_state: GameState,
    pub created: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserRecord {
    pub username: String, // PK
    #[serde(rename = "sk")]
    pub sort_key: String, // SK: INFO | GAME-<game-id>
    pub connection_id: Option<String>,
    pub winner: Option<String>,
    pub created: String,
}
