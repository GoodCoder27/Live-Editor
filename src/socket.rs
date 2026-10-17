use axum::{
    extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}}, response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, UnboundedSender};
use futures_util::{SinkExt, StreamExt};

use crate::{AppState, room::{self, RoomId}};
use crate::room::{UserId, RoomManager};

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessage {
    action: String,
    data: String,
}

#[derive(Deserialize, Debug)]
struct JoinRoomData {
    room_id: String,
    user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerMessage {
    status: String,
    data: String,
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(manager): State<AppState>) -> impl IntoResponse {
    let (tx, rx) = mpsc::unbounded_channel::<String>();
    // Upgrade the connection to a WebSocket
    ws.on_upgrade(move |socket| async move { handle_socket(socket, manager, rx, tx).await; })
}

pub async fn handle_socket(socket: WebSocket, manager: AppState,
    mut rx: mpsc::UnboundedReceiver<String>, tx: mpsc::UnboundedSender<String>) {
    let (mut sender, mut receiver) = socket.split();

    let mut membership: Option<(RoomId, UserId)> = None;
    
    loop {
        tokio::select! {
            incoming = receiver.next() => {
               let msg = match incoming {
                    Some(Ok(msg)) => msg,
                    Some(Err(e)) => {
                        eprintln!("WebSocket error: {e}");
                        break;
                    }
                    None => break,
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

                let (response, joined) = {
                    let mut mn = manager.lock().await;
                    process_message(client_msg, &mut mn, &tx)
                };
                if let Some((room_id, user_id)) = joined {
                    membership = Some((room_id, user_id));
                }
                
                let json = serde_json::to_string(&response)
                    .expect("Failed to serialize response");

                if sender.send(Message::Text(json.into())).await.is_err() {
                    eprintln!("Client disconnected");
                    break;
                }
            }
            // Receive messages queued by RoomManager::broadcast().
            outgoing = rx.recv() => {
                match outgoing {
                    Some(text) => {
                        if sender.send(Message::Text(text.into())).await.is_err() {
                            eprintln!("Client disconnected");
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }
    // Remove User connection
    if let Some((room_id, user_id)) = membership {
        let mut mn = manager.lock().await;
        mn.remove_user(&room_id, user_id);
    }
}

fn process_message(client_msg: ClientMessage, manager: &mut room::RoomManager, tx: &UnboundedSender<String>)
        -> (ServerMessage, Option<(RoomId, UserId)>)  {
    match client_msg.action.as_str() {
        "join_room" => {
            let join_data: JoinRoomData = 
                match serde_json::from_str(&client_msg.data) {
                    Ok(data) => data,
                    Err(e) => {
                        return (ServerMessage {
                        status: "error".to_string(),
                        data: format!("Invalid join-room data: {e}")
                        }, None);
                    }
                };

            let user_id: UserId = join_data.user_id;
            let room_id: RoomId = join_data.room_id; 
                        
            if RoomManager::add_user(manager, &room_id, user_id.clone(), tx.clone()) == false {
                RoomManager::create_room(manager, room_id.clone(), user_id.clone(), String::new(), tx.clone());
            }

            let response: ServerMessage = ServerMessage {
                status: "success".to_string(),
                data: format!("New user {} has joined room {}", user_id, room_id),
            };
            let json = serde_json::to_string(&response).expect("Failed to serialize server message");
            RoomManager::broadcast(manager, &room_id, &json);
            
            return (ServerMessage {
                status: "success".to_string(),
                data: format!("User {} has joined room {}", user_id, room_id),
            }, Some((room_id, user_id)));
        },
        "code_change" => {
           return (ServerMessage {
                status: "success".to_string(),
                data: format!("Code change received, {}", client_msg.data)
            }, None);
        },
        "cursor_change" => {
            return (ServerMessage {
                status: "success".to_string(),
                data: "Cursor moved".to_string()
            }, None);
        },
        "user_joined" => {
            return (ServerMessage {
                status: "success".to_string(),
                data: format!("New User {} joined the room", client_msg.data)
            }, None);
        },
        "user_left" => {
            return (ServerMessage {
                status: "success".to_string(),
                data: format!("User {} left the room", client_msg.data)
            }, None);
        }
        "run" => {
            return (ServerMessage {
                status: "success".to_string(),
                data: format!("running code: {}", client_msg.data)
            }, None);
        },
        "output" => {
            return (ServerMessage {
                status: "success".to_string(),
                data: "outputing the result".to_string()
            }, None);
        },
        _ => { return (ServerMessage { status: "error".to_string(), data: format!("Uknown action: {}", client_msg.action)}, None) }
    }
}
/*
this is a working example of a simple websocket
async fn send_response(socket: &mut WebSocket, response: ServerMessage) -> Result<(), axum::Error> {
    let json = serde_json::to_string(&response).expect("Failed to serialize server message");

    socket.send(Message::Text(json.into())).await
}

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

        let mut mn = manager.lock().await;
        let response = process_message(client_msg, &mut mn);

        if send_response(&mut socket, response).await.is_err() {
            eprintln!("Client disconnected");
            break;
        }
    }*/
