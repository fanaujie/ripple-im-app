use crate::ripple_syncer::event_emitter::UIMessageItem;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MessageUpdateEvent {
    pub action: i32,
    pub message: Option<UIMessageItem>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationReceivedMessageEvent {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "unreadCount")]
    pub unread_count: i32,
    pub message: String,
    pub timestamp: String,
}

pub enum UIEvent {
    UserProfileUpdated,
    RelationInserted,
    RelationUpdated,
    RelationDeleted,
    RelationClearedAll,
    ConversationInserted,
    ConversationUpdated,
    ConversationsDeleted,
    ConversationsClearedAll,
    ConversationReceivedNewMessage,
    MessageUpdated,
    UserGroupInserted,
    UserGroupUpdated,
    UserGroupDeleted,
    UserGroupsClearedAll,
}

impl Display for UIEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            UIEvent::UserProfileUpdated => "user-profile-updated".to_string(),
            UIEvent::RelationInserted => "relation-inserted".to_string(),
            UIEvent::RelationUpdated => "relation-updated".to_string(),
            UIEvent::RelationDeleted => "relation-deleted".to_string(),
            UIEvent::RelationClearedAll => "relations-cleared-all".to_string(),
            UIEvent::ConversationInserted => "conversation-inserted".to_string(),
            UIEvent::ConversationUpdated => "conversation-updated".to_string(),
            UIEvent::ConversationsDeleted => "conversations-deleted".to_string(),
            UIEvent::ConversationsClearedAll => "conversations-cleared-all".to_string(),
            UIEvent::ConversationReceivedNewMessage => {
                "conversation-received-new-message".to_string()
            }
            UIEvent::MessageUpdated => "message-updated".to_string(),
            UIEvent::UserGroupInserted => "user-group-inserted".to_string(),
            UIEvent::UserGroupUpdated => "user-group-updated".to_string(),
            UIEvent::UserGroupDeleted => "user-group-deleted".to_string(),
            UIEvent::UserGroupsClearedAll => "user-groups-cleared-all".to_string(),
        };
        write!(f, "{}", str)
    }
}
