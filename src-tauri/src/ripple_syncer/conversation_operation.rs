use crate::store_engine::store_engine::StorageConversationData;

pub mod conversation_operation {
    pub const CREATE_CONVERSATION: i32 = 0;
    pub const NEW_MESSAGE: i32 = 1;
    pub const READ_MESSAGE: i32 = 2;
    pub const UPDATE_CONVERSATION_NAME: i32 = 3;
    pub const UPDATE_CONVERSATION_AVATAR: i32 = 4;
    pub const DELETE_CONVERSATION: i32 = 5;
}

#[derive(Debug)]
pub enum ConversationStorageAction {
    Create(StorageConversationData),
    NewMessage(StorageConversationData),
    UpdateReadStatus {
        conversation_id: String,
        last_read_message_id: Option<i64>,
    },
    UpdateName {
        conversation_id: String,
        name: String,
    },
    UpdateAvatar {
        conversation_id: String,
        avatar: String,
    },
    Delete {
        conversation_id: String,
    },
}

pub mod conversation_event_action {
    pub const CREATE: i32 = 0;
    pub const NEW_MESSAGE: i32 = 1;
    pub const READ_MESSAGE: i32 = 2;
    pub const UPDATE_NAME: i32 = 3;
    pub const UPDATE_AVATAR: i32 = 4;
    pub const DELETE: i32 = 5;
}
