use crate::ripple_api::api_response::{ConversationChange, ConversationItem};
use crate::store_engine::store_engine::StorageConversationData;

/// Conversation operation types from backend
/// These represent semantic operations on conversations
pub mod conversation_operation {
    /// New message arrived in conversation
    /// Updates: last_message_id, last_message, last_message_timestamp
    pub const NEW_MESSAGE: i32 = 1;

    /// Messages marked as read
    /// Updates: last_read_message_id
    pub const READ_MESSAGE: i32 = 2;

    /// Conversation deleted/removed
    pub const DELETE: i32 = 3;

    /// New conversation created (optional, for future use)
    pub const CREATE: i32 = 4;
}

/// Actions to apply to conversation storage
#[derive(Debug)]
pub enum ConversationAction {
    /// Create or update entire conversation (used for CREATE and NEW_MESSAGE)
    Upsert(StorageConversationData),

    /// Update only the last read message ID (used for READ_MESSAGE)
    UpdateReadStatus {
        conversation_id: String,
        last_read_message_id: Option<i64>,
    },

    /// Delete conversation
    Delete { conversation_id: String },
}

/// Event actions to emit to frontend
pub mod conversation_event_action {
    /// New message received in conversation
    pub const NEW_MESSAGE: i32 = 0;

    /// Messages marked as read
    pub const READ_MESSAGE: i32 = 1;

    /// Conversation deleted
    pub const DELETE: i32 = 2;

    /// New conversation created
    pub const CREATE: i32 = 3;

    /// Clear all conversations (for full sync)
    pub const CLEAR: i32 = -1;
}
