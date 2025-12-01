use crate::ripple_api::api_response::{ConversationChange, RelationUser, UserProfileData};
use crate::store_engine::store_engine::StorageConversationData;
use ripple_proto::ripple_pb::{push_message_request, send_message_req, PushMessageRequest};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UIConversationItem {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "peerId")]
    pub peer_id: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "lastMessageId")]
    pub last_message_id: Option<String>,
    #[serde(rename = "lastMessage")]
    pub last_message: Option<String>,
    #[serde(rename = "lastMessageTimestamp")]
    pub last_message_timestamp: Option<i64>,
    #[serde(rename = "lastReadMessageId")]
    pub last_read_message_id: Option<String>,
    #[serde(rename = "unreadCount")]
    pub unread_count: i32,
    #[serde(rename = "name")]
    pub name: Option<String>,
    #[serde(rename = "avatar")]
    pub avatar: Option<String>,
}

impl From<StorageConversationData> for UIConversationItem {
    fn from(item: StorageConversationData) -> Self {
        let last_msg_id = item.last_read_message_id.unwrap_or(0);

        UIConversationItem {
            conversation_id: item.conversation_id,
            peer_id: item.peer_id,
            group_id: item.group_id,
            last_message_id: Some(item.last_message_id.to_string()),
            last_message: Some(item.last_message),
            last_message_timestamp: Some(item.last_message_timestamp),
            last_read_message_id: item
                .last_read_message_id
                .as_ref()
                .and_then(|id| Some(id.to_string())),
            unread_count: item.unread_count,
            name: Some(item.name),
            avatar: item.avatar,
        }
    }
}

impl From<ConversationChange> for UIConversationItem {
    fn from(item: ConversationChange) -> Self {
        UIConversationItem {
            conversation_id: item.conversation_id,
            peer_id: item.peer_id,
            group_id: item.group_id,
            last_message_id: item.last_message_id,
            last_message: item.last_message,
            last_message_timestamp: item.last_message_timestamp,
            last_read_message_id: item.last_read_message_id,
            unread_count: 0,
            name: item.name,
            avatar: item.avatar,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UIMessageItem {
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "senderId")]
    pub sender_id: String,
    pub content: String,
    pub timestamp: i64,
}

impl From<PushMessageRequest> for UIMessageItem {
    fn from(req: PushMessageRequest) -> Self {
        match req.payload {
            Some(push_message_request::Payload::MessageData(message_data)) => {
                match &message_data.message {
                    Some(send_message_req::Message::SingleMessageContent(msg_context)) => {
                        UIMessageItem {
                            message_id: message_data.message_id.to_string(),
                            conversation_id: message_data.conversation_id,
                            sender_id: message_data.sender_id.to_string(),
                            content: msg_context.text.clone(),
                            timestamp: message_data.send_timestamp,
                        }
                    }
                    _ => panic!("Unsupported message type in PushMessageRequest"),
                }
            }
            _ => panic!("Invalid PushMessageRequest payload"),
        }
    }
}

pub trait EventEmitter: Send + Sync + Clone + 'static {
    fn emit_user_profile_updated(&self, profile: UserProfileData) -> anyhow::Result<()>;
    fn emit_relation_updated(&self, action: i32, user: Option<RelationUser>) -> anyhow::Result<()>;
    fn emit_relations_cleared(&self) -> anyhow::Result<()>;

    fn emit_conversation_updated(
        &self,
        action: i32,
        conversation: Option<UIConversationItem>,
    ) -> anyhow::Result<()>;
    fn emit_conversations_cleared(&self) -> anyhow::Result<()>;

    fn emit_message_updated(
        &self,
        action: i32,
        message: Option<UIMessageItem>,
    ) -> anyhow::Result<()>;
    fn emit_messages_cleared(&self) -> anyhow::Result<()>;
}
