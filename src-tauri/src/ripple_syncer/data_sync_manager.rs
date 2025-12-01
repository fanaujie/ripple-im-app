use crate::ripple_api::api_response::{
    CommonResponse, ConversationChange, ConversationSyncData, ReadMessagesData, RelationChange,
    RelationUser, UserProfileData,
};
use crate::ripple_api::RippleApi;
use crate::ripple_syncer::conversation_operation::ConversationStorageAction;
use crate::ripple_syncer::relation_operation::RelationAction;
use crate::store_engine::store_engine::{
    RippleStorage, StorageConversationData, StorageMessageData, Token,
};
use uuid::Uuid;

#[derive(Debug)]
pub enum RelationSyncResult {
    FullSync {
        relations: Vec<RelationUser>,
    },
    IncrementalSync {
        changes: Vec<(u64, String, Option<RelationUser>)>,
    },
    NoChange,
}

#[derive(Debug)]
pub enum ConversationSyncResult {
    FullSync {
        conversations: Vec<StorageConversationData>,
    },
    IncrementalSync {
        changes: Vec<(i32, String, Option<StorageConversationData>)>,
    },
    NoChange,
}

#[derive(Clone)]
pub struct DataSyncManager<S: RippleStorage> {
    ripple_api: RippleApi<S>,
    store_engine: S,
}

impl<S: RippleStorage> DataSyncManager<S> {
    pub fn new(ripple_api: RippleApi<S>, store_engine: S) -> Self {
        DataSyncManager {
            ripple_api,
            store_engine,
        }
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        let uuid = self.store_engine.get_device_id().await?;
        if uuid.is_none() {
            let new_id = Uuid::new_v4();
            self.store_engine.save_device_id(&new_id).await?;
        }
        self.sync_user_profile().await?;
        if !self.exist_relations().await? {
            self.sync_all_relations().await?;
        } else {
            self.process_relations_sync(false).await?;
        }
        if !self.exist_conversations().await? {
            self.sync_all_conversations().await?;
        } else {
            self.process_conversations_sync(false).await?;
        }
        Ok(())
    }

    pub async fn get_device_id(&self) -> anyhow::Result<Uuid> {
        let uuid = self.store_engine.get_device_id().await?;
        match uuid {
            Some(id) => Ok(id),
            None => anyhow::bail!("Device ID not found in storage"),
        }
    }

    pub async fn get_token(&self) -> anyhow::Result<Token> {
        match self.store_engine.get_token().await? {
            Some(token) => Ok(token),
            None => anyhow::bail!("Auth token not found in storage"),
        }
    }

    pub async fn exists_token(&self) -> anyhow::Result<bool> {
        self.store_engine.exists_token().await
    }

    pub async fn exists_profile(&self) -> anyhow::Result<bool> {
        self.store_engine.exists_profile().await
    }
    pub async fn exist_relations(&self) -> anyhow::Result<bool> {
        self.store_engine.exist_relations().await
    }
    pub async fn exist_conversations(&self) -> anyhow::Result<bool> {
        self.store_engine.exist_conversations().await
    }

    pub async fn sync_user_profile(&self) -> anyhow::Result<()> {
        let profile_response = self.ripple_api.get_user_profile().await?;

        if profile_response.code != 200 {
            anyhow::bail!(
                "Failed to get user profile: code={}, message={}",
                profile_response.code,
                profile_response.message
            )
        }

        self.store_engine
            .save_user_profile(&profile_response.data)
            .await?;

        Ok(())
    }

    pub async fn sync_all_relations(&self) -> anyhow::Result<()> {
        let mut all_users = Vec::new();
        let mut next_page_token: Option<String> = None;
        let page_size = 50; // Max page size

        let relation_version = self.ripple_api.get_relation_version().await?;
        if relation_version.code != 200 {
            anyhow::bail!(
                "Failed to get relation version: code={}, message={}",
                relation_version.code,
                relation_version.message
            )
        }

        let version = match relation_version.data.latest_version {
            Some(v) => v,
            None => return Ok(()),
        };
        loop {
            let relations_response = self
                .ripple_api
                .get_relations(next_page_token.clone(), page_size)
                .await?;
            if relations_response.code != 200 {
                anyhow::bail!(
                    "Failed to get relations: code={}, message={}",
                    relations_response.code,
                    relations_response.message
                )
            }
            all_users.extend(relations_response.data.users);
            if !relations_response.data.has_more {
                break;
            }
            next_page_token = relations_response.data.next_page_token;
        }
        self.store_engine.clear_all_relations().await?;
        self.store_engine
            .apply_relation_all(all_users, &version)
            .await?;
        Ok(())
    }

