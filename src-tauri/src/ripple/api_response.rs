use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AvatarUploadData {
    #[serde(rename = "avatarUrl")]
    pub avatar_url: String,
}

#[derive(Deserialize)]
pub struct AvatarUploadResponse {
    pub code: i64,
    pub message: String,
    pub data: AvatarUploadData,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserProfileData {
    #[serde(rename = "userId")]
    pub user_id: i64,
    #[serde(rename = "nickName")]
    pub nick_name: String,
    pub avatar: Option<String>,
}

#[derive(Deserialize)]
pub struct UserProfileResponse {
    pub code: i64,
    pub message: String,
    pub data: UserProfileData,
}

#[derive(Deserialize)]
pub struct CommonResponse {
    pub code: i64,
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct UpdateNickNameRequest {
    #[serde(rename = "nickName")]
    pub nick_name: String,
}
