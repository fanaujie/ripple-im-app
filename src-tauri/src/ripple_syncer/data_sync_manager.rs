use crate::ripple_api::api_response::{
    CommonResponse, ConversationSyncData, ReadMessagesData, RelationUser, UserProfileData,
};
use crate::ripple_api::RippleApi;
use crate::ripple_syncer::conversation_operation::ConversationAction;
use crate::ripple_syncer::relation_operation::RelationAction;
use crate::ripple_syncer::ui_event::{BLOCKED_FLAG, FRIEND_FLAG, HIDDEN_FLAG};
use crate::store_engine::store_engine::{
    RippleStorage, StorageConversationData, StorageMessageData,
};

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

    pub async fn get_profile(&self) -> anyhow::Result<Option<UserProfileData>> {
        self.store_engine.get_user_profile().await
    }

    pub async fn get_relations(&self) -> anyhow::Result<(Vec<RelationUser>, Vec<RelationUser>)> {
        let mut friends: Vec<RelationUser> = Vec::new();
        let mut blocked_users: Vec<RelationUser> = Vec::new();
        let relations = self.store_engine.get_all_relations().await?;
        for relation in relations {
            if (relation.relation_flags & FRIEND_FLAG) != 0
                && relation.relation_flags & BLOCKED_FLAG == 0
            {
                friends.push(relation);
            } else if (relation.relation_flags & BLOCKED_FLAG) != 0
                && (relation.relation_flags & HIDDEN_FLAG) == 0
            {
                blocked_users.push(relation);
            }
        }
        Ok((friends, blocked_users))
    }

    pub async fn sync_all_conversations(&self) -> anyhow::Result<()> {
        let mut all_conversations = Vec::new();
        let mut next_page_token: Option<String> = None;
        let page_size = 50; // Default page size

        // Get current version
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
    ) -> anyhow::Result<()> {
        self.store_engine
            .apply_relation_action(action, version)
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
        action: ConversationAction,
        version: String,
        user_id: i64,
    ) -> anyhow::Result<()> {
        self.store_engine
            .apply_conversation_action(action, version, user_id)
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
        println!(
            "[DataSyncManager] read_latest_messages called: conversation_id={}, read_size={}",
            conversation_id, read_size
        );

        let storage_messages = self
            .store_engine
            .get_latest_messages(&conversation_id, read_size)
            .await?;

        println!(
            "[DataSyncManager] Retrieved {} latest messages from storage for conversation {}",
            storage_messages.len(),
            conversation_id
        );

        if storage_messages.len() >= read_size as usize {
            let messages: Vec<_> = storage_messages.into_iter().map(|msg| msg.into()).collect();
            println!("[DataSyncManager] Returning latest messages from storage (sufficient count)");
            return Ok(ReadMessagesData { messages });
        }

        // If we don't have enough messages in storage, fall back to API
        println!(
            "[DataSyncManager] Not enough messages in storage, fetching from API for conversation {}",
            conversation_id
        );
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

        println!(
            "[DataSyncManager] API returned {} messages, storing them",
            api_response.data.messages.len()
        );
        for message_item in &api_response.data.messages {
            self.store_engine.store_message(message_item.into()).await?;
        }
        println!("[DataSyncManager] Latest messages stored successfully");
        Ok(api_response.data)
    }

    pub async fn read_messages_before(
        &self,
        conversation_id: String,
        before_message_id: String,
        read_size: u32,
    ) -> anyhow::Result<ReadMessagesData> {
        println!(
            "[DataSyncManager] read_messages_before called: conversation_id={}, before_message_id={}, read_size={}",
            conversation_id, before_message_id, read_size
        );

        let before_id = before_message_id.parse::<i64>().unwrap_or(0);
        if before_id == 0 {
            println!("[DataSyncManager] Invalid before_message_id, returning empty");
            return Ok(ReadMessagesData {
                messages: Vec::new(),
            });
        }

        let storage_messages = self
            .store_engine
            .get_messages_before(&conversation_id, before_id, read_size)
            .await?;

        println!(
            "[DataSyncManager] Retrieved {} messages before id {} from storage",
            storage_messages.len(),
            before_id
        );

        if storage_messages.len() >= read_size as usize {
            let messages: Vec<_> = storage_messages.into_iter().map(|msg| msg.into()).collect();
            println!("[DataSyncManager] Returning messages from storage (sufficient count)");
            return Ok(ReadMessagesData { messages });
        }

        // If we don't have enough messages in storage, fetch from API
        println!("[DataSyncManager] Not enough messages in storage, fetching from API");
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

        println!(
            "[DataSyncManager] API returned {} messages, storing them",
            api_response.data.messages.len()
        );
        for message_item in &api_response.data.messages {
            self.store_engine.store_message(message_item.into()).await?;
        }

        // After storing, retry getting messages before the specified ID
        let storage_messages = self
            .store_engine
            .get_messages_before(&conversation_id, before_id, read_size)
            .await?;

        let messages: Vec<_> = storage_messages.into_iter().map(|msg| msg.into()).collect();
        println!(
            "[DataSyncManager] Returning {} messages after API fetch and store",
            messages.len()
        );
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
}
