#[derive(Clone)]
pub struct ApiPaths {
    // Pre-built full URLs
    pub upload_avatar: String,
    pub profile: String,
    pub profile_nickname: String,
    pub profile_avatar: String,
    // Relation paths
    pub relations: String,
    pub relations_sync: String,
    pub relations_version: String,
    pub friends: String,
    pub blocked_users: String,
}

impl ApiPaths {
    pub fn new(upload_gateway_url: &str, api_gateway_url: &str) -> Self {
        Self {
            upload_avatar: format!("{}/api/upload/avatar", upload_gateway_url),
            profile: format!("{}/api/profile", api_gateway_url),
            profile_nickname: format!("{}/api/profile/nickname", api_gateway_url),
            profile_avatar: format!("{}/api/profile/avatar", api_gateway_url),
            relations: format!("{}/api/relations", api_gateway_url),
            relations_sync: format!("{}/api/relations/sync", api_gateway_url),
            relations_version: format!("{}/api/relations/version", api_gateway_url),
            friends: format!("{}/api/relations/friends", api_gateway_url),
            blocked_users: format!("{}/api/relations/blocked-users", api_gateway_url),
        }
    }
}