pub struct ApiPaths {
    // Pre-built full URLs
    pub upload_avatar: String,
    pub profile: String,
    pub profile_nickname: String,
    pub profile_portrait: String,
}

impl ApiPaths {
    pub fn new(upload_gateway_url: &str, api_gateway_url: &str) -> Self {
        Self {
            upload_avatar: format!("{}/api/upload/avatar", upload_gateway_url),
            profile: format!("{}/api/profile", api_gateway_url),
            profile_nickname: format!("{}/api/profile/nickname", api_gateway_url),
            profile_portrait: format!("{}/api/profile/portrait", api_gateway_url),
        }
    }
}