use crate::ripple_api::api_response::{
    GroupMemberData, RelationUser, UserGroupData, UserProfileData,
};
use crate::store_engine::store_engine::ConversationRecord;
use ripple_proto::ripple_pb::{push_message_request, send_message_req, PushMessageRequest};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ConversationType {
    #[serde(rename = "peer")]
    Peer,
    #[serde(rename = "group")]
    Group,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UIConversationItem {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "conversationType")]
    pub conversation_type: ConversationType,
    #[serde(rename = "peerId")]
    pub peer_id: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "lastMessageId")]
    pub last_message_id: Option<String>,
    #[serde(rename = "lastReadMessageId")]
    pub last_read_message_id: Option<String>,
    #[serde(rename = "unreadCount")]
    pub unread_count: i64,
    #[serde(rename = "lastMessage")]
    pub last_message_text: Option<String>,
    #[serde(rename = "lastMessageTimestamp")]
    pub last_message_timestamp: Option<i64>,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "avatar")]
    pub avatar: Option<String>,
    #[serde(rename = "botSessionId")]
    pub bot_session_id: Option<String>,
}

impl From<ConversationRecord> for UIConversationItem {
    fn from(item: ConversationRecord) -> Self {
        UIConversationItem {
            conversation_id: item.conversation_id,
            conversation_type: if item.peer_id.is_some() {
                ConversationType::Peer
            } else {
                ConversationType::Group
            },
            peer_id: item.peer_id,
            group_id: item.group_id,
            last_message_id: item.last_message_id,
            last_read_message_id: item.last_read_message_id,
            unread_count: item.unread_count,
            last_message_text: item.last_message_text,
            last_message_timestamp: item.last_message_timestamp,
            name: item.name,
            avatar: item.avatar,
            bot_session_id: item.bot_session_id,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UIConversations {
    pub conversations: Vec<UIConversationItem>,
}

impl From<Vec<ConversationRecord>> for UIConversations {
    fn from(items: Vec<ConversationRecord>) -> Self {
        UIConversations {
            conversations: items.into_iter().map(|item| item.into()).collect(),
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
    #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    pub content: String,
    pub timestamp: i64,
    #[serde(rename = "messageType")]
    pub message_type: i32,
    #[serde(rename = "commandType", skip_serializing_if = "Option::is_none")]
    pub command_type: Option<i32>,
    #[serde(rename = "commandData", skip_serializing_if = "Option::is_none")]
    pub command_data: Option<String>,
    #[serde(rename = "fileUrl", skip_serializing_if = "Option::is_none")]
    pub file_url: Option<String>,
    #[serde(rename = "fileName", skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

impl From<PushMessageRequest> for UIMessageItem {
    fn from(req: PushMessageRequest) -> Self {
        match req.payload {
            Some(push_message_request::Payload::MessagePayload(msg_payload)) => {
                let message_data = msg_payload
                    .message_data
                    .expect("MessagePayload must have message_data");
                match &message_data.message {
                    Some(send_message_req::Message::SingleMessageContent(msg_context)) => {
                        // Map file_url and file_name, treating empty strings as None
                        let file_url = if msg_context.file_url.is_empty() {
                            None
                        } else {
                            Some(msg_context.file_url.clone())
                        };
                        let file_name = if msg_context.file_name.is_empty() {
                            None
                        } else {
                            Some(msg_context.file_name.clone())
                        };
                        // Convert group_id: 0 means no group (1v1 chat)
                        let group_id = if message_data.group_id != 0 {
                            Some(message_data.group_id.to_string())
                        } else {
                            None
                        };
                        UIMessageItem {
                            message_id: message_data.message_id.to_string(),
                            conversation_id: message_data.conversation_id,
                            sender_id: message_data.sender_id.to_string(),
                            group_id,
                            content: msg_context.text.clone(),
                            timestamp: message_data.send_timestamp,
                            message_type: 1, // Text
                            command_type: None,
                            command_data: None,
                            file_url,
                            file_name,
                        }
                    }
                    Some(send_message_req::Message::GroupCommandMessageContent(cmd_content)) => {
                        // For group commands, group_id is always present
                        let group_id = if message_data.group_id != 0 {
                            Some(message_data.group_id.to_string())
                        } else {
                            None
                        };
                        UIMessageItem {
                            message_id: message_data.message_id.to_string(),
                            conversation_id: message_data.conversation_id,
                            sender_id: message_data.sender_id.to_string(),
                            group_id,
                            content: cmd_content.text.clone(),
                            timestamp: message_data.send_timestamp,
                            message_type: 2, // Command
                            command_type: Some(cmd_content.command_type),
                            command_data: Some(cmd_content.text.clone()),
                            file_url: None,
                            file_name: None,
                        }
                    }
                    _ => panic!("Unsupported message type in PushMessageRequest"),
                }
            }
            _ => panic!("Invalid PushMessageRequest payload"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UIUserGroups {
    pub groups: Vec<UserGroupData>,
}

pub trait EventEmitter: Send + Sync + Clone + 'static {
    fn emit_user_profile_updated(&self, profile: UserProfileData) -> anyhow::Result<()>;
    fn emit_relation_insert(&self, user: RelationUser) -> anyhow::Result<()>;
    fn emit_relation_update(&self, user: RelationUser) -> anyhow::Result<()>;
    fn emit_relation_delete(&self, user_id: String) -> anyhow::Result<()>;
    fn emit_relations_clear_all(&self) -> anyhow::Result<()>;

    fn emit_conversation_insert(&self, conversation: UIConversationItem) -> anyhow::Result<()>;
    fn emit_conversation_update(&self, conversation: UIConversationItem) -> anyhow::Result<()>;
    fn emit_conversation_delete(&self, conversation_id: String) -> anyhow::Result<()>;
    fn emit_conversation_delete_all(&self) -> anyhow::Result<()>;

    fn emit_conversations_received(
        &self,
        conversation_id: String,
        unread_count: i32,
        message: String,
        timestamp: String,
    ) -> anyhow::Result<()>;

    fn emit_message_updated(
        &self,
        action: i32,
        message: Option<UIMessageItem>,
    ) -> anyhow::Result<()>;
    fn emit_messages_cleared(&self) -> anyhow::Result<()>;

    fn emit_user_group_insert(&self, group: UserGroupData) -> anyhow::Result<()>;
    fn emit_user_group_update(&self, group: UserGroupData) -> anyhow::Result<()>;
    fn emit_user_group_delete(&self, group_id: String) -> anyhow::Result<()>;
    fn emit_user_groups_clear_all(&self) -> anyhow::Result<()>;
}
