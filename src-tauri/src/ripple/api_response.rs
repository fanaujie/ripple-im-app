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

#[derive(Deserialize)]
pub struct UserProfileData {
    pub account: String,
    #[serde(rename = "nickName")]
    pub nick_name: String,
    #[serde(rename = "userPortrait")]
    pub user_portrait: Option<String>,
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