    pub async fn sync_relations_incremental(
        &self,
        last_version: Option<String>,
    ) -> anyhow::Result<crate::ripple_api::api_response::RelationsSyncData> {
        let sync_response = self.ripple_api.sync_relations(last_version).await?;

        if sync_response.code != 200 {
            anyhow::bail!(
                "Failed to sync relations: code={}, message={}",
                sync_response.code,
                sync_response.message
            )
        }
        Ok(sync_response.data)
    }

    pub async fn process_relations_sync(
        &self,
        need_result: bool,
    ) -> anyhow::Result<Option<RelationSyncResult>> {
        let last_version = self.get_relation_version().await?;
        let sync_data = self.sync_relations_incremental(last_version).await?;
        if sync_data.full_sync {
            self.sync_all_relations().await?;
            if need_result {
                let relations = self.get_relations().await?;
                return Ok(Some(RelationSyncResult::FullSync { relations }));
            }
            return Ok(None);
        }
        if sync_data.changes.is_empty() {
            return Ok(if need_result {
                Some(RelationSyncResult::NoChange)
            } else {
                None
            });
        }
        if need_result {
            let mut changes = Vec::new();
            for change in sync_data.changes {
                let operation = change.operation;
                let user_id = change.user_id.clone();
                let action = self.to_relation_storage_action(&change);

                // One call to both write and read the updated data
                let updated_user = self
                    .store_engine
                    .apply_relation_action(action, change.version.clone(), true)
                    .await?;

                changes.push((operation, user_id, updated_user));
            }
            Ok(Some(RelationSyncResult::IncrementalSync { changes }))
        } else {
            for change in sync_data.changes {
                let action = self.to_relation_storage_action(&change);
                // Don't need result, don't read data
                self.store_engine
                    .apply_relation_action(action, change.version.clone(), false)
                    .await?;
            }
            Ok(None)
        }
    }

    pub async fn get_profile(&self) -> anyhow::Result<Option<UserProfileData>> {
        self.store_engine.get_user_profile().await
    }

    pub async fn get_relations(&self) -> anyhow::Result<Vec<RelationUser>> {
        self.store_engine.get_all_relations().await
    }

    pub async fn sync_all_conversations(&self) -> anyhow::Result<()> {
        let mut all_conversations = Vec::new();
        let mut next_page_token: Option<String> = None;
        let page_size = 50; // Default page size
        let conversation_version = self.ripple_api.get_conversation_version().await?;
        if conversation_version.code != 200 {
            anyhow::bail!(
                "Failed to get conversation version: code={}, message={}",
                conversation_version.code,
                conversation_version.message
            )
        }
        let last_version = match conversation_version.data.latest_version {
            Some(v) => v,
            None => return Ok(()),
        };
        // Paginate through all conversations
        loop {
            let conversations_response = self
                .ripple_api
                .get_conversations(next_page_token.clone(), page_size)
                .await?;
            if conversations_response.code != 200 {
                anyhow::bail!(
                    "Failed to get conversations: code={}, message={}",
                    conversations_response.code,
                    conversations_response.message
                )
            }
            all_conversations.extend(conversations_response.data.conversations);
            if !conversations_response.data.has_more {
                break;
            }
            next_page_token = conversations_response.data.next_page_token;
        }
        let storage_conversation_data: Vec<StorageConversationData> = all_conversations
            .into_iter()
            .map(|item| item.into())
            .collect();
        self.store_engine.clear_all_conversations().await?;
        self.store_engine
            .apply_conversation_all(storage_conversation_data, &last_version)
            .await?;
        Ok(())
    }

    pub async fn sync_conversations_incremental(
        &self,
        last_version: Option<String>,
    ) -> anyhow::Result<ConversationSyncData> {
        let sync_response = self.ripple_api.sync_conversations(last_version).await?;

        if sync_response.code != 200 {
            anyhow::bail!(
                "Failed to sync conversations: code={}, message={}",
                sync_response.code,
                sync_response.message
            )
        }
        Ok(sync_response.data)
    }

