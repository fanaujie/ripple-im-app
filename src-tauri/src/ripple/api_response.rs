use serde::Deserialize;

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
