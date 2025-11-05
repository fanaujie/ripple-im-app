use crate::ripple_api::api_response::{RelationUser, UserProfileData};
use crate::ripple_syncer::relation_operation::RelationAction;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

#[trait_variant::make(RippleStorage: Send)]
pub trait StoreEngine: Sync + Clone + 'static {
    async fn exists_token(&self) -> anyhow::Result<bool>;

    async fn get_device_id(&self) -> anyhow::Result<Option<Uuid>>;
    async fn save_device_id(&self, device_id: &Uuid) -> anyhow::Result<()>;
    async fn get_token(&self) -> anyhow::Result<Token>;
    async fn save_token(&self, access_token: &str, refresh_token: &str) -> anyhow::Result<()>;
    async fn get_user_profile(&self) -> anyhow::Result<Option<UserProfileData>>;
    async fn save_user_profile(&self, profile: &UserProfileData) -> anyhow::Result<()>;

    async fn apply_relation_all(
        &self,
        action: &Vec<RelationUser>,
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
}

#[derive(Clone)]
pub struct MemoryStore {
    inner: Arc<tokio::sync::Mutex<InnerStore>>,
}

struct InnerStore {
    access_token: Option<String>,
    refresh_token: Option<String>,
    uuid: Option<Uuid>,
    // Global data cache
    user_profile: Option<UserProfileData>,
    // Unified relations storage (user_id -> StoredRelationUser)
    relations: HashMap<String, RelationUser>,
    relation_version: Option<String>,
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
            })),
        }
    }
}

impl RippleStorage for MemoryStore {
    async fn exists_token(&self) -> anyhow::Result<bool> {
        // always return false for in-memory store
        Ok(false)
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

    async fn get_token(&self) -> anyhow::Result<Token> {
        let inner = self.inner.lock().await;
        if let Some(token) = &inner.access_token {
            Ok(Token {
                access_token: inner.access_token.clone().unwrap(),
                refresh_token: inner.refresh_token.clone().unwrap(),
            })
        } else {
            Err(anyhow::anyhow!("No token found"))
        }
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
        action: &Vec<RelationUser>,
        last_version: &str,
    ) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.relations.clear();
        for relation in action {
            inner
                .relations
                .insert(relation.user_id.clone(), relation.clone());
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
}
