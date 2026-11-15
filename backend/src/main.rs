use std::sync::{Arc};

use axum::{
    Router,
    routing::get,
};
use tokio::sync::Mutex;

use crate::room::RoomManager;

mod room;
mod socket;

type AppState = Arc<Mutex<RoomManager>>;

#[tokio::main]
async fn main() {
    // Build router with a WebSocket route
    let manager = Arc::new(Mutex::new(RoomManager::new()));
    let app = Router::new().route("/ws", get(socket::ws_handler)).with_state(manager);   
    // Run server on localhost:3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("WebSocket server running on ws://127.0.0.1:3000/ws");

    axum::serve(listener, app).await.unwrap();
}
