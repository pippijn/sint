use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;

#[tokio::test]
async fn test_get_rooms_empty() {
    // 1. Setup Server
    let app = sint_server::create_app();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn server in background
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // 2. Make Request
    let url = format!("http://{}/api/rooms", addr);
    let resp = reqwest::get(&url).await.unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    let rooms = body.get("rooms").unwrap().as_array().unwrap();
    assert!(rooms.is_empty());
}

#[tokio::test]
async fn test_websocket_join() {
    // 1. Setup Server
    let app = sint_server::create_app();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn server in background
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // 2. Connect WS
    let url = format!("ws://{}/ws", addr);
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    // 3. Send Join Message
    let join_msg = serde_json::json!({
        "type": "Join",
        "payload": {
            "room_id": "test_room",
            "player_id": "P1"
        }
    });
    ws_stream
        .send(tokio_tungstenite::tungstenite::Message::Text(
            join_msg.to_string(),
        ))
        .await
        .unwrap();

    // 4. Expect Welcome Message
    if let Some(msg) = ws_stream.next().await {
        let msg = msg.unwrap();
        if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
            let server_msg: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(server_msg["type"], "Welcome");
            assert_eq!(server_msg["payload"]["room_id"], "test_room");
        } else {
            panic!("Expected text message");
        }
    } else {
        panic!("Connection closed unexpectedly");
    }
}
