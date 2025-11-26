use crate::ripple_api::api_response::{
    ConversationChange, ConversationItem, MessageItem, RelationUser, UserProfileData,
};
use crate::ripple_syncer::conversation_operation::ConversationAction;
use crate::ripple_syncer::relation_operation::RelationAction;
use ripple_proto::ripple_pb::{
    push_message_request, send_message_req, PushMessageRequest, SendMessageReq,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use uuid::Uuid;

pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Clone, Debug)]
pub struct StorageConversationData {
    pub conversation_id: String,
    pub peer_id: Option<String>,
    pub group_id: Option<String>,
    pub last_message_id: i64,
    pub last_message: String,
    pub last_message_timestamp: i64,
    pub last_read_message_id: Option<i64>,
    pub unread_count: i32,
}

impl From<ConversationItem> for StorageConversationData {
    fn from(item: ConversationItem) -> Self {
        StorageConversationData {
            conversation_id: item.conversation_id,
            peer_id: item.peer_id,
            group_id: item.group_id,
            last_message_id: item.last_message_id.parse().unwrap_or(0),
            last_message: item.last_message,
            last_message_timestamp: item.last_message_timestamp,
            last_read_message_id: item
                .last_read_message_id
                .as_ref()
                .and_then(|id| id.parse().ok()),
            unread_count: item.unread_count as i32,
        }
    }
}

impl From<&ConversationChange> for StorageConversationData {
    fn from(change: &ConversationChange) -> Self {
        StorageConversationData {
            conversation_id: change.conversation_id.clone(),
            peer_id: change.peer_id.clone(),
            group_id: change.group_id.clone(),
            last_message_id: change
                .last_message_id
                .as_ref()
                .and_then(|id| id.parse().ok())
                .unwrap_or(0),
            last_message: change.last_message.as_ref().cloned().unwrap_or_default(),
            last_message_timestamp: change.last_message_timestamp.unwrap_or(0),
            last_read_message_id: change
                .last_read_message_id
                .as_ref()
                .and_then(|id| id.parse().ok()),
            unread_count: 0,
        }
    }
}
#[derive(Clone, Debug)]
pub struct StorageMessageData {
    pub conversation_id: String,
    pub message_id: i64,
    pub sender_id: i64,
    pub receiver_id: i64,
    pub group_id: i64,
    pub send_timestamp: i64,
    pub text_content: Option<String>,
    pub file_url: Option<String>,
    pub file_name: Option<String>,
}

impl From<&MessageItem> for StorageMessageData {
    fn from(item: &MessageItem) -> Self {
        StorageMessageData {
            conversation_id: item.conversation_id.clone(),
            message_id: item.message_id.parse().unwrap_or(0),
            sender_id: item.sender_id.parse().unwrap_or(0),
            receiver_id: item
                .receiver_id
                .as_ref()
                .and_then(|id| id.parse().ok())
                .unwrap_or(0),
            group_id: item
                .group_id
                .as_ref()
                .and_then(|id| id.parse().ok())
                .unwrap_or(0),
            send_timestamp: item.send_timestamp,
            text_content: item.text_content.clone(),
            file_url: item.file_url.clone(),
            file_name: item.file_name.clone(),
        }
    }
}

impl From<&PushMessageRequest> for StorageMessageData {
    fn from(req: &PushMessageRequest) -> Self {
        match req.payload.as_ref() {
            Some(push_message_request::Payload::MessageData(message_data)) => {
                match &message_data.message {
                    Some(send_message_req::Message::SingleMessageContent(msg_context)) => {
                        StorageMessageData {
                            conversation_id: message_data.conversation_id.clone(),
                            message_id: message_data.message_id,
                            sender_id: message_data.sender_id,
                            receiver_id: message_data.receiver_id,
                            group_id: message_data.group_id,
                            send_timestamp: message_data.send_timestamp,
                            text_content: Some(msg_context.text.clone()),
                            file_url: Some(msg_context.file_url.clone()),
                            file_name: Some(msg_context.file_name.clone()),
                        }
                    }
                    _ => panic!("Unsupported message type in PushMessageRequest"),
                }
            }
            _ => panic!("Invalid PushMessageRequest payload"),
        }
    }
}

impl From<StorageMessageData> for MessageItem {
    fn from(data: StorageMessageData) -> Self {
        MessageItem {
            conversation_id: data.conversation_id,
            message_id: data.message_id.to_string(),
            sender_id: data.sender_id.to_string(),
            receiver_id: if data.receiver_id == 0 {
                None
            } else {
                Some(data.receiver_id.to_string())
            },
            group_id: if data.group_id == 0 {
                None
            } else {
                Some(data.group_id.to_string())
            },
            send_timestamp: data.send_timestamp,
            text_content: data.text_content,
            file_url: data.file_url,
            file_name: data.file_name,
        }
    }
}

#[trait_variant::make(RippleStorage: Send)]
pub trait StoreEngine: Sync + Clone + 'static {
    async fn exists_token(&self) -> anyhow::Result<bool>;
    async fn exists_profile(&self) -> anyhow::Result<bool>;
    async fn exist_relations(&self) -> anyhow::Result<bool>;
    async fn exist_conversations(&self) -> anyhow::Result<bool>;

    async fn get_device_id(&self) -> anyhow::Result<Option<Uuid>>;
    async fn save_device_id(&self, device_id: &Uuid) -> anyhow::Result<()>;
    async fn get_token(&self) -> anyhow::Result<Option<Token>>;
    async fn save_token(&self, access_token: &str, refresh_token: &str) -> anyhow::Result<()>;
    async fn get_user_profile(&self) -> anyhow::Result<Option<UserProfileData>>;
    async fn save_user_profile(&self, profile: &UserProfileData) -> anyhow::Result<()>;

    async fn apply_relation_all(
        &self,
        action: Vec<RelationUser>,
        last_version: &str,
    ) -> anyhow::Result<()>;
    async fn apply_relation_action(
        &self,
        action: RelationAction,
        version: String,
    ) -> anyhow::Result<()>;

    async fn get_relation(&self, user_id: &str) -> anyhow::Result<Option<RelationUser>>;
    async fn get_all_relations(&self) -> anyhow::Result<Vec<RelationUser>>;
    async fn get_relation_version(&self) -> anyhow::Result<Option<String>>;
    async fn clear_all_relations(&self) -> anyhow::Result<()>;

    async fn apply_conversation_all(
        &self,
        conversations: Vec<StorageConversationData>,
        last_version: &str,
    ) -> anyhow::Result<()>;
    async fn apply_conversation_action(
        &self,
        action: ConversationAction,
        version: String,
        user_id: i64,
    ) -> anyhow::Result<()>;
    async fn get_conversation(
        &self,
        conversation_id: &str,
    ) -> anyhow::Result<Option<StorageConversationData>>;
    async fn get_all_conversations(&self) -> anyhow::Result<Vec<StorageConversationData>>;
    async fn get_conversation_version(&self) -> anyhow::Result<Option<String>>;
    async fn clear_all_conversations(&self) -> anyhow::Result<()>;
    async fn store_message(&self, message: StorageMessageData) -> anyhow::Result<()>;
    async fn get_latest_messages(
        &self,
        conversation_id: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<StorageMessageData>>;
    async fn get_messages_before(
        &self,
        conversation_id: &str,
        before_message_id: i64,
        limit: u32,
    ) -> anyhow::Result<Vec<StorageMessageData>>;
}

#[derive(Clone)]
pub struct MemoryStore {
    inner: Arc<tokio::sync::Mutex<InnerStore>>,
}

struct InnerStore {
    access_token: Option<String>,
    refresh_token: Option<String>,
    uuid: Option<Uuid>,
    user_profile: Option<UserProfileData>,
    relations: HashMap<String, RelationUser>,
    relation_version: Option<String>,
    conversations: HashMap<String, StorageConversationData>,
    conversation_version: Option<String>,
    messages: HashMap<String, BTreeMap<i64, StorageMessageData>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore {
            inner: Arc::new(tokio::sync::Mutex::new(InnerStore {
                access_token: None,
                refresh_token: None,
                uuid: None,
                user_profile: None,
                relations: HashMap::new(),
                relation_version: None,
                conversations: HashMap::new(),
                conversation_version: None,
                messages: HashMap::new(),
            })),
        }
    }
}

impl RippleStorage for MemoryStore {
    async fn exists_token(&self) -> anyhow::Result<bool> {
        // always return false for in-memory store
        Ok(false)
    }

