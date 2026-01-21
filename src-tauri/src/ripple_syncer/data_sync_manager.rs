use crate::ripple_api::api_response::{
    CommonResponse, ConversationChange, ConversationOperation, ConversationSyncData,
    GroupMemberData, GroupMemberOperation, GroupSyncData, MessageItem, ReadMessagesData,
    RelationChange, RelationOperation, RelationUser, RelationUsers, RelationsSyncData,
    UserGroupChange, UserGroupData, UserGroupOperation, UserGroupSyncData, UserProfileData,
};
use crate::ripple_api::auth_token_parser::AuthTokenParser;
use crate::ripple_api::RippleApi;
use crate::ripple_syncer::event_emitter::UIConversations;
use crate::ripple_syncer::incremental_operations::{process_incremental_operations, Operation};

use crate::store_engine::store_engine::{
    ConversationRecord, ConversationStorageAction, GroupMemberStorageAction, RelationStorageAction,
    RippleStorage, Token, UserGroupStorageAction,
};
use uuid::Uuid;

#[derive(Debug)]
pub enum ConversationSyncResult {
    FullSync {
        conversations: UIConversations,
    },
    IncrementalSync {
        insert: UIConversations,
        update: UIConversations,
        delete: Vec<String>,
    },
    NoChange,
}

#[derive(Debug)]
pub enum RelationSyncResult {
    FullSync {
        relations: RelationUsers,
    },
    IncrementalSync {
        insert: Vec<RelationUser>,
        update: Vec<RelationUser>,
        delete: Vec<String>,
    },
    NoChange,
}

#[derive(Debug)]
pub enum UserGroupSyncResult {
    FullSync {
        groups: Vec<UserGroupData>,
    },
    IncrementalSync {
        insert: Vec<UserGroupData>,
        update: Vec<UserGroupData>,
        delete: Vec<String>,
    },
    NoChange,
}

#[derive(Debug)]
pub enum GroupMemberSyncResult {
    FullSync {
        group_id: String,
        members: Vec<GroupMemberData>,
    },
    IncrementalSync {
        group_id: String,
        insert: Vec<GroupMemberData>,
        update: Vec<GroupMemberData>,
        delete: Vec<String>,
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
            self.sync_conversation_summaries().await?;
        }
        if !self.exist_user_groups().await? {
            self.sync_all_user_groups().await?;
        } else {
            self.process_user_groups_sync(false).await?;
        }

