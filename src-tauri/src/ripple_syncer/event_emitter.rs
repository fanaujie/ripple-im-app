use crate::ripple_api::api_response::{RelationUser, UserProfileData};

pub trait EventEmitter: Send + Sync + Clone + 'static {
    fn emit_user_profile_updated(&self, profile: UserProfileData) -> anyhow::Result<()>;
    fn emit_relation_updated(&self, action: i32, user: Option<RelationUser>) -> anyhow::Result<()>;
    fn emit_relations_cleared(&self) -> anyhow::Result<()>;
}
