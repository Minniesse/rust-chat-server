use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use comms::event::Event;  // MODIFIED: Added Event import

use super::room::{ChatRoom, ChatRoomMetadata, SessionAndUserId, UserSessionHandle};

pub type RoomJoinResult = (broadcast::Receiver<Event>, UserSessionHandle, Vec<String>);

#[derive(Debug, Clone)]
pub struct RoomManager {
    chat_rooms: HashMap<String, Arc<Mutex<ChatRoom>>>,
    chat_room_metadata: Vec<ChatRoomMetadata>,
}

impl RoomManager {
    pub(super) fn new(chat_rooms: Vec<(ChatRoomMetadata, Arc<Mutex<ChatRoom>>)>) -> RoomManager {
        let chat_room_metadata = chat_rooms
            .iter()
            .map(|(metadata, _)| metadata.clone())
            .collect();

        RoomManager {
            chat_room_metadata,
            chat_rooms: chat_rooms
                .into_iter()
                .map(|(metadata, chat_room)| (metadata.name.clone(), chat_room))
                .collect(),
        }
    }

    pub fn chat_room_metadata(&self) -> &Vec<ChatRoomMetadata> {
        &self.chat_room_metadata
    }

    /// Joins to a room given a user session
    pub async fn join_room(
        &self,
        room_name: &str,
        session_and_user_id: &SessionAndUserId,
    ) -> anyhow::Result<RoomJoinResult> {
        let room = self
            .chat_rooms
            .get(room_name)
            .ok_or_else(|| anyhow::anyhow!("room '{}' not found", room_name))?;

        let mut room = room.lock().await;
        let (broadcast_rx, user_session_handle) = room.join(session_and_user_id);

        Ok((
            broadcast_rx,
            user_session_handle,
            room.get_unique_user_ids().clone(),
        ))
    }

    pub async fn drop_user_session_handle(&self, handle: UserSessionHandle) -> anyhow::Result<()> {
        let room = self
            .chat_rooms
            .get(handle.room())
            .ok_or_else(|| anyhow::anyhow!("room '{}' not found", handle.room()))?;

        let mut room = room.lock().await;

        room.leave(handle);

        Ok(())
    }

    // MODIFIED: Added method to handle messages at room manager level
    pub async fn handle_message(&self, room_name: &str, user_id: String, content: String) -> anyhow::Result<()> {
        let room = self
            .chat_rooms
            .get(room_name)
            .ok_or_else(|| anyhow::anyhow!("room '{}' not found", room_name))?;

        let mut room = room.lock().await;
        room.handle_message(user_id, content)
    }

    // MODIFIED: Added method to get room history
    pub async fn get_room_history(&self, room_name: &str, session_id: &str) -> anyhow::Result<()> {
        let room = self
            .chat_rooms
            .get(room_name)
            .ok_or_else(|| anyhow::anyhow!("room '{}' not found", room_name))?;

        let room = room.lock().await;
        room.send_history_to_session(session_id)
    }
}
