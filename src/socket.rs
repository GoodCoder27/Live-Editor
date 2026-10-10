use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::room;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessage {
    action: String,
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerMessage {
    status: String,
    data: String,
}


pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    // Upgrade the connection to a WebSocket
    ws.on_upgrade(handle_socket)
}


pub async fn handle_socket(mut socket: WebSocket) {
    while let Some(result) = socket.recv().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("WebSocket error: {e}");
                break;
            }
        };

        let Message::Text(text) = msg else {
            continue;
        };

        let client_msg: ClientMessage =
            match serde_json::from_str(&text) {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("Invalid client message: {e}");
                    continue;
                }
            };

        let response = process_message(client_msg);

        

        if send_response(&mut socket, response).await.is_err() {
            eprintln!("Client disconnected");
            break;
        }
    }
}

fn process_message(client_msg: ClientMessage) -> ServerMessage {
    match client_msg.action.as_str() {
        "create_room" => ServerMessage {
            status: "success".to_string(),
            data: format!("User {} created a room", client_msg.data)
        },
        "code_change" => ServerMessage {
            status: "success".to_string(),
            data: format!("Code change received, {}", client_msg.data)
        },
        "cursor_change" => ServerMessage {
            status: "success".to_string(),
            data: "Cursor moved".to_string()
        },
        "user_joined" => ServerMessage {
            status: "success".to_string(),
            data: format!("New User {} joined the room", client_msg.data)
        },
        "user_left" => ServerMessage {
            status: "success".to_string(),
            data: format!("User {} left the room", client_msg.data)
        },
        "run" => ServerMessage {
            status: "success".to_string(),
            data: format!("running code: {}", client_msg.data)
        },
        "output" => ServerMessage {
            status: "success".to_string(),
            data: "outputing the result".to_string()
        },
        _ => ServerMessage { status: "error".to_string(), data: format!("Uknown action: {}", client_msg.action) }
    }
}

async fn send_response(socket: &mut WebSocket, response: ServerMessage) -> Result<(), axum::Error> {
    let json = serde_json::to_string(&response).expect("Failed to serialize server message");

    socket.send(Message::Text(json.into())).await
}
