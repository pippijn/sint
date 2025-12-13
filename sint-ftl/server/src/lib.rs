use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::{IntoResponse, Json},
    routing::get,
};
use dashmap::DashMap;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

// --- Types ---

#[derive(Serialize)]
pub struct RoomList {
    pub rooms: Vec<String>,
}

#[derive(Clone)]
pub struct AppState {
    // Room ID -> Broadcast Channel
    pub rooms: Arc<DashMap<String, broadcast::Sender<String>>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    Join {
        room_id: String,
        player_id: String,
    },
    Event {
        sequence_id: u64,
        data: serde_json::Value,
    },
    SyncRequest {
        requestor_id: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    Welcome {
        room_id: String,
    },
    Event {
        sequence_id: u64,
        data: serde_json::Value,
    },
    SyncRequest {
        requestor_id: String,
    },
    Error {
        msg: String,
    },
}

// --- App Factory ---

pub fn create_app() -> Router {
    let state = AppState {
        rooms: Arc::new(DashMap::new()),
    };

    Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/rooms", get(list_rooms))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn list_rooms(State(state): State<AppState>) -> impl IntoResponse {
    let rooms: Vec<String> = state.rooms.iter().map(|r| r.key().clone()).collect();
    Json(RoomList { rooms })
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx_broadcast: Option<broadcast::Receiver<String>> = None;
    let mut my_room_id: Option<String> = None;

    loop {
        tokio::select! {
            // A. Receive from Client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                         // 1. Parse Message
                        let client_msg: Result<ClientMessage, _> = serde_json::from_str(&text);

                        match client_msg {
                            Ok(ClientMessage::Join { room_id, player_id: _ }) => {
                                // Create room if not exists
                                let tx = state.rooms.entry(room_id.clone()).or_insert_with(|| {
                                    let (tx, _) = broadcast::channel(100);
                                    tx
                                });

                                // Subscribe
                                rx_broadcast = Some(tx.subscribe());
                                my_room_id = Some(room_id.clone());

                                // Send Welcome
                                let welcome = serde_json::to_string(&ServerMessage::Welcome { room_id: room_id.clone() }).unwrap();
                                let _ = sender.send(Message::Text(welcome.into())).await;

                                tracing::info!("Player joined room {}", room_id);
                            }

                            Ok(ClientMessage::Event { sequence_id, data }) => {
                                // Relay to Room
                                if let Some(room_id) = &my_room_id
                                    && let Some(tx) = state.rooms.get(room_id) {
                                        // Re-wrap as Server Message
                                        let relay_msg = serde_json::to_string(&ServerMessage::Event { sequence_id, data }).unwrap();
                                        // We send raw string to broadcast
                                        let _ = tx.send(relay_msg);
                                    }
                            }

                            Ok(ClientMessage::SyncRequest { requestor_id }) => {
                                // Broadcast SyncRequest to peers
                                if let Some(room_id) = &my_room_id
                                    && let Some(tx) = state.rooms.get(room_id) {
                                        let relay_msg = serde_json::to_string(&ServerMessage::SyncRequest { requestor_id }).unwrap();
                                        let _ = tx.send(relay_msg);
                                    }
                            }

                            Err(e) => {
                                tracing::error!("Bad message: {:?}", e);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    _ => {}
                }
            }

            // B. Receive from Room (Broadcast)
            res = async {
                match rx_broadcast.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => futures::future::pending().await,
                }
            } => {
                match res {
                    Ok(msg) => {
                        // Forward to Client
                        let _ = sender.send(Message::Text(msg.into())).await;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!("Client lagged, skipped {} messages", skipped);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        // Room closed
                        break;
                    }
                }
            }
        }
    }
}
