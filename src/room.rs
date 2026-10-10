use std::collections::HashMap;

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
}

pub struct RoomManager {
    rooms: HashMap<RoomId, Room>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    pub fn create_room(&mut self, writer: UserId, source: String, users: HashMap<UserId, User>) {
        let id: RoomId = Self::create_id();

        let room = Room {
            id: id.clone(),
            writer,
            source,
            users,
        };
        self.rooms.insert(id, room);
    }
    fn create_id() -> RoomId {
        "ABC123".to_string()
    }
    
    pub fn add_user(&mut self, mut room: Room, user_id: UserId) {
        let user: User = User { id: user_id.clone() };
        room.users.insert(user_id, user);
    }
}
