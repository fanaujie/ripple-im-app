use crate::ripple_api::api_response::{MessageItem, RelationUser};
use crate::ripple_syncer::event_emitter::{UIConversationItem, UIMessageItem};
use ripple_proto::ripple_pb::PushMessageRequest;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub const FRIEND_FLAG: i32 = 0b0001;
pub const BLOCKED_FLAG: i32 = 0b0010;
pub const HIDDEN_FLAG: i32 = 0b0100;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationsUpdateEvent {
    pub action: i32,
    #[serde(rename = "userProfile")]
    pub user_profile: Option<RelationUser>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationUpdateEvent {
    pub action: i32,
    pub conversation: Option<UIConversationItem>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MessageUpdateEvent {
    pub action: i32,
    pub message: Option<UIMessageItem>,
}

pub enum UIEvent {
    UserProfileUpdated,
    RelationAdded,
    RelationRemoved,
    RelationUpdated,
    ConversationUpdated,
    MessageUpdated,
}

impl Display for UIEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            UIEvent::UserProfileUpdated => "user-profile-updated".to_string(),
            UIEvent::RelationAdded => "relation-added".to_string(),
            UIEvent::RelationRemoved => "relation-removed".to_string(),
            UIEvent::RelationUpdated => "relation-updated".to_string(),
            UIEvent::ConversationUpdated => "conversation-updated".to_string(),
            UIEvent::MessageUpdated => "message-updated".to_string(),
        };
        write!(f, "{}", str)
    }
}
