#[derive(Clone)]
pub struct ApiPaths {
    // Pre-built full URLs for upload gateway
    pub upload_avatar: String,
    pub upload_group_avatar_base: String,
    // Attachment upload paths
    pub attachment_initiate: String,
    pub attachment_single: String,
    pub attachment_chunk: String,
    pub attachment_complete: String,
    pub attachment_abort: String,
    // User profile paths
    pub my_profile: String,
    pub user_profile: String,
    pub my_avatar: String,
    // Relation paths
    pub relations: String,
    pub relations_sync: String,
    pub friends: String,
    pub blocked_users: String,
    // Conversation paths
    pub conversations: String,
    pub conversations_sync: String,
    pub conversations_summary: String,
    // Group paths
    pub groups: String,
    // User groups paths
    pub user_groups: String,
    pub user_groups_sync: String,
}

impl ApiPaths {
    pub fn new(upload_gateway_url: &str, api_gateway_url: &str) -> Self {
        Self {
            upload_avatar: format!("{}/api/upload/avatar", upload_gateway_url),
            upload_group_avatar_base: format!("{}/api/upload/groups", upload_gateway_url),
            // Attachment upload paths
            attachment_initiate: format!("{}/api/upload/attachment/initiate", upload_gateway_url),
            attachment_single: format!("{}/api/upload/attachment/single", upload_gateway_url),
            attachment_chunk: format!("{}/api/upload/attachment/chunk", upload_gateway_url),
            attachment_complete: format!(
                "{}/api/upload/attachment/chunk/complete",
                upload_gateway_url
            ),
            attachment_abort: format!("{}/api/upload/attachment/abort", upload_gateway_url),
            // User profile paths
            my_profile: format!("{}/api/users/me/profile", api_gateway_url),
            user_profile: format!("{}/api/users", api_gateway_url),
            my_avatar: format!("{}/api/users/me/avatar", api_gateway_url),
            // Relation paths
            relations: format!("{}/api/users/me/relations", api_gateway_url),
            relations_sync: format!("{}/api/users/me/relations/sync", api_gateway_url),
            friends: format!("{}/api/users/me/friends", api_gateway_url),
            blocked_users: format!("{}/api/users/me/blocked-users", api_gateway_url),
            // Conversation paths
            conversations: format!("{}/api/users/me/conversations", api_gateway_url),
            conversations_sync: format!("{}/api/users/me/conversations/sync", api_gateway_url),
            conversations_summary: format!(
                "{}/api/users/me/conversations/summary",
                api_gateway_url
            ),
            // Group paths
            groups: format!("{}/api/groups", api_gateway_url),
            // User groups paths
            user_groups: format!("{}/api/users/me/groups", api_gateway_url),
            user_groups_sync: format!("{}/api/users/me/groups/sync", api_gateway_url),
        }
    }
}