    pub async fn process_conversations_sync(
        &self,
        need_result: bool,
    ) -> anyhow::Result<Option<ConversationSyncResult>> {
        let last_version = self.get_conversation_version().await?;
        let sync_data = self.sync_conversations_incremental(last_version).await?;
        if sync_data.full_sync {
            self.sync_all_conversations().await?;
            if need_result {
                let conversations = self.get_conversations().await?;
                return Ok(Some(ConversationSyncResult::FullSync { conversations }));
            }
            return Ok(None);
        }
        if sync_data.changes.is_empty() {
            return Ok(if need_result {
                Some(ConversationSyncResult::NoChange)
            } else {
                None
            });
        }
        let user_id = match self.get_profile().await? {
            Some(profile) => profile.user_id.parse::<i64>().unwrap_or(0),
            None => {
                anyhow::bail!("Cannot get user_id for conversation sync");
            }
        };
        if need_result {
            let mut changes = Vec::new();
            for change in sync_data.changes {
                let operation = change.operation;
                let conversation_id = change.conversation_id.clone();
                let action = self.to_conversation_storage_action(&change);
                let updated_conversation = self
                    .store_engine
                    .apply_conversation_action(action, change.version.clone(), user_id, true)
                    .await?;

                changes.push((operation, conversation_id, updated_conversation));
            }
            Ok(Some(ConversationSyncResult::IncrementalSync { changes }))
        } else {
            for change in sync_data.changes {
                let action = self.to_conversation_storage_action(&change);
                self.store_engine
                    .apply_conversation_action(action, change.version.clone(), user_id, false)
                    .await?;
            }
            Ok(None)
        }
    }

    pub async fn get_conversations(&self) -> anyhow::Result<Vec<StorageConversationData>> {
        self.store_engine.get_all_conversations().await
    }

    pub async fn get_relation_version(&self) -> anyhow::Result<Option<String>> {
        self.store_engine.get_relation_version().await
    }

