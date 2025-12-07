use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    Join { room_id: String, player_id: String },
    Event { sequence_id: u64, data: Value },
    SyncRequest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    Welcome { room_id: String },
    Event { sequence_id: u64, data: Value },
    SyncRequest,
    Error { msg: String },
}
