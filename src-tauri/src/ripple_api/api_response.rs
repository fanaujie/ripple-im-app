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
    pub remark_name: Option<String>,
    #[serde(rename = "relationFlags")]
    pub relation_flags: i32,
}

impl From<&RelationChange> for RelationUser {
    fn from(change: &RelationChange) -> Self {
        RelationUser {
            user_id: change.user_id.clone(),
            nick_name: change.nick_name.clone().unwrap_or_default(),
            avatar: change.avatar.clone(),
            remark_name: change.remark_name.clone(),
            relation_flags: change.relation_flags,
        }
    }
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationItem {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "peerId")]
    pub peer_id: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "lastMessageId")]
    pub last_message_id: String,
    #[serde(rename = "lastMessage")]
    pub last_message: String,
    #[serde(rename = "lastMessageTimestamp")]
    pub last_message_timestamp: i64,
    #[serde(rename = "lastReadMessageId")]
    pub last_read_message_id: Option<String>,
    #[serde(rename = "unreadCount")]
    pub unread_count: i64,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "avatar")]
    pub avatar: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationsData {
    pub conversations: Vec<ConversationItem>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationsResponse {
    pub code: i64,
    pub message: String,
    pub data: ConversationsData,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationChange {
    pub version: String,
    pub operation: i32,
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "peerId")]
    pub peer_id: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "lastMessageId")]
    pub last_message_id: Option<String>,
    #[serde(rename = "lastMessage")]
    pub last_message: Option<String>,
    #[serde(rename = "lastMessageTimestamp")]
    pub last_message_timestamp: Option<i64>,
    #[serde(rename = "lastReadMessageId")]
    pub last_read_message_id: Option<String>,
    #[serde(rename = "name")]
    pub name: Option<String>,
    #[serde(rename = "avatar")]
    pub avatar: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationSyncData {
    #[serde(rename = "fullSync")]
    pub full_sync: bool,
    #[serde(rename = "latestVersion")]
    pub latest_version: Option<String>,
    pub changes: Vec<ConversationChange>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationSyncResponse {
    pub code: i64,
    pub message: String,
    pub data: ConversationSyncData,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationVersionData {
    #[serde(rename = "latestVersion")]
    pub latest_version: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationVersionResponse {
    pub code: i64,
    pub message: String,
    pub data: ConversationVersionData,
}

// ==================== Message Types ====================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SendMessageRequest {
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "receiverId")]
    pub receiver_id: String,
    #[serde(rename = "textContent", skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
    #[serde(rename = "fileUrl", skip_serializing_if = "Option::is_none")]
    pub file_url: Option<String>,
    #[serde(rename = "fileName", skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MessageResponseData {
    #[serde(rename = "messageId")]
    pub message_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MessageResponse {
    pub code: i64,
    pub message: String,
    pub data: Option<MessageResponseData>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MessageItem {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "receiverId")]
    pub receiver_id: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "sendTimestamp")]
    pub send_timestamp: i64,
    #[serde(rename = "textContent")]
    pub text_content: Option<String>,
    #[serde(rename = "fileUrl")]
    pub file_url: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ReadMessagesData {
    pub messages: Vec<MessageItem>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ReadMessagesResponse {
    pub code: i64,
    pub message: String,
    pub data: ReadMessagesData,
}

impl ReadMessagesResponse {
    pub fn error(code: i64, message: &str) -> Self {
        ReadMessagesResponse {
            code,
            message: message.to_string(),
            data: ReadMessagesData {
                messages: Vec::new(),
            },
        }
    }
}