    pub async fn apply_relation_action(
        &self,
        action: RelationAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<RelationUser>> {
        self.store_engine
            .apply_relation_action(action, version, need_result)
            .await
    }

    pub async fn get_relation(&self, user_id: &str) -> anyhow::Result<Option<RelationUser>> {
        self.store_engine.get_relation(user_id).await
    }

    pub async fn get_conversation_version(&self) -> anyhow::Result<Option<String>> {
        self.store_engine.get_conversation_version().await
    }

    pub async fn apply_conversation_action(
        &self,
        action: ConversationStorageAction,
        version: String,
        user_id: i64,
        need_result: bool,
    ) -> anyhow::Result<Option<StorageConversationData>> {
        self.store_engine
            .apply_conversation_action(action, version, user_id, need_result)
            .await
    }

    pub async fn get_conversation(
        &self,
        conversation_id: &str,
    ) -> anyhow::Result<Option<StorageConversationData>> {
        self.store_engine.get_conversation(conversation_id).await
    }

    pub async fn read_latest_messages(
        &self,
        conversation_id: String,
        read_size: u32,
    ) -> anyhow::Result<ReadMessagesData> {
        let storage_messages = self
            .store_engine
            .get_latest_messages(&conversation_id, read_size)
            .await?;

        if storage_messages.len() >= read_size as usize {
            let messages: Vec<_> = storage_messages.into_iter().map(|msg| msg.into()).collect();
            return Ok(ReadMessagesData { messages });
        }
        let api_response = self
            .ripple_api
            .read_messages(conversation_id.clone(), "0".to_string(), read_size)
            .await?;

        if api_response.code != 200 {
            eprintln!(
                "[DataSyncManager] API returned error: code={}, message={}",
                api_response.code, api_response.message
            );
            anyhow::bail!(
                "Failed to read latest messages: code={}, message={}",
                api_response.code,
                api_response.message
            )
        }
        for message_item in &api_response.data.messages {
            self.store_engine.store_message(message_item.into()).await?;
        }
        Ok(api_response.data)
    }

    pub async fn read_messages_before(
        &self,
        conversation_id: String,
        before_message_id: String,
        read_size: u32,
    ) -> anyhow::Result<ReadMessagesData> {
        let before_id = before_message_id.parse::<i64>().unwrap_or(0);
        if before_id == 0 {
            return Ok(ReadMessagesData {
                messages: Vec::new(),
            });
        }

        let storage_messages = self
            .store_engine
            .get_messages_before(&conversation_id, before_id, read_size)
            .await?;

        if storage_messages.len() >= read_size as usize {
            let messages: Vec<_> = storage_messages.into_iter().map(|msg| msg.into()).collect();
            return Ok(ReadMessagesData { messages });
        }

        let api_response = self
            .ripple_api
            .read_messages(conversation_id.clone(), "0".to_string(), read_size)
            .await?;

        if api_response.code != 200 {
            eprintln!(
                "[DataSyncManager] API returned error: code={}, message={}",
                api_response.code, api_response.message
            );
            anyhow::bail!(
                "Failed to read messages before: code={}, message={}",
                api_response.code,
                api_response.message
            )
        }
        for message_item in &api_response.data.messages {
            self.store_engine.store_message(message_item.into()).await?;
        }

        // After storing, retry getting messages before the specified ID
        let storage_messages = self
            .store_engine
            .get_messages_before(&conversation_id, before_id, read_size)
            .await?;

        let messages: Vec<_> = storage_messages.into_iter().map(|msg| msg.into()).collect();
        Ok(ReadMessagesData { messages })
    }

    pub async fn store_message(&self, message: StorageMessageData) -> anyhow::Result<()> {
        self.store_engine.store_message(message).await
    }
    pub async fn mark_last_read_message_id(
        &self,
        conversation_id: String,
        message_id: String,
    ) -> anyhow::Result<CommonResponse> {
        let response = self
            .ripple_api
            .mark_last_read_message_id(conversation_id.clone(), message_id.clone())
            .await?;
        Ok(response)
    }

    fn to_relation_storage_action(&self, change: &RelationChange) -> RelationAction {
        use crate::ripple_syncer::relation_operation::relation_operation;

        match change.operation {
            relation_operation::ADD_FRIEND => RelationAction::Upsert(change.into()),
            relation_operation::UPDATE_FRIEND_REMARK_NAME => RelationAction::UpdateRemarkName {
                user_id: change.user_id.clone(),
                remark_name: change.remark_name.clone().unwrap_or_default(),
            },
            relation_operation::DELETE_FRIEND => RelationAction::Delete {
                user_id: change.user_id.clone(),
            },
            relation_operation::ADD_BLOCK => RelationAction::UpdateFlags {
                user_id: change.user_id.clone(),
                flags: change.relation_flags,
            },
            relation_operation::DELETE_BLOCK => RelationAction::Delete {
                user_id: change.user_id.clone(),
            },
            relation_operation::UNBLOCK_RESTORE_FRIEND => RelationAction::UpdateFlags {
                user_id: change.user_id.clone(),
                flags: change.relation_flags,
            },
            relation_operation::HIDE_BLOCK => RelationAction::UpdateFlags {
                user_id: change.user_id.clone(),
                flags: change.relation_flags,
            },
            relation_operation::UPDATE_FRIEND_NICK_NAME => RelationAction::UpdateNickName {
                user_id: change.user_id.clone(),
                nick_name: change.nick_name.clone().unwrap_or_default(),
            },
            relation_operation::UPDATE_FRIEND_AVATAR => RelationAction::UpdateAvatar {
                user_id: change.user_id.clone(),
                avatar: change.avatar.clone(),
            },
            relation_operation::BLOCK_STRANGER => RelationAction::Upsert(change.into()),
            relation_operation::SYNC_FRIEND_INFO => RelationAction::UpdateNickNameAvatar {
                user_id: change.user_id.clone(),
                nick_name: change.nick_name.clone().unwrap_or_default(),
                avatar: change.avatar.clone(),
            },
            _ => {
                eprintln!(
                    "[DataSyncManager] Unknown relation operation: {}, using no-op",
                    change.operation
                );
                RelationAction::UpdateAvatar {
                    user_id: change.user_id.clone(),
                    avatar: None,
                }
            }
        }
    }

    fn to_conversation_storage_action(
        &self,
        change: &ConversationChange,
    ) -> ConversationStorageAction {
        use crate::ripple_syncer::conversation_operation::conversation_operation;

        match change.operation {
            conversation_operation::CREATE_CONVERSATION => {
                ConversationStorageAction::Create(change.into())
            }
            conversation_operation::NEW_MESSAGE => {
                ConversationStorageAction::NewMessage(change.into())
            }
            conversation_operation::READ_MESSAGE => ConversationStorageAction::UpdateReadStatus {
                conversation_id: change.conversation_id.clone(),
                last_read_message_id: change
                    .last_read_message_id
                    .as_ref()
                    .and_then(|id_str| id_str.parse::<i64>().ok()),
            },
            conversation_operation::UPDATE_CONVERSATION_NAME => {
                ConversationStorageAction::UpdateName {
                    conversation_id: change.conversation_id.clone(),
                    name: change.name.clone().unwrap_or_default(),
                }
            }
            conversation_operation::UPDATE_CONVERSATION_AVATAR => {
                ConversationStorageAction::UpdateAvatar {
                    conversation_id: change.conversation_id.clone(),
                    avatar: change.avatar.clone().unwrap_or_default(),
                }
            }
            conversation_operation::DELETE_CONVERSATION => ConversationStorageAction::Delete {
                conversation_id: change.conversation_id.clone(),
            },
            _ => {
                panic!(
                    "[DataSyncManager] Unknown conversation operation: {}",
                    change.operation
                )
            }
        }
    }
}
