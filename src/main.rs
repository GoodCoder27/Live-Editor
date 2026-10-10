use axum::{
    Router,
    routing::get,
};

mod room;
mod socket;

#[tokio::main]
async fn main() {
    // Build router with a WebSocket route
    let app = Router::new().route("/ws", get(socket::ws_handler));

    // Run server on localhost:3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("WebSocket server running on ws://127.0.0.1:3000/ws");

    axum::serve(listener, app).await.unwrap();
}