    async fn exists_profile(&self) -> anyhow::Result<bool> {
        let inner = self.inner.lock().await;
        Ok(inner.user_profile.is_some())
    }

    async fn exist_relations(&self) -> anyhow::Result<bool> {
        let inner = self.inner.lock().await;
        Ok(!inner.relations.is_empty())
    }

    async fn exist_conversations(&self) -> anyhow::Result<bool> {
        let inner = self.inner.lock().await;
        Ok(!inner.conversations.is_empty())
    }

    async fn get_device_id(&self) -> anyhow::Result<Option<Uuid>> {
        let inner = self.inner.lock().await;
        if let Some(uuid) = &inner.uuid {
            Ok(Some(*uuid))
        } else {
            Ok(None)
        }
    }

    async fn save_device_id(&self, device_id: &Uuid) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.uuid = Some(*device_id);
        Ok(())
    }

    async fn get_token(&self) -> anyhow::Result<Option<Token>> {
        let inner = self.inner.lock().await;
        if inner.access_token.is_none() || inner.refresh_token.is_none() {
            return Ok(None);
        }
        Ok(Some(Token {
            access_token: inner.access_token.clone().unwrap(),
            refresh_token: inner.refresh_token.clone().unwrap(),
        }))
    }

    async fn save_token(&self, access_token: &str, refresh_token: &str) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.access_token = Some(access_token.to_string());
        inner.refresh_token = Some(refresh_token.to_string());
        Ok(())
    }

    async fn get_user_profile(&self) -> anyhow::Result<Option<UserProfileData>> {
        let inner = self.inner.lock().await;
        Ok(inner.user_profile.clone())
    }

    async fn save_user_profile(&self, profile: &UserProfileData) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.user_profile = Some(profile.clone());
        Ok(())
    }

    async fn apply_relation_all(
        &self,
        action: Vec<RelationUser>,
        last_version: &str,
    ) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.relations.clear();
        for relation in action {
            inner.relations.insert(relation.user_id.clone(), relation);
        }
        inner.relation_version = Some(last_version.to_string());
        Ok(())
    }
    async fn apply_relation_action(
        &self,
        action: RelationAction,
        version: String,
    ) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;

        match action {
            RelationAction::Upsert(relation) => {
                inner.relations.insert(relation.user_id.clone(), relation);
            }

            RelationAction::UpdateRemarkName {
                user_id,
                remark_name,
            } => {
                if let Some(relation) = inner.relations.get_mut(&user_id) {
                    relation.remark_name = remark_name;
                }
            }

            RelationAction::UpdateNickName { user_id, nick_name } => {
                if let Some(relation) = inner.relations.get_mut(&user_id) {
                    relation.nick_name = nick_name;
                }
            }

            RelationAction::UpdateAvatar { user_id, avatar } => {
                if let Some(relation) = inner.relations.get_mut(&user_id) {
                    relation.avatar = avatar;
                }
            }

            RelationAction::UpdateFlags { user_id, flags } => {
                if let Some(relation) = inner.relations.get_mut(&user_id) {
                    relation.relation_flags = flags;
                }
                // If user doesn't exist, this operation is ignored
                // The caller should ensure complete user data is provided via Upsert
            }
            RelationAction::Delete { user_id } => {
                inner.relations.remove(&user_id);
            }
            RelationAction::UpdateNickNameAvatar {
                user_id,
                nick_name,
                avatar,
            } => {
                if let Some(relation) = inner.relations.get_mut(&user_id) {
                    relation.nick_name = nick_name;
                    relation.avatar = avatar;
                }
            }
        }
        inner.relation_version = Some(version);
        Ok(())
    }

    async fn get_relation(&self, user_id: &str) -> anyhow::Result<Option<RelationUser>> {
        let inner = self.inner.lock().await;
        Ok(inner.relations.get(user_id).cloned())
    }

    async fn get_all_relations(&self) -> anyhow::Result<Vec<RelationUser>> {
        let inner = self.inner.lock().await;
        Ok(inner.relations.values().cloned().collect())
    }

    async fn get_relation_version(&self) -> anyhow::Result<Option<String>> {
        let inner = self.inner.lock().await;
        Ok(inner.relation_version.clone())
    }

    async fn clear_all_relations(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.relations.clear();
        Ok(())
    }

    async fn apply_conversation_all(
        &self,
        conversations: Vec<StorageConversationData>,
        last_version: &str,
    ) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.conversations.clear();
        for conversation in conversations {
            inner
                .conversations
                .insert(conversation.conversation_id.clone(), conversation);
        }
        inner.conversation_version = Some(last_version.to_string());
        Ok(())
    }

    async fn apply_conversation_action(
        &self,
        action: ConversationAction,
        version: String,
        user_id: i64,
    ) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;

        match action {
            ConversationAction::Upsert(conversation) => {
                if let Some(conv) = inner.conversations.get_mut(&conversation.conversation_id) {
                    conv.unread_count += 1;
                }
                inner
                    .conversations
                    .insert(conversation.conversation_id.clone(), conversation);
            }
            ConversationAction::UpdateReadStatus {
                conversation_id,
                last_read_message_id,
            } => {
                let last_read_id = last_read_message_id.unwrap_or(0);
                let new_unread_count = if let Some(messages) = inner.messages.get(&conversation_id)
                {
                    messages
                        .iter()
                        .filter(|(msg_id, msg)| {
                            msg.receiver_id == user_id && **msg_id > last_read_id
                        })
                        .count() as i32
                } else {
                    0
                };
                if let Some(conversation) = inner.conversations.get_mut(&conversation_id) {
                    conversation.last_read_message_id = last_read_message_id;
                    conversation.unread_count = new_unread_count;
                } else {
                    eprintln!(
                        "[StoreEngine] Warning: Attempted to update read status for non-existent conversation: {}",
                        conversation_id
                    );
                }
            }
            ConversationAction::Delete { conversation_id } => {
                inner.conversations.remove(&conversation_id);
            }
        }
        inner.conversation_version = Some(version);
        Ok(())
    }

    async fn get_conversation(
        &self,
        conversation_id: &str,
    ) -> anyhow::Result<Option<StorageConversationData>> {
        let inner = self.inner.lock().await;
        Ok(inner.conversations.get(conversation_id).cloned())
    }

    async fn get_all_conversations(&self) -> anyhow::Result<Vec<StorageConversationData>> {
        let inner = self.inner.lock().await;
        Ok(inner.conversations.values().cloned().collect())
    }

    async fn get_conversation_version(&self) -> anyhow::Result<Option<String>> {
        let inner = self.inner.lock().await;
        Ok(inner.conversation_version.clone())
    }

    async fn clear_all_conversations(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.conversations.clear();
        Ok(())
    }

    async fn store_message(&self, message: StorageMessageData) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        let conversation_messages = inner
            .messages
            .entry(message.conversation_id.clone())
            .or_insert_with(BTreeMap::new);

        conversation_messages.insert(message.message_id, message.clone());
        Ok(())
    }

    async fn get_latest_messages(
        &self,
        conversation_id: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<StorageMessageData>> {
        let inner = self.inner.lock().await;

        let messages = match inner.messages.get(conversation_id) {
            Some(msgs) => msgs,
            None => {
                println!(
                    "[StoreEngine] No messages found for conv_id {}",
                    conversation_id
                );
                return Ok(Vec::new());
            }
        };

        // Use rev() to iterate from newest to oldest, then reverse back
        let mut result: Vec<StorageMessageData> = messages
            .iter()
            .rev()
            .take(limit as usize)
            .map(|(_, msg)| msg.clone())
            .collect();

        // Reverse to maintain old-to-new order
        result.reverse();

        println!("[StoreEngine] Returning {} latest messages", result.len());
        Ok(result)
    }

    async fn get_messages_before(
        &self,
        conversation_id: &str,
        before_message_id: i64,
        limit: u32,
    ) -> anyhow::Result<Vec<StorageMessageData>> {
        let inner = self.inner.lock().await;

        let messages = match inner.messages.get(conversation_id) {
            Some(msgs) => msgs,
            None => {
                return Ok(Vec::new());
            }
        };

        let mut result: Vec<StorageMessageData> = messages
            .range(..before_message_id)
            .rev()
            .take(limit as usize)
            .map(|(_, msg)| msg.clone())
            .collect();

        // Reverse to maintain old-to-new order
        result.reverse();
        Ok(result)
    }
}
