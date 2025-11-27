use crate::store_engine::store_engine::StorageConversationData;

pub mod conversation_operation {
    pub const NEW_MESSAGE: i32 = 1;
    pub const READ_MESSAGE: i32 = 2;
    pub const DELETE: i32 = 3;
    pub const CREATE: i32 = 4;
}

#[derive(Debug)]
pub enum ConversationAction {
    Create(StorageConversationData),
    NewMessage(StorageConversationData),
    UpdateReadStatus {
        conversation_id: String,
        last_read_message_id: Option<i64>,
    },

    Delete {
        conversation_id: String,
    },
}

pub mod conversation_event_action {
    pub const NEW_MESSAGE: i32 = 0;

    pub const READ_MESSAGE: i32 = 1;

    pub const DELETE: i32 = 2;

    pub const CREATE: i32 = 3;

    pub const CLEAR: i32 = -1;
}
