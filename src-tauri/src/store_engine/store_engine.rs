use crate::ripple_api::api_response::{
    ConversationChange, ConversationItem, GroupMemberData, MessageItem, RelationUser,
    UserGroupData, UserProfileData,
};

use std::collections::{BTreeMap, HashMap};
use std::option::Option;
use std::sync::Arc;
use uuid::Uuid;

pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Clone, Debug)]
pub struct ConversationRecord {
    pub conversation_id: String,
    pub peer_id: Option<String>,
    pub group_id: Option<String>,
    pub last_message_id: Option<String>,
    pub last_read_message_id: Option<String>,
    pub unread_count: i64,
    pub last_message_text: Option<String>,
    pub last_message_timestamp: Option<i64>,
    pub name: String,
    pub avatar: Option<String>,
    pub bot_session_id: Option<String>,
}

impl From<ConversationItem> for ConversationRecord {
    fn from(item: ConversationItem) -> Self {
        ConversationRecord {
            conversation_id: item.conversation_id,
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

impl From<ConversationChange> for ConversationRecord {
    fn from(item: ConversationChange) -> Self {
        ConversationRecord {
            conversation_id: item.conversation_id,
            peer_id: item.peer_id,
            group_id: item.group_id,
            last_message_id: None, // ConversationChange doesn't include last_message_id
            last_read_message_id: item.last_read_message_id,
            unread_count: 0, // ConversationChange doesn't include unread_count
            last_message_text: None, // ConversationChange doesn't include last_message_text
            last_message_timestamp: None, // ConversationChange doesn't include last_message_timestamp
            name: item.name.unwrap(),
            avatar: item.avatar,
            bot_session_id: item.bot_session_id,
        }
    }
}

#[derive(Debug)]
pub enum RelationStorageAction {
    Upsert(RelationUser),
    UpdateRemarkName {
        user_id: String,
        remark_name: String,
    },
    UpdateNickName {
        user_id: String,
        nick_name: String,
    },
    UpdateAvatar {
        user_id: String,
        avatar: Option<String>,
    },
    UpdateFlags {
        user_id: String,
        flags: i32,
    },
    Delete {
        user_id: String,
    },
    UpdateNickNameAvatar {
        user_id: String,
        nick_name: String,
        avatar: Option<String>,
    },
}
#[derive(Debug)]
pub enum ConversationStorageAction {
    Create(ConversationRecord),
    UpdateLastReadMessageId {
        conversation_id: String,
        last_read_message_id: String,
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
    Delete {
        conversation_id: String,
    },
    UpdateBotSessionId {
        conversation_id: String,
        bot_session_id: String,
    },
}

#[derive(Debug)]
pub enum UserGroupStorageAction {
    Upsert(UserGroupData),
    UpdateName {
        group_id: String,
        name: String,
    },
    UpdateAvatar {
        group_id: String,
        avatar: Option<String>,
    },
    Delete {
        group_id: String,
    },
}

#[derive(Debug)]
pub enum GroupMemberStorageAction {
    Upsert(GroupMemberData),
    UpdateName {
        user_id: String,
        name: String,
    },
    UpdateAvatar {
        user_id: String,
        avatar: Option<String>,
    },
    Delete {
        user_id: String,
    },
}

#[trait_variant::make(RippleStorage: Send)]
pub trait StoreEngine: Sync + Clone + 'static {
    async fn exists_token(&self) -> anyhow::Result<bool>;
    async fn exist_relations(&self) -> anyhow::Result<bool>;
    async fn exist_conversations(&self) -> anyhow::Result<bool>;

    async fn get_device_id(&self) -> anyhow::Result<Option<Uuid>>;
    async fn save_device_id(&self, device_id: &Uuid) -> anyhow::Result<()>;
    async fn get_token(&self) -> anyhow::Result<Option<Token>>;
    async fn save_token(&self, access_token: &str, refresh_token: &str) -> anyhow::Result<()>;
    async fn clear_token(&self) -> anyhow::Result<()>;

    async fn get_stored_user_id(&self) -> anyhow::Result<Option<String>>;
    async fn save_user_id(&self, user_id: &str) -> anyhow::Result<()>;
    async fn clear_all_data(&self) -> anyhow::Result<()>;

    async fn get_user_profile(&self) -> anyhow::Result<Option<UserProfileData>>;
    async fn save_user_profile(&self, profile: UserProfileData) -> anyhow::Result<()>;

    async fn apply_relation_all(
        &self,
        action: Vec<RelationUser>,
        last_version: &str,
    ) -> anyhow::Result<()>;
    async fn apply_relation_action(
        &self,
        action: RelationStorageAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<RelationUser>>;

    async fn get_relation(&self, user_id: &str) -> anyhow::Result<Option<RelationUser>>;
    async fn get_all_relations(&self) -> anyhow::Result<Vec<RelationUser>>;
    async fn get_relation_version(&self) -> anyhow::Result<Option<String>>;
    async fn clear_all_relations(&self) -> anyhow::Result<()>;

    async fn apply_conversation_all(
        &self,
        conversations: Vec<ConversationRecord>,
        last_version: &str,
    ) -> anyhow::Result<()>;
    async fn apply_conversation_action(
        &self,
        action: ConversationStorageAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<ConversationRecord>>;
    async fn conversation_exists(&self, conversation_id: &str) -> anyhow::Result<bool>;
    async fn get_conversation_by_id(
        &self,
        conversation_id: &str,
    ) -> anyhow::Result<Option<ConversationRecord>>;
    async fn get_all_conversations(&self) -> anyhow::Result<Vec<ConversationRecord>>;
    async fn get_conversation_version(&self) -> anyhow::Result<Option<String>>;
    async fn clear_all_conversations(&self) -> anyhow::Result<()>;
    async fn update_conversation_summary(
        &self,
        conversation_id: &str,
        unread_count: i64,
        last_message_id: Option<String>,
        last_message_text: Option<String>,
        last_message_timestamp: Option<i64>,
    ) -> anyhow::Result<()>;
    async fn store_message(&self, message: MessageItem) -> anyhow::Result<()>;
    async fn get_latest_message(
        &self,
        conversation_id: &str,
    ) -> anyhow::Result<Option<MessageItem>>;
    async fn get_latest_messages(
        &self,
        conversation_id: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<MessageItem>>;
    async fn get_messages_before(
        &self,
        conversation_id: &str,
        before_message_id: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<MessageItem>>;

    async fn exist_user_groups(&self) -> anyhow::Result<bool>;
    async fn apply_user_group_all(
        &self,
        groups: Vec<UserGroupData>,
        version: &str,
    ) -> anyhow::Result<()>;
    async fn apply_user_group_action(
        &self,
        action: UserGroupStorageAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<UserGroupData>>;
    async fn get_all_user_groups(&self) -> anyhow::Result<Vec<UserGroupData>>;
    async fn get_user_group_version(&self) -> anyhow::Result<Option<String>>;
    async fn clear_all_user_groups(&self) -> anyhow::Result<()>;

    async fn exist_group_members(&self, group_id: &str) -> anyhow::Result<bool>;
    async fn apply_group_member_all(
        &self,
        group_id: &str,
        members: Vec<GroupMemberData>,
        version: &str,
    ) -> anyhow::Result<()>;
    async fn apply_group_member_action(
        &self,
        group_id: &str,
        action: GroupMemberStorageAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<GroupMemberData>>;
    async fn get_all_group_members(&self, group_id: &str) -> anyhow::Result<Vec<GroupMemberData>>;
    async fn get_group_member(&self, group_id: &str, user_id: &str) -> anyhow::Result<Option<GroupMemberData>>;
    async fn get_group_member_version(&self, group_id: &str) -> anyhow::Result<Option<String>>;
    async fn clear_group_members(&self, group_id: &str) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct MemoryStore {
    inner: Arc<tokio::sync::Mutex<InnerStore>>,
}

struct InnerStore {
    access_token: Option<String>,
    refresh_token: Option<String>,
    uuid: Option<Uuid>,
    user_id: Option<String>,
    user_profile: Option<UserProfileData>,
    relations: HashMap<String, RelationUser>,
    relation_version: Option<String>,
    conversations: HashMap<String, ConversationRecord>,
    conversation_version: Option<String>,
    messages: HashMap<String, BTreeMap<String, MessageItem>>,
    // User Groups
    user_groups: HashMap<String, UserGroupData>,
    user_groups_version: Option<String>,
    // Group Members: group_id -> (user_id -> member)
    group_members: HashMap<String, HashMap<String, GroupMemberData>>,
    group_member_versions: HashMap<String, String>,
}

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore {
            inner: Arc::new(tokio::sync::Mutex::new(InnerStore {
                access_token: None,
                refresh_token: None,
                uuid: None,
                user_id: None,
                user_profile: None,
                relations: HashMap::new(),
                relation_version: None,
                conversations: HashMap::new(),
                conversation_version: None,
                messages: HashMap::new(),
                user_groups: HashMap::new(),
                user_groups_version: None,
                group_members: HashMap::new(),
                group_member_versions: HashMap::new(),
            })),
        }
    }
}

impl RippleStorage for MemoryStore {
    async fn exists_token(&self) -> anyhow::Result<bool> {
        // always return false for in-memory store
        Ok(false)
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

    async fn clear_token(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.access_token = None;
        inner.refresh_token = None;
        Ok(())
    }

    async fn get_stored_user_id(&self) -> anyhow::Result<Option<String>> {
        let inner = self.inner.lock().await;
        Ok(inner.user_id.clone())
    }

    async fn save_user_id(&self, user_id: &str) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.user_id = Some(user_id.to_string());
        Ok(())
    }

    async fn clear_all_data(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        // Clear all data except device_id (uuid) and token (which is the new user's token)
        // Token and user_id are managed separately by check_and_clear_on_user_change
        inner.user_profile = None;
        inner.relations.clear();
        inner.relation_version = None;
        inner.conversations.clear();
        inner.conversation_version = None;
        inner.messages.clear();
        inner.user_groups.clear();
        inner.user_groups_version = None;
        inner.group_members.clear();
        inner.group_member_versions.clear();
        Ok(())
    }

    async fn get_user_profile(&self) -> anyhow::Result<Option<UserProfileData>> {
        let inner = self.inner.lock().await;
        Ok(inner.user_profile.clone())
    }

    async fn save_user_profile(&self, profile: UserProfileData) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.user_profile = Some(profile);
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
        action: RelationStorageAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<RelationUser>> {
        let mut inner = self.inner.lock().await;
        inner.relation_version = Some(version);
        match action {
            RelationStorageAction::Upsert(relation) => {
                inner
                    .relations
                    .insert(relation.user_id.clone(), relation.clone());
                if need_result {
                    Ok(Some(relation))
                } else {
                    Ok(None)
                }
            }
            RelationStorageAction::UpdateRemarkName {
                user_id,
                remark_name,
            } => match inner.relations.get_mut(&user_id) {
                Some(relation) => {
                    relation.remark_name = Some(remark_name);
                    if need_result {
                        Ok(Some(relation.clone()))
                    } else {
                        Ok(None)
                    }
                }
                None => Ok(None),
            },

            RelationStorageAction::UpdateNickName { user_id, nick_name } => {
                match inner.relations.get_mut(&user_id) {
                    Some(relation) => {
                        relation.nick_name = nick_name;
                        if need_result {
                            Ok(Some(relation.clone()))
                        } else {
                            Ok(None)
                        }
                    }
                    None => Ok(None),
                }
            }

            RelationStorageAction::UpdateAvatar { user_id, avatar } => {
                match inner.relations.get_mut(&user_id) {
                    Some(relation) => {
                        relation.avatar = avatar;
                        if need_result {
                            Ok(Some(relation.clone()))
                        } else {
                            Ok(None)
                        }
                    }
                    None => Ok(None),
                }
            }

            RelationStorageAction::UpdateFlags { user_id, flags } => {
                match inner.relations.get_mut(&user_id) {
                    Some(relation) => {
                        relation.relation_flags = flags;
                        if need_result {
                            Ok(Some(relation.clone()))
                        } else {
                            Ok(None)
                        }
                    }
                    None => Ok(None),
                }
            }
            RelationStorageAction::Delete { user_id } => {
                inner.relations.remove(&user_id);
                Ok(None)
            }
            RelationStorageAction::UpdateNickNameAvatar {
                user_id,
                nick_name,
                avatar,
            } => match inner.relations.get_mut(&user_id) {
                Some(relation) => {
                    relation.nick_name = nick_name;
                    relation.avatar = avatar;
                    if need_result {
                        Ok(Some(relation.clone()))
                    } else {
                        Ok(None)
                    }
                }
                None => Ok(None),
            },
        }
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
        conversations: Vec<ConversationRecord>,
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
        action: ConversationStorageAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<ConversationRecord>> {
        let mut inner = self.inner.lock().await;
        inner.conversation_version = Some(version);
        match action {
            ConversationStorageAction::Create(conversation) => {
                inner
                    .conversations
                    .insert(conversation.conversation_id.clone(), conversation.clone());
                if need_result {
                    Ok(Some(conversation))
                } else {
                    Ok(None)
                }
            }
            ConversationStorageAction::UpdateLastReadMessageId {
                conversation_id,
                last_read_message_id,
            } => match inner.conversations.get_mut(&conversation_id) {
                Some(conv) => {
                    conv.last_read_message_id = Some(last_read_message_id);
                    if need_result {
                        Ok(Some(conv.clone()))
                    } else {
                        Ok(None)
                    }
                }
                None => Ok(None),
            },
            ConversationStorageAction::UpdateName {
                conversation_id,
                name,
            } => match inner.conversations.get_mut(&conversation_id) {
                Some(conv) => {
                    conv.name = name;
                    if need_result {
                        Ok(Some(conv.clone()))
                    } else {
                        Ok(None)
                    }
                }
                None => Ok(None),
            },
            ConversationStorageAction::UpdateAvatar {
                conversation_id,
                avatar,
            } => match inner.conversations.get_mut(&conversation_id) {
                Some(conv) => {
                    conv.avatar = Some(avatar);
                    if need_result {
                        Ok(Some(conv.clone()))
                    } else {
                        Ok(None)
                    }
                }
                None => Ok(None),
            },
            ConversationStorageAction::UpdateNameAvatar {
                conversation_id,
                name,
                avatar,
            } => match inner.conversations.get_mut(&conversation_id) {
                Some(conv) => {
                    conv.name = name;
                    conv.avatar = Some(avatar);
                    if need_result {
                        Ok(Some(conv.clone()))
                    } else {
                        Ok(None)
                    }
                }
                None => Ok(None),
            },
            ConversationStorageAction::Delete { conversation_id } => {
                let removed = inner.conversations.remove(&conversation_id);
                if need_result {
                    Ok(removed)
                } else {
                    Ok(None)
                }
            }
            ConversationStorageAction::UpdateBotSessionId {
                conversation_id,
                bot_session_id,
            } => match inner.conversations.get_mut(&conversation_id) {
                Some(conv) => {
                    conv.bot_session_id = Some(bot_session_id);
                    if need_result {
                        Ok(Some(conv.clone()))
                    } else {
                        Ok(None)
                    }
                }
                None => Ok(None),
            },
        }
    }
    async fn conversation_exists(&self, conversation_id: &str) -> anyhow::Result<bool> {
        let inner = self.inner.lock().await;
        Ok(inner.conversations.contains_key(conversation_id))
    }

    async fn get_conversation_by_id(
        &self,
        conversation_id: &str,
    ) -> anyhow::Result<Option<ConversationRecord>> {
        let inner = self.inner.lock().await;
        Ok(inner.conversations.get(conversation_id).cloned())
    }

    async fn get_all_conversations(&self) -> anyhow::Result<Vec<ConversationRecord>> {
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

    async fn update_conversation_summary(
        &self,
        conversation_id: &str,
        unread_count: i64,
        last_message_id: Option<String>,
        last_message_text: Option<String>,
        last_message_timestamp: Option<i64>,
    ) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        if let Some(conv) = inner.conversations.get_mut(conversation_id) {
            conv.unread_count = unread_count;
            conv.last_message_id = last_message_id;
            conv.last_message_text = last_message_text;
            conv.last_message_timestamp = last_message_timestamp;
        }
        Ok(())
    }

    async fn store_message(&self, message: MessageItem) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;

        // Store the message
        let conversation_messages = inner
            .messages
            .entry(message.conversation_id.clone())
            .or_insert_with(BTreeMap::new);
        conversation_messages.insert(message.message_id.clone(), message.clone());

        // Update conversation's last_message_id if this message is newer
        if let Some(conv) = inner.conversations.get_mut(&message.conversation_id) {
            let should_update = match &conv.last_message_id {
                Some(existing_id) => message.message_id > *existing_id,
                None => true,
            };
            if should_update {
                println!(
                    "[StoreEngine] store_message: updating last_message_id from {:?} to {}",
                    conv.last_message_id, message.message_id
                );
                conv.last_message_id = Some(message.message_id.clone());
            }
        } else {
            println!(
                "[StoreEngine] store_message: conversation {} not found in store",
                message.conversation_id
            );
        }

        Ok(())
    }

    async fn get_latest_message(
        &self,
        conversation_id: &str,
    ) -> anyhow::Result<Option<MessageItem>> {
        let inner = self.inner.lock().await;

        let messages = match inner.messages.get(conversation_id) {
            Some(msgs) => msgs,
            None => {
                return Ok(None);
            }
        };
        Ok(messages.iter().rev().next().map(|(_, msg)| msg.clone()))
    }

    async fn get_latest_messages(
        &self,
        conversation_id: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<MessageItem>> {
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
        let mut result: Vec<MessageItem> = messages
            .iter()
            .rev()
            .take(limit as usize)
            .map(|(_, msg)| msg.clone())
            .collect();
        result.reverse();
        println!("[StoreEngine] Returning {} latest messages", result.len());
        Ok(result)
    }

    async fn get_messages_before(
        &self,
        conversation_id: &str,
        before_message_id: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<MessageItem>> {
        let inner = self.inner.lock().await;

        let messages = match inner.messages.get(conversation_id) {
            Some(msgs) => msgs,
            None => {
                return Ok(Vec::new());
            }
        };
        Ok(messages
            .range(..before_message_id.to_string())
            .take(limit as usize)
            .map(|(_, msg)| msg.clone())
            .collect())
    }

    // User Groups implementations
    async fn exist_user_groups(&self) -> anyhow::Result<bool> {
        let inner = self.inner.lock().await;
        Ok(!inner.user_groups.is_empty())
    }

    async fn apply_user_group_all(
        &self,
        groups: Vec<UserGroupData>,
        version: &str,
    ) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.user_groups.clear();
        for group in groups {
            inner.user_groups.insert(group.group_id.clone(), group);
        }
        inner.user_groups_version = Some(version.to_string());
        Ok(())
    }

    async fn apply_user_group_action(
        &self,
        action: UserGroupStorageAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<UserGroupData>> {
        let mut inner = self.inner.lock().await;
        inner.user_groups_version = Some(version);
        match action {
            UserGroupStorageAction::Upsert(group) => {
                inner
                    .user_groups
                    .insert(group.group_id.clone(), group.clone());
                if need_result {
                    Ok(Some(group))
                } else {
                    Ok(None)
                }
            }
            UserGroupStorageAction::UpdateName { group_id, name } => {
                match inner.user_groups.get_mut(&group_id) {
                    Some(group) => {
                        group.group_name = name;
                        if need_result {
                            Ok(Some(group.clone()))
                        } else {
                            Ok(None)
                        }
                    }
                    None => Ok(None),
                }
            }
            UserGroupStorageAction::UpdateAvatar { group_id, avatar } => {
                match inner.user_groups.get_mut(&group_id) {
                    Some(group) => {
                        group.group_avatar = avatar;
                        if need_result {
                            Ok(Some(group.clone()))
                        } else {
                            Ok(None)
                        }
                    }
                    None => Ok(None),
                }
            }
            UserGroupStorageAction::Delete { group_id } => {
                inner.user_groups.remove(&group_id);
                // Also remove group members when user quits a group
                inner.group_members.remove(&group_id);
                inner.group_member_versions.remove(&group_id);
                Ok(None)
            }
        }
    }

    async fn get_all_user_groups(&self) -> anyhow::Result<Vec<UserGroupData>> {
        let inner = self.inner.lock().await;
        Ok(inner.user_groups.values().cloned().collect())
    }

    async fn get_user_group_version(&self) -> anyhow::Result<Option<String>> {
        let inner = self.inner.lock().await;
        Ok(inner.user_groups_version.clone())
    }

    async fn clear_all_user_groups(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.user_groups.clear();
        inner.user_groups_version = None;
        Ok(())
    }

    // Group Members implementations
    async fn exist_group_members(&self, group_id: &str) -> anyhow::Result<bool> {
        let inner = self.inner.lock().await;
        Ok(inner.group_members.contains_key(group_id))
    }

    async fn apply_group_member_all(
        &self,
        group_id: &str,
        members: Vec<GroupMemberData>,
        version: &str,
    ) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        let group_members = inner
            .group_members
            .entry(group_id.to_string())
            .or_insert_with(HashMap::new);
        group_members.clear();
        for member in members {
            group_members.insert(member.user_id.clone(), member);
        }
        inner
            .group_member_versions
            .insert(group_id.to_string(), version.to_string());
        Ok(())
    }

    async fn apply_group_member_action(
        &self,
        group_id: &str,
        action: GroupMemberStorageAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<GroupMemberData>> {
        let mut inner = self.inner.lock().await;
        inner
            .group_member_versions
            .insert(group_id.to_string(), version);
        let group_members = inner
            .group_members
            .entry(group_id.to_string())
            .or_insert_with(HashMap::new);

        match action {
            GroupMemberStorageAction::Upsert(member) => {
                group_members.insert(member.user_id.clone(), member.clone());
                if need_result {
                    Ok(Some(member))
                } else {
                    Ok(None)
                }
            }
            GroupMemberStorageAction::UpdateName { user_id, name } => {
                match group_members.get_mut(&user_id) {
                    Some(member) => {
                        member.name = name;
                        if need_result {
                            Ok(Some(member.clone()))
                        } else {
                            Ok(None)
                        }
                    }
                    None => Ok(None),
                }
            }
            GroupMemberStorageAction::UpdateAvatar { user_id, avatar } => {
                match group_members.get_mut(&user_id) {
                    Some(member) => {
                        member.avatar = avatar;
                        if need_result {
                            Ok(Some(member.clone()))
                        } else {
                            Ok(None)
                        }
                    }
                    None => Ok(None),
                }
            }
            GroupMemberStorageAction::Delete { user_id } => {
                group_members.remove(&user_id);
                Ok(None)
            }
        }
    }

    async fn get_all_group_members(&self, group_id: &str) -> anyhow::Result<Vec<GroupMemberData>> {
        let inner = self.inner.lock().await;
        match inner.group_members.get(group_id) {
            Some(members) => Ok(members.values().cloned().collect()),
            None => Ok(Vec::new()),
        }
    }

    async fn get_group_member(&self, group_id: &str, user_id: &str) -> anyhow::Result<Option<GroupMemberData>> {
        let inner = self.inner.lock().await;
        match inner.group_members.get(group_id) {
            Some(members) => Ok(members.get(user_id).cloned()),
            None => Ok(None),
        }
    }

    async fn get_group_member_version(&self, group_id: &str) -> anyhow::Result<Option<String>> {
        let inner = self.inner.lock().await;
        Ok(inner.group_member_versions.get(group_id).cloned())
    }

    async fn clear_group_members(&self, group_id: &str) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.group_members.remove(group_id);
        inner.group_member_versions.remove(group_id);
        Ok(())
    }
}
