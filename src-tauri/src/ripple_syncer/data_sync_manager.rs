use crate::ripple_api::api_response::{RelationUser, UserProfileData};
use crate::ripple_api::RippleApi;
use crate::ripple_syncer::ui_event::{BLOCKED_FLAG, FRIEND_FLAG, HIDDEN_FLAG};
use crate::store_engine::store_engine::RippleStorage;
use crate::store_engine::StoreEngine;

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

    pub async fn sync_user_profile(&self) -> anyhow::Result<UserProfileData> {
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

        Ok(profile_response.data)
    }

    pub async fn sync_all_relations(&self) -> anyhow::Result<Vec<RelationUser>> {
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

        let version = relation_version
            .data
            .latest_version
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No latest version available"))?;

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
            .apply_relation_all(&all_users, &version)
            .await?;
        Ok(all_users)
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

    pub async fn get_cached_profile(&self) -> anyhow::Result<Option<UserProfileData>> {
        self.store_engine.get_user_profile().await
    }

    pub async fn get_cached_relations(
        &self,
    ) -> anyhow::Result<(Vec<RelationUser>, Vec<RelationUser>)> {
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

    pub fn store_engine(&self) -> &S {
        &self.store_engine
    }
}
