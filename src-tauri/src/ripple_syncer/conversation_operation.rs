use crate::store_engine::store_engine::StorageConversationData;

pub mod conversation_operation {
    pub const CREATE_CONVERSATION: i32 = 1;
    pub const READ_MESSAGE: i32 = 2;
    pub const UPDATE_CONVERSATION_NAME: i32 = 3;
    pub const UPDATE_CONVERSATION_AVATAR: i32 = 4;
    pub const UPDATE_CONVERSATION_NAME_AVATAR: i32 = 5;
}

#[derive(Debug)]
pub enum ConversationStorageAction {
    Create(StorageConversationData),
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
    UpdateNameAvatar {
        conversation_id: String,
        name: String,
        avatar: String,
    },
}

pub mod conversation_ui_event_action {
    pub const CREATE: i32 = 1;
    pub const NEW_MESSAGE: i32 = 2;
    pub const READ_MESSAGE: i32 = 3;
    pub const UPDATE_NAME: i32 = 4;
    pub const UPDATE_AVATAR: i32 = 5;
    pub const UPDATE_NAME_AVATAR: i32 = 6;
    pub const DELETE: i32 = 7;
}
