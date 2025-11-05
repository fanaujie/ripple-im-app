use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserProfileData {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "nickName")]
    pub nick_name: String,
    pub avatar: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserProfileResponse {
    pub code: i64,
    pub message: String,
    pub data: UserProfileData,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CommonResponse {
    pub code: i64,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateNickNameRequest {
    #[serde(rename = "nickName")]
    pub nick_name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateFriendDisplayNameRequest {
    #[serde(rename = "remarkName")]
    pub remark_name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AddFriendRequest {
    #[serde(rename = "targetUserId")]
    pub target_user_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BlockUserRequest {
    #[serde(rename = "targetUserId")]
    pub target_user_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationUser {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "nickName")]
    pub nick_name: String,
    pub avatar: Option<String>,
    #[serde(rename = "remarkName")]
    pub remark_name: String,
    #[serde(rename = "relationFlags")]
    pub relation_flags: i32,
}

impl From<&RelationChange> for RelationUser {
    fn from(change: &RelationChange) -> Self {
        RelationUser {
            user_id: change.user_id.clone(),
            nick_name: change.nick_name.clone().unwrap_or_default(),
            avatar: change.avatar.clone(),
            remark_name: change.remark_name.clone().unwrap_or_default(),
            relation_flags: change.relation_flags,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GlobalInitData {
    pub user_profile: UserProfileData,
    pub friends: Vec<RelationUser>,
    #[serde(rename = "blockedUsers")]
    pub blocked_users: Vec<RelationUser>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationsPageData {
    pub users: Vec<RelationUser>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationsPageResponse {
    pub code: i64,
    pub message: String,
    pub data: RelationsPageData,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationChange {
    pub version: String,
    pub operation: u64,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "nickName")]
    pub nick_name: Option<String>,
    pub avatar: Option<String>,
    #[serde(rename = "remarkName")]
    pub remark_name: Option<String>,
    #[serde(rename = "relationFlags")]
    pub relation_flags: i32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationsSyncData {
    #[serde(rename = "fullSync")]
    pub full_sync: bool,
    #[serde(rename = "latestVersion")]
    pub latest_version: Option<String>,
    pub changes: Vec<RelationChange>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationsSyncResponse {
    pub code: i64,
    pub message: String,
    pub data: RelationsSyncData,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationVersionData {
    #[serde(rename = "latestVersion")]
    pub latest_version: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationVersionResponse {
    pub code: i64,
    pub message: String,
    pub data: RelationVersionData,
}
