use crate::ripple_api::api_response::RelationUser;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub const FRIEND_FLAG: i32 = 0b0001;
pub const BLOCKED_FLAG: i32 = 0b0010;
pub const HIDDEN_FLAG: i32 = 0b0100;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationsUpdateEvent {
    pub action: i32,                                // Action type: 0=ADD_FRIEND, 1=REMOVE_FRIEND, etc.
    #[serde(rename = "userProfile")]
    pub user_profile: Option<RelationUser>,         // Relation user data (None for CLEAR action)
}

pub enum UIEvent {
    UserProfileUpdated,
    RelationAdded,
    RelationRemoved,
    RelationUpdated,
}

impl Display for UIEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            UIEvent::UserProfileUpdated => "user-profile-updated".to_string(),
            UIEvent::RelationAdded => "relation-added".to_string(),
            UIEvent::RelationRemoved => "relation-removed".to_string(),
            UIEvent::RelationUpdated => "relation-updated".to_string(),
        };
        write!(f, "{}", str)
    }
}
