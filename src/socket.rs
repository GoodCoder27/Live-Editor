use axum::{
    extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}}, response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncWriteExt, process::Command, sync::mpsc::{self, UnboundedSender}};
use futures_util::{SinkExt, StreamExt};

use uuid::Uuid;
use std::process::Stdio;

use crate::{AppState, room::{self, CodeChange, CursorChange, RoomId}};
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
                if client_msg.action == "run" {
                    let input: String = client_msg.data.clone();
                        // match serde_json::from_str(&client_msg.data) {
                        //     Ok(data) => data,
                        //     Err(e) => {
                        //         let err_msg = ServerMessage {
                        //             status: "error".to_string(),
                        //             data: format!("Invalid input data: {e}")
                        //         };
                        //         let _ = sender.send(Message::Text(serde_json::to_string(&err_msg).unwrap().into())).await;
                        //         continue;
                        //     }
                        // };

                    let code_opt = if let Some((room_id, user_id)) = &membership {
                        let mut mn = manager.lock().await;
                        Some(RoomManager::get_source(&mut mn, room_id, user_id))
                    } else {
                        None
                    };

                    let Some(code) = code_opt else {
                        continue;
                    };

                    let mut stdo = "".to_string();
                    let mut stde = "".to_string();
                    match run_cpp(code, input).await {
                        Ok((stdout, stderr)) => {
                            stdo = stdout;
                            stde = stderr;
                        }
                        Err(e) => {
                            println!("Execution failed: {e}");
                        }
                    }

                    if let Some((room_id, _)) = &membership {
                        let mut mn = manager.lock().await;
                        let message = serde_json::json!({
                            "action": "output",
                            "stdout": stdo,
                            "stderr": stde,
                        });
                        RoomManager::broadcast(&mut mn, room_id, &message.to_string());
                    }
                    let response = ServerMessage {
                        status: "success".to_string(),
                        data: "Code executed successfully".to_string(),
                    };

                    let json = serde_json::to_string(&response).expect("Failed to serialize response");
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        eprintln!("Client disconnected");
                        break;
                    }
               } else {
                    let response = {
                        let mut mn = manager.lock().await;
                        process_sync_message(client_msg, &mut mn, &tx, &mut membership)
                    };                
                    let json = serde_json::to_string(&response)
                        .expect("Failed to serialize response");

                    if sender.send(Message::Text(json.into())).await.is_err() {
                        eprintln!("Client disconnected");
                        break;
                    }    
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

fn process_sync_message(client_msg: ClientMessage, manager: &mut room::RoomManager,
    tx: &UnboundedSender<String>, membership: &mut Option<(RoomId, UserId)>) -> ServerMessage {
    match client_msg.action.as_str() {
        "join_room" => {
            let join_data: JoinRoomData = 
                match serde_json::from_str(&client_msg.data) {
                    Ok(data) => data,
                    Err(e) => {
                        return ServerMessage {
                        status: "error".to_string(),
                        data: format!("Invalid join-room data: {e}")
                        };
                    }
                };

            let user_id: UserId = join_data.user_id;
            let room_id: RoomId = join_data.room_id; 
            let mut writer: bool = false;
        
            if RoomManager::add_user(manager, &room_id, user_id.clone(), tx.clone()) == false {
                RoomManager::create_room(manager, room_id.clone(), user_id.clone(), String::new(), tx.clone());
                writer = true;
            }
            let message = serde_json::json!({
                "action": "join_room",
                "user_id": user_id,
                "writer": writer,
            });
            let json = serde_json::to_string(&message).expect("Failed to serialize server message");
            RoomManager::broadcast(manager, &room_id, &json);
            *membership = Some((room_id.clone(), user_id.clone()));
            
            return ServerMessage {
                status: "success".to_string(),
                data: format!("User {} has joined room {}", user_id, room_id),
            };
        },
        "code_change" => {
            let code_data: CodeChange = 
                match serde_json::from_str(&client_msg.data) {
                    Ok(data) => data,
                    Err(e) => {
                        return ServerMessage {
                            status: "error".to_string(),
                            data: format!("Invalid code data: {e}")
                        };
                    }
                };
            
            if let Some((room_id, user_id)) = membership {
                manager.change_source(room_id, user_id, code_data.clone());
                let message = serde_json::json!({
                    "action": "code_update",
                    "user_id": user_id,
                    "change": code_data,
                });
                RoomManager::broadcast(manager, room_id, &message.to_string());
            }
            return ServerMessage {
                status: "success".to_string(),
                data: format!("Code change received, {}", client_msg.data)
            };
        },
        "cursor_change" => {
            let cursor: CursorChange =
                match serde_json::from_str(&client_msg.data) {
                    Ok(data) => data,
                    Err(e) => {
                        return ServerMessage {
                            status: "error".to_string(),
                            data: format!("Invalid cursor data: {e}")
                        };
                    }
                };

            if let Some((room_id, user_id)) = membership {
                manager.update_position(room_id, user_id, cursor.clone());
                let message = serde_json::json!({
                    "action": "cursor_update",
                    "user_id": user_id,
                    "change": cursor,
                });
                RoomManager::broadcast(manager, room_id, &message.to_string());
            }
            
            return ServerMessage {
                status: "success".to_string(),
                data: "Cursor moved".to_string()
            };
        },
        "user_joined" => {
            return ServerMessage {
                status: "success".to_string(),
                data: format!("New User {} joined the room", client_msg.data)
            };
        },
        "user_left" => {
            return ServerMessage {
                status: "success".to_string(),
                data: format!("User {} left the room", client_msg.data)
            };
        },
        "input_update" => {
            let input: String = client_msg.data;

            if let Some((room_id, user_id)) = membership {
                let message = serde_json::json!({
                    "action": "input_update",
                    "user_id": user_id,
                    "input": input,
                });
                RoomManager::broadcast(manager, room_id, &message.to_string());
            }
            
            return ServerMessage {
                status: "success".to_string(),
                data: "you don't need that".to_string()
            };
        },
        _ => ServerMessage { status: "error".to_string(), data: format!("Uknown action: {}", client_msg.action) } 
    }
}

async fn run_cpp(
    code: String,
    input: String,
) -> Result<(String, String), Box<dyn std::error::Error>> {

    // Create a unique temporary directory.
    let workdir = format!("/tmp/ide-{}", Uuid::new_v4());
    fs::create_dir(&workdir).await?;

    // Save the source code.
    let source_path = format!("{}/main.cpp", workdir);
    fs::write(&source_path, code).await?;

    // Allow the container's UID 10001 to access the directory.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(
            &workdir,
            std::fs::Permissions::from_mode(0o777),
        ).await?;
    }

    // Start Docker.
    let mut child = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-i",
            "--memory=500m",
            "--memory-swap=500m",
            "--cpus=1",
            "--pids-limit=64",
            "--network=none",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--read-only",
            "--user=10001:10001",
        ])
        .arg(format!("--mount=type=bind,src={},dst=/work", workdir))
        .args([
            "--tmpfs",
            "/tmp:rw,exec,nosuid,nodev,size=64m,uid=10001,gid=10001,mode=0700",
            "ide",
            "bash",
            "-c",
            "cd /work && \
             timeout --signal=KILL 15s \
             g++ -std=c++20 -O2 -Wall -Wextra main.cpp -o main && \
             timeout --signal=KILL 2s ./main",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Pass the user's input to the C++ program.
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).await?;
    }

    // Wait for Docker to finish and collect its output.
    let output = child.wait_with_output().await?;

    // Convert bytes into Strings.
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    // Remove the temporary directory.
    fs::remove_dir_all(&workdir).await?;

    Ok((stdout, stderr))
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
