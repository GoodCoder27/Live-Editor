use std::collections::HashMap;

use tokio::sync::mpsc::{self, UnboundedSender};

pub type RoomId = String;
pub type UserId = String;

pub struct Room {
    id: RoomId,
    writer: UserId,
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

    pub fn change_source(&mut self, id: RoomId) {
        
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
            println!("Removed user, {} from room, {}", user_id, room_id);
        }
    }
}
