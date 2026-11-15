use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, UnboundedSender};

pub type RoomId = String;
pub type UserId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    range_offset: u32,
    range_length: u32,
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CursorLoc {
    line: u32,
    column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Selection {
    #[serde(rename = "startLine")]
    start_line: u32,

    #[serde(rename = "startColumn")]
    start_column: u32,

    #[serde(rename = "endLine")]
    end_line: u32,

    #[serde(rename = "endColumn")]
    end_column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorChange {
    cursor: CursorLoc,
    selection: Option<Selection>,
}

pub struct Room {
    id: RoomId,
    writer: UserId,
    cursor: CursorChange,
    source: String,
    users: HashMap<UserId, User>,
}

pub struct User {
    id: UserId,
    tx: mpsc::UnboundedSender<String>,
}

pub struct RoomManager {
    pub rooms: HashMap<RoomId, Room>,
}

impl User {
    pub fn new(id: UserId, tx: UnboundedSender<String>) -> Self {
        User { id , tx }
    }
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    pub fn create_room(&mut self, id: RoomId, writer: UserId, source: String, tx: UnboundedSender<String>) {
        let writer_user: User = User::new(writer.clone(), tx);
        let mut users: HashMap<UserId, User> = HashMap::new();
        users.insert(writer.clone(), writer_user);

        let room = Room {
            id: id.clone(),
            writer,
            source,
            users,
            cursor: CursorChange { cursor: CursorLoc { line: 1, column: 1 }, selection: None },
        };
        self.rooms.insert(id, room);
    }

    pub fn broadcast(&self, room_id: &RoomId,  message: &String) {
        let Some(room) = self.rooms.get(room_id) else {
            return;
        };

        for user in room.users.values() {
            if let Err(error) = user.tx.send(message.to_string()) {
                eprintln!("Failed to send to user: {error}");   
            }
        }
    }

    pub fn change_source(&mut self, room_id: &RoomId, user_id: &UserId, change: CodeChange) {
        // CodeChange -> range_offset (place where text is place)
        //            -> range_length (range of text that is placed) -> Applies only to deletion
        //            -> the text itself
        if let Some(room) = self.rooms.get_mut(room_id) {
            if room.writer == user_id.to_string() {
                if change.range_length != 0 {
                    room.source.replace_range((change.range_offset as usize)..((change.range_length+change.range_offset) as usize), &change.text);
                } else {
                    room.source.insert_str(change.range_offset as usize, &change.text);
                }
            }
        }
    }

    pub fn get_source(&mut self, room_id: &RoomId, user_id: &UserId) -> String {
        if let Some(room) = self.rooms.get_mut(room_id) {
            if room.writer == user_id.to_string() {
                return room.source.clone();
            } else {
                return "".to_string();
            }
        } else {
            return "".to_string();
        }
    }

    pub fn update_position(&mut self, room_id: &RoomId, user_id: &UserId, cursor_data: CursorChange) {
        if let Some(room) = self.rooms.get_mut(room_id) {
            if room.writer == user_id.to_string() {
                if let Some(selection) = cursor_data.selection {
                    room.cursor.selection = Some(selection);
                } else {
                    room.cursor.selection = None;
                    room.cursor.cursor = cursor_data.cursor;
                }
            }
        }
    }
     
    pub fn add_user(&mut self, room_id: &RoomId, user_id: UserId, tx: UnboundedSender<String>) -> bool {
        let user = User::new(user_id.clone(), tx);
        if let Some(room) = self.rooms.get_mut(room_id) {
            room.users.insert(user_id.clone(), user);
            return true;
        }
        return false;
    }

    pub fn remove_user(&mut self, room_id: &RoomId, user_id: UserId) {
        if let Some(room) = self.rooms.get_mut(room_id) {
            if room.writer == user_id {
                if let Some(user_id) = room.users.keys().nth(1) {
                    room.writer = user_id.clone();
                }
            }
            room.users.remove(&user_id);
            if room.users.is_empty() {
                self.rooms.remove(room_id);
            }
            //println!("Removed user, {} from room, {}", user_id, room_id);
        }
    }
}