        let user_groups = self.store_engine.get_all_user_groups().await?;
        for group in user_groups {
            if !self
                .store_engine
                .exist_group_members(&group.group_id)
                .await?
            {
                self.sync_all_group_members(&group.group_id).await?;
            } else {
                self.process_group_members_sync(&group.group_id, false)
                    .await?;
            }
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

    pub async fn clear_token(&self) -> anyhow::Result<()> {
        self.store_engine.clear_token().await
    }

    pub async fn check_and_clear_on_user_change(&self) -> anyhow::Result<bool> {
        let token = self.get_token().await?;
        let claims = AuthTokenParser::decode_jwt_payload(&token.access_token)?;
        let new_user_id = claims.get_sub();

        let stored_user_id = self.store_engine.get_stored_user_id().await?;

        match stored_user_id {
            Some(old_user_id) if old_user_id != new_user_id => {
                // User changed, clear all data
                println!(
                    "User changed from {} to {}, clearing all stored data",
                    old_user_id, new_user_id
                );
                self.store_engine.clear_all_data().await?;
                self.store_engine.save_user_id(&new_user_id).await?;
                Ok(true)
            }
            None => {
                self.store_engine.save_user_id(&new_user_id).await?;
                Ok(false)
            }
            Some(_) => {
                // Same user, no action needed
                Ok(false)
            }
        }
    }

    pub async fn exists_profile(&self) -> anyhow::Result<bool> {
        match self.store_engine.get_user_profile().await? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
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
            .save_user_profile(profile_response.data)
            .await?;

        Ok(())
    }

    pub async fn sync_all_relations(&self) -> anyhow::Result<()> {
        let mut all_users = Vec::new();
        let mut next_page_token: Option<String> = None;
        let page_size = 50; // Max page size
        let mut last_version: Option<String> = None;
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
                last_version = relations_response.data.last_version;
                break;
            }
            next_page_token = relations_response.data.next_page_token;
        }
        if !all_users.is_empty() && last_version.is_some() {
            self.store_engine.clear_all_relations().await?;
            self.store_engine
                .apply_relation_all(all_users.clone(), &last_version.unwrap())
                .await?;
        }
        Ok(())
    }

    async fn sync_incremental_relation_change(
        &self,
        last_version: Option<String>,
    ) -> anyhow::Result<RelationsSyncData> {
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
        let sync_data = self.sync_incremental_relation_change(last_version).await?;
        if sync_data.full_sync {
            self.sync_all_relations().await?;
            if need_result {
                return Ok(Some(RelationSyncResult::FullSync {
                    relations: RelationUsers {
                        users: self.store_engine.get_all_relations().await?,
                    },
                }));
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
        let mut operations: Vec<Operation<RelationUser, RelationOperation>> = Vec::new();
        for change in sync_data.changes.into_iter() {
            let id = change.user_id.clone();
            let op = change.operation.clone();
            let version = change.version.clone();
            let relation_storage_action = self.to_relation_storage_action(change);
            let data = self
                .store_engine
                .apply_relation_action(relation_storage_action, version, true)
                .await?;
            operations.push(Operation { id, op, data });
        }
        let result = process_incremental_operations(operations);

        Ok(Some(RelationSyncResult::IncrementalSync {
            insert: result.inserts.into_iter().map(|item| item.data).collect(),
            update: result.updates.into_iter().map(|item| item.data).collect(),
            delete: result.deletes.into_iter().map(|item| item.id).collect(),
        }))
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
        let mut last_version: Option<String> = None;
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
                if conversations_response.data.last_version.is_some() {
                    last_version = conversations_response.data.last_version;
                }
                break;
            }
            next_page_token = conversations_response.data.next_page_token;
        }
        if !all_conversations.is_empty() && last_version.is_some() {
            let storage_conversation_data: Vec<ConversationRecord> = all_conversations
                .into_iter()
                .map(|item| item.into())
                .collect();
            self.store_engine.clear_all_conversations().await?;
            self.store_engine
                .apply_conversation_all(storage_conversation_data, &last_version.unwrap())
                .await?;
        }
        Ok(())
    }

    async fn sync_incremental_conversation_change(
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
        let sync_data = self
            .sync_incremental_conversation_change(last_version)
            .await?;

        if sync_data.full_sync {
            self.sync_all_conversations().await?;
            if need_result {
                return Ok(Some(ConversationSyncResult::FullSync {
                    conversations: self.store_engine.get_all_conversations().await?.into(),
                }));
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
        let mut operations: Vec<Operation<ConversationRecord, ConversationOperation>> = Vec::new();
        for change in sync_data.changes.into_iter() {
            let id = change.conversation_id.clone();
            let version = change.version.clone();
            let op = change.operation.clone();
            let relation_storage_action = self.to_conversation_storage_action(change);
            let data = self
                .store_engine
                .apply_conversation_action(relation_storage_action, version, true)
                .await?;
            operations.push(Operation { id, op, data });
        }
        let result = process_incremental_operations(operations);

        Ok(Some(ConversationSyncResult::IncrementalSync {
            insert: result
                .inserts
                .into_iter()
                .map(|item| item.data)
                .collect::<Vec<ConversationRecord>>()
                .into(),
            update: result
                .updates
                .into_iter()
                .map(|item| item.data)
                .collect::<Vec<ConversationRecord>>()
                .into(),
            delete: result.deletes.into_iter().map(|item| item.id).collect(),
        }))
    }

    pub async fn get_conversations(&self) -> anyhow::Result<Vec<ConversationRecord>> {
        self.store_engine.get_all_conversations().await
    }

    pub async fn get_conversation(
        &self,
        conversation_id: &str,
    ) -> anyhow::Result<Option<ConversationRecord>> {
        self.store_engine
            .get_conversation_by_id(conversation_id)
            .await
    }

    pub async fn sync_conversation_summaries(&self) -> anyhow::Result<()> {
        let conversations = self.store_engine.get_all_conversations().await?;
        if conversations.is_empty() {
            return Ok(());
        }

        let conversation_ids: Vec<String> = conversations
            .iter()
            .map(|c| c.conversation_id.clone())
            .collect();

        let response = self
            .ripple_api
            .get_conversation_summaries(conversation_ids)
            .await?;

        if response.code != 200 {
            anyhow::bail!(
                "Failed to get conversation summaries: code={}, message={}",
                response.code,
                response.message
            )
        }

        for summary in response.data.summaries {
            // Pass raw template text to storage - frontend handles personalization
            self.store_engine
                .update_conversation_summary(
                    &summary.conversation_id,
                    summary.unread_count,
                    summary.last_message_id,
                    summary.last_message_text.clone(),
                    Some(summary.last_message_timestamp),
                )
                .await?;
        }

        Ok(())
    }

    async fn get_relation_version(&self) -> anyhow::Result<Option<String>> {
        self.store_engine.get_relation_version().await
    }

    pub async fn get_conversation_version(&self) -> anyhow::Result<Option<String>> {
        self.store_engine.get_conversation_version().await
    }

    pub async fn conversation_exists(&self, conversation_id: &str) -> anyhow::Result<bool> {
        self.store_engine.conversation_exists(conversation_id).await
    }

    /// Load latest messages for a conversation, filling any gaps from offline period
    pub async fn read_latest_messages(
        &self,
        conversation_id: String,
        read_size: u32,
        _last_read_message_id: String,
    ) -> anyhow::Result<ReadMessagesData> {
        // 1. Get cached messages from local storage
        let mut storage_messages = self
            .store_engine
            .get_latest_messages(&conversation_id, read_size)
            .await?;

        // 2. Check for gap between local cache and server
        let conversation = self
            .store_engine
            .get_conversation_by_id(&conversation_id)
            .await?;
        let server_last_msg_id = conversation
            .as_ref()
            .and_then(|c| c.last_message_id.as_ref());
        let cache_newest_msg_id = storage_messages.last().map(|m| &m.message_id);

        let has_gap = match (server_last_msg_id, cache_newest_msg_id) {
            (Some(server), Some(cache)) => server != cache,
            (Some(_), None) => true,
            _ => false,
        };

        // 3. Fill gap if exists (fetch messages newer than cache)
        if has_gap {
            let after_id = cache_newest_msg_id
                .cloned()
                .unwrap_or_else(|| "0".to_string());

            println!(
                "[DataSyncManager] Filling gap for {}: after_id={}, server_last={:?}",
                conversation_id, after_id, server_last_msg_id
            );

            let api_response = self
                .ripple_api
                .read_messages_after(conversation_id.clone(), after_id, read_size)
                .await?;

            if api_response.code == 200 && !api_response.data.messages.is_empty() {
                println!(
                    "[DataSyncManager] Fetched {} new messages for {}",
                    api_response.data.messages.len(),
                    conversation_id
                );

                for msg in &api_response.data.messages {
                    self.store_engine.store_message(msg.clone()).await?;
                }
                storage_messages.extend(api_response.data.messages);
            }
        }

        Ok(ReadMessagesData {
            messages: storage_messages,
        })
    }

    /// Load older messages before a specific message ID (for pagination / scrolling up)
    pub async fn read_messages_before(
        &self,
        conversation_id: String,
        before_message_id: String,
        read_size: u32,
    ) -> anyhow::Result<ReadMessagesData> {
        // 1. Try local cache first
        let storage_messages = self
            .store_engine
            .get_messages_before(&conversation_id, &before_message_id, read_size)
            .await?;

        if storage_messages.len() >= read_size as usize {
            return Ok(ReadMessagesData {
                messages: storage_messages,
            });
        }

        // 2. Fetch from API if local cache insufficient
        let api_response = self
            .ripple_api
            .read_messages(conversation_id.clone(), before_message_id, read_size)
            .await?;

        if api_response.code != 200 {
            anyhow::bail!(
                "Failed to read messages before: code={}, message={}",
                api_response.code,
                api_response.message
            )
        }

        for msg in &api_response.data.messages {
            self.store_engine.store_message(msg.clone()).await?;
        }

        Ok(api_response.data)
    }

    /// Store a message in local cache (for WebSocket-received messages)
    pub async fn store_message(&self, message: MessageItem) -> anyhow::Result<()> {
        self.store_engine.store_message(message).await
    }

    pub async fn mark_last_read_message_id(
        &self,
        conversation_id: String,
        message_id: String,
    ) -> anyhow::Result<CommonResponse> {
        let response = self
            .ripple_api
            .update_read_position(conversation_id.clone(), message_id.clone())
            .await?;
        Ok(response)
    }

    fn to_relation_storage_action(&self, change: RelationChange) -> RelationStorageAction {
        match change.operation {
            RelationOperation::AddFriend => RelationStorageAction::Upsert(change.into()),
            RelationOperation::UpdateFriendRemarkName => RelationStorageAction::UpdateRemarkName {
                user_id: change.user_id,
                remark_name: change.remark_name.unwrap_or_default(),
            },
            RelationOperation::DeleteFriend => RelationStorageAction::Delete {
                user_id: change.user_id,
            },
            RelationOperation::AddBlock => RelationStorageAction::UpdateFlags {
                user_id: change.user_id,
                flags: change.relation_flags,
            },
            RelationOperation::DeleteBlock => RelationStorageAction::Delete {
                user_id: change.user_id,
            },
            RelationOperation::UnblockRestoreFriend => RelationStorageAction::UpdateFlags {
                user_id: change.user_id,
                flags: change.relation_flags,
            },
            RelationOperation::HideBlock => RelationStorageAction::UpdateFlags {
                user_id: change.user_id,
                flags: change.relation_flags,
            },
            RelationOperation::UpdateFriendNickName => RelationStorageAction::UpdateNickName {
                user_id: change.user_id,
                nick_name: change.nick_name.unwrap_or_default(),
            },
            RelationOperation::UpdateFriendAvatar => RelationStorageAction::UpdateAvatar {
                user_id: change.user_id,
                avatar: change.avatar,
            },
            RelationOperation::BlockStranger => RelationStorageAction::Upsert(change.into()),
            RelationOperation::SyncFriendInfo => RelationStorageAction::UpdateNickNameAvatar {
                user_id: change.user_id,
                nick_name: change.nick_name.unwrap_or_default(),
                avatar: change.avatar,
            },
            _ => {
                panic!("[DataSyncManager] Unknown relation operation")
            }
        }
    }

    fn to_conversation_storage_action(
        &self,
        change: ConversationChange,
    ) -> ConversationStorageAction {
        match change.operation {
            ConversationOperation::CreateConversation => {
                ConversationStorageAction::Create(change.into())
            }
            ConversationOperation::UpdateLastReadMessageId => {
                ConversationStorageAction::UpdateLastReadMessageId {
                    conversation_id: change.conversation_id,
                    last_read_message_id: change.last_read_message_id.unwrap(),
                }
            }
            ConversationOperation::UpdateConversationName => {
                ConversationStorageAction::UpdateName {
                    conversation_id: change.conversation_id,
                    name: change.name.unwrap(),
                }
            }
            ConversationOperation::UpdateConversationAvatar => {
                ConversationStorageAction::UpdateAvatar {
                    conversation_id: change.conversation_id,
                    avatar: change.avatar.unwrap(),
                }
            }
            ConversationOperation::UpdateConversationNameAvatar => {
                ConversationStorageAction::UpdateNameAvatar {
                    conversation_id: change.conversation_id,
                    name: change.name.unwrap(),
                    avatar: change.avatar.unwrap(),
                }
            }
            ConversationOperation::RemoverConversation => ConversationStorageAction::Delete {
                conversation_id: change.conversation_id,
            },
            _ => {
                panic!("[DataSyncManager] Unknown conversation operation")
            }
        }
    }

    pub async fn exist_user_groups(&self) -> anyhow::Result<bool> {
        self.store_engine.exist_user_groups().await
    }

    pub async fn sync_all_user_groups(&self) -> anyhow::Result<()> {
        let mut all_groups: Vec<UserGroupData> = Vec::new();
        let mut next_page_token: Option<String> = None;
        let page_size = 50;
        let mut last_version: Option<String> = None;

        loop {
            let response = self
                .ripple_api
                .get_user_groups(next_page_token.clone(), page_size)
                .await?;
            if response.code != 200 {
                anyhow::bail!(
                    "Failed to get user groups: code={}, message={}",
                    response.code,
                    response.message
                )
            }
            let groups: Vec<UserGroupData> =
                response.data.groups.into_iter().map(|g| g.into()).collect();
            all_groups.extend(groups);

            if !response.data.has_more {
                last_version = response.data.last_version;
                break;
            }
            next_page_token = response.data.next_page_token;
        }

        // Store all groups with the lastVersion
        self.store_engine.clear_all_user_groups().await?;
        if !all_groups.is_empty() {
            self.store_engine
                .apply_user_group_all(all_groups, &last_version.unwrap())
                .await?;
        }
        Ok(())
    }

    async fn sync_incremental_user_group_change(
        &self,
        last_version: Option<String>,
    ) -> anyhow::Result<UserGroupSyncData> {
        let sync_response = self.ripple_api.sync_user_groups(last_version).await?;
        if sync_response.code != 200 {
            anyhow::bail!(
                "Failed to sync user groups: code={}, message={}",
                sync_response.code,
                sync_response.message
            )
        }
        Ok(sync_response.data)
    }

    pub async fn process_user_groups_sync(
        &self,
        need_result: bool,
    ) -> anyhow::Result<Option<UserGroupSyncResult>> {
        let last_version = self.get_user_group_version().await?;
        let sync_data = self
            .sync_incremental_user_group_change(last_version)
            .await?;

        if sync_data.full_sync {
            self.sync_all_user_groups().await?;
            if need_result {
                return Ok(Some(UserGroupSyncResult::FullSync {
                    groups: self.store_engine.get_all_user_groups().await?,
                }));
            }
            return Ok(None);
        }

        if sync_data.changes.is_empty() {
            return Ok(if need_result {
                Some(UserGroupSyncResult::NoChange)
            } else {
                None
            });
        }

        let mut operations: Vec<Operation<UserGroupData, UserGroupOperation>> = Vec::new();
        for change in sync_data.changes.into_iter() {
            let id = change.group_id.clone();
            let op = change.operation.clone();
            let version = change.version.clone();
            let storage_action = self.to_user_group_storage_action(change);
            let data = self
                .store_engine
                .apply_user_group_action(storage_action, version, true)
                .await?;
            operations.push(Operation { id, op, data });
        }
        let result = process_incremental_operations(operations);

        Ok(Some(UserGroupSyncResult::IncrementalSync {
            insert: result.inserts.into_iter().map(|item| item.data).collect(),
            update: result.updates.into_iter().map(|item| item.data).collect(),
            delete: result.deletes.into_iter().map(|item| item.id).collect(),
        }))
    }

    pub async fn get_user_groups(&self) -> anyhow::Result<Vec<UserGroupData>> {
        self.store_engine.get_all_user_groups().await
    }

    async fn get_user_group_version(&self) -> anyhow::Result<Option<String>> {
        self.store_engine.get_user_group_version().await
    }

    fn to_user_group_storage_action(&self, change: UserGroupChange) -> UserGroupStorageAction {
        let op: UserGroupOperation = change.operation.into();
        match op {
            UserGroupOperation::Join => UserGroupStorageAction::Upsert(change.into()),
            UserGroupOperation::Quit => UserGroupStorageAction::Delete {
                group_id: change.group_id,
            },
            UserGroupOperation::UpdateGroupName => UserGroupStorageAction::UpdateName {
                group_id: change.group_id,
                name: change.group_name.unwrap_or_default(),
            },
            UserGroupOperation::UpdateGroupAvatar => UserGroupStorageAction::UpdateAvatar {
                group_id: change.group_id,
                avatar: change.group_avatar,
            },
            UserGroupOperation::Unknown => {
                panic!("[DataSyncManager] Unknown user group operation")
            }
        }
    }

    // ==================== Group Members Methods ====================

    pub async fn sync_all_group_members(&self, group_id: &str) -> anyhow::Result<()> {
        let mut all_members: Vec<GroupMemberData> = Vec::new();
        let mut next_page_token: Option<String> = None;
        let page_size = 50;
        let mut last_version: Option<String> = None;

        loop {
            let response = self
                .ripple_api
                .get_group_members(group_id.to_string(), next_page_token.clone(), page_size)
                .await?;
            if response.code != 200 {
                anyhow::bail!(
                    "Failed to get group members: code={}, message={}",
                    response.code,
                    response.message
                )
            }
            all_members.extend(response.data.members);
            if !response.data.has_more {
                last_version = response.data.last_version;
                break;
            }
            next_page_token = response.data.next_page_token;
        }

        // Store all members with the lastVersion
        self.store_engine.clear_group_members(group_id).await?;
        if !all_members.is_empty() {
            self.store_engine
                .apply_group_member_all(group_id, all_members, &last_version.unwrap_or_default())
                .await?;
        }
        Ok(())
    }

    async fn sync_incremental_group_member_change(
        &self,
        group_id: &str,
        last_version: Option<String>,
    ) -> anyhow::Result<GroupSyncData> {
        let sync_response = self
            .ripple_api
            .sync_group_members(group_id.to_string(), last_version)
            .await?;
        if sync_response.code != 200 {
            anyhow::bail!(
                "Failed to sync group members: code={}, message={}",
                sync_response.code,
                sync_response.message
            )
        }
        Ok(sync_response.data)
    }

    pub async fn process_group_members_sync(
        &self,
        group_id: &str,
        need_result: bool,
    ) -> anyhow::Result<Option<GroupMemberSyncResult>> {
        let last_version = self.get_group_member_version(group_id).await?;
        let sync_data = self
            .sync_incremental_group_member_change(group_id, last_version)
            .await?;

        if sync_data.full_sync {
            self.sync_all_group_members(group_id).await?;
            if need_result {
                return Ok(Some(GroupMemberSyncResult::FullSync {
                    group_id: group_id.to_string(),
                    members: self.store_engine.get_all_group_members(group_id).await?,
                }));
            }
            return Ok(None);
        }

        if sync_data.changes.is_empty() {
            return Ok(if need_result {
                Some(GroupMemberSyncResult::NoChange)
            } else {
                None
            });
        }

        // Process changes - the changes array may contain multiple groups, filter for our group
        let mut operations: Vec<Operation<GroupMemberData, GroupMemberOperation>> = Vec::new();
        for change in sync_data.changes.into_iter() {
            if change.group_id != group_id {
                continue;
            }
            let version = change.version.clone();
            for detail in change.data.into_iter() {
                let op: GroupMemberOperation = detail.operation.into();
                let user_id = detail.user_id.clone().unwrap_or_default();
                let storage_action = self.to_group_member_storage_action(&detail);
                let data = self
                    .store_engine
                    .apply_group_member_action(group_id, storage_action, version.clone(), true)
                    .await?;
                operations.push(Operation {
                    id: user_id,
                    op,
                    data,
                });
            }
        }

        let result = process_incremental_operations(operations);

        Ok(Some(GroupMemberSyncResult::IncrementalSync {
            group_id: group_id.to_string(),
            insert: result.inserts.into_iter().map(|item| item.data).collect(),
            update: result.updates.into_iter().map(|item| item.data).collect(),
            delete: result.deletes.into_iter().map(|item| item.id).collect(),
        }))
    }

    pub async fn get_group_members(&self, group_id: &str) -> anyhow::Result<Vec<GroupMemberData>> {
        self.store_engine.get_all_group_members(group_id).await
    }

    pub async fn exist_group_members(&self, group_id: &str) -> anyhow::Result<bool> {
        self.store_engine.exist_group_members(group_id).await
    }

    pub async fn store_group_members(
        &self,
        group_id: &str,
        members: Vec<GroupMemberData>,
    ) -> anyhow::Result<()> {
        // Use empty version for manually fetched members (not from sync)
        self.store_engine
            .apply_group_member_all(group_id, members, "")
            .await
    }

    async fn get_group_member_version(&self, group_id: &str) -> anyhow::Result<Option<String>> {
        self.store_engine.get_group_member_version(group_id).await
    }

    pub async fn clear_group_members(&self, group_id: &str) -> anyhow::Result<()> {
        self.store_engine.clear_group_members(group_id).await
    }

    fn to_group_member_storage_action(
        &self,
        detail: &crate::ripple_api::api_response::GroupChangeDetail,
    ) -> GroupMemberStorageAction {
        let op: GroupMemberOperation = detail.operation.into();
        match op {
            GroupMemberOperation::MemberJoin | GroupMemberOperation::CreateGroup => {
                GroupMemberStorageAction::Upsert(GroupMemberData {
                    user_id: detail.user_id.clone().unwrap(),
                    name: detail.name.clone().unwrap(),
                    avatar: detail.avatar.clone(),
                })
            }
            GroupMemberOperation::MemberQuit | GroupMemberOperation::DeleteGroup => {
                GroupMemberStorageAction::Delete {
                    user_id: detail.user_id.clone().unwrap(),
                }
            }
            GroupMemberOperation::MemberUpdateName => GroupMemberStorageAction::UpdateName {
                user_id: detail.user_id.clone().unwrap(),
                name: detail.name.clone().unwrap(),
            },
            GroupMemberOperation::MemberUpdateAvatar => GroupMemberStorageAction::UpdateAvatar {
                user_id: detail.user_id.clone().unwrap(),
                avatar: detail.avatar.clone(),
            },
            GroupMemberOperation::Unknown => {
                panic!("[DataSyncManager] Unknown group member operation")
            }
        }
    }
}
