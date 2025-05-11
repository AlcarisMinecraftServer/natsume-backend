use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::broadcast::Sender;
use std::sync::Arc;

pub fn ws_router(tx: Arc<Sender<String>>) -> Router {
    Router::new().route("/v1/ws", get(move |ws| handler(ws, tx.clone())))
}

async fn handler(ws: WebSocketUpgrade, tx: Arc<Sender<String>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| socket_handler(socket, tx))
}

async fn socket_handler(socket: WebSocket, tx: Arc<Sender<String>>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(t) => println!("[WebSocket] Received: {}", t),
            Message::Close(_) => break,
            _ => {}
        }
    }

    let _ = send_task.await;
}

pub fn make_message(_type: &str, category: &str, actor: &str, platform: &str) -> String {
    json!({
        "type": _type,
        "category": category,
        "actor": actor,
        "platform": platform
    })
    .to_string()
}
