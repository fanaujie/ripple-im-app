use crate::ripple_syncer::incremental_operations::{Categorized, OpCategory};
use ripple_proto::ripple_pb::{push_message_request, send_message_req, PushMessageRequest};
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
    pub code: i32,
    pub message: String,
    pub data: UserProfileData,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CommonResponse {
    pub code: i32,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UploadImageData {
    #[serde(rename = "avatarUrl")]
    pub url: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UploadImageResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<UploadImageData>,
}

// ==================== Profile Request Types ====================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateProfileRequest {
    #[serde(rename = "nickname", skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
}

// ==================== Friend Request Types ====================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AddFriendRequest {
    #[serde(rename = "targetUserId")]
    pub target_user_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateFriendRequest {
    #[serde(rename = "remarkName", skip_serializing_if = "Option::is_none")]
    pub remark_name: Option<String>,
}

// ==================== Block User Request Types ====================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BlockUserRequest {
    #[serde(rename = "targetUserId")]
    pub target_user_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateBlockedUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
}

// ==================== Relation Types ====================

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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationUsers {
    pub users: Vec<RelationUser>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationsPageData {
    pub users: Vec<RelationUser>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
    #[serde(rename = "lastVersion")]
    pub last_version: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationsPageResponse {
    pub code: i32,
    pub message: String,
    pub data: RelationsPageData,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(from = "i32", into = "i32")]
pub enum RelationOperation {
    AddFriend = 1,
    UpdateFriendRemarkName = 2,
    DeleteFriend = 3,
    AddBlock = 4,
    DeleteBlock = 5,
    UnblockRestoreFriend = 6,
    HideBlock = 7,
    UpdateFriendNickName = 8,
    UpdateFriendAvatar = 9,
    BlockStranger = 10,
    SyncFriendInfo = 11,
    Unknown,
}

impl From<i32> for RelationOperation {
    fn from(value: i32) -> Self {
        match value {
            1 => RelationOperation::AddFriend,
            2 => RelationOperation::UpdateFriendRemarkName,
            3 => RelationOperation::DeleteFriend,
            4 => RelationOperation::AddBlock,
            5 => RelationOperation::DeleteBlock,
            6 => RelationOperation::UnblockRestoreFriend,
            7 => RelationOperation::HideBlock,
            8 => RelationOperation::UpdateFriendNickName,
            9 => RelationOperation::UpdateFriendAvatar,
            10 => RelationOperation::BlockStranger,
            11 => RelationOperation::SyncFriendInfo,
            _ => RelationOperation::Unknown,
        }
    }
}

impl From<RelationOperation> for i32 {
    fn from(op: RelationOperation) -> Self {
        op as i32
    }
}

impl Categorized for RelationOperation {
    fn category(&self) -> OpCategory {
        match self {
            RelationOperation::AddFriend | RelationOperation::BlockStranger => OpCategory::Add,
            RelationOperation::AddBlock
            | RelationOperation::UnblockRestoreFriend
            | RelationOperation::SyncFriendInfo
            | RelationOperation::UpdateFriendRemarkName
            | RelationOperation::UpdateFriendNickName
            | RelationOperation::UpdateFriendAvatar
            | RelationOperation::HideBlock => OpCategory::Update,
            RelationOperation::DeleteFriend | RelationOperation::DeleteBlock => OpCategory::Delete,
            RelationOperation::Unknown => OpCategory::Update,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RelationChange {
    pub version: String,
    pub operation: RelationOperation,
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

impl From<RelationChange> for RelationUser {
    fn from(value: RelationChange) -> Self {
        RelationUser {
            user_id: value.user_id,
            nick_name: value.nick_name.unwrap_or_default(),
            avatar: value.avatar,
            remark_name: value.remark_name,
            relation_flags: value.relation_flags,
        }
    }
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
    pub code: i32,
    pub message: String,
    pub data: RelationsSyncData,
}

// ==================== Conversation Types ====================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationItem {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "peerId")]
    pub peer_id: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "lastMessageId")]
    pub last_message_id: Option<String>,
    #[serde(rename = "lastReadMessageId")]
    pub last_read_message_id: Option<String>,
    #[serde(rename = "unreadCount")]
    pub unread_count: i64,
    #[serde(rename = "lastMessageText")]
    pub last_message_text: Option<String>,
    #[serde(rename = "lastMessageTimestamp")]
    pub last_message_timestamp: Option<i64>,
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
    #[serde(rename = "lastVersion")]
    pub last_version: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationsResponse {
    pub code: i32,
    pub message: String,
    pub data: ConversationsData,
}

// ==================== Conversation Summary Types ====================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationSummaryRequest {
    #[serde(rename = "conversationIds")]
    pub conversation_ids: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationSummary {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "unreadCount")]
    pub unread_count: i64,
    #[serde(rename = "lastMessageText")]
    pub last_message_text: Option<String>,
    #[serde(rename = "lastMessageTimestamp")]
    pub last_message_timestamp: i64,
    #[serde(rename = "lastMessageId")]
    pub last_message_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationSummariesData {
    pub summaries: Vec<ConversationSummary>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationSummariesResponse {
    pub code: i32,
    pub message: String,
    pub data: ConversationSummariesData,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(from = "i32", into = "i32")]
pub enum ConversationOperation {
    CreateConversation = 1,
    UpdateLastReadMessageId = 2,
    UpdateConversationName = 3,
    UpdateConversationAvatar = 4,
    UpdateConversationNameAvatar = 5,
    RemoverConversation = 6,
    Unknown,
}

impl From<i32> for ConversationOperation {
    fn from(value: i32) -> Self {
        match value {
            1 => ConversationOperation::CreateConversation,
            2 => ConversationOperation::UpdateLastReadMessageId,
            3 => ConversationOperation::UpdateConversationName,
            4 => ConversationOperation::UpdateConversationAvatar,
            5 => ConversationOperation::UpdateConversationNameAvatar,
            6 => ConversationOperation::RemoverConversation,
            _ => ConversationOperation::Unknown,
        }
    }
}

impl From<ConversationOperation> for i32 {
    fn from(op: ConversationOperation) -> Self {
        op as i32
    }
}

impl Categorized for ConversationOperation {
    fn category(&self) -> OpCategory {
        match self {
            ConversationOperation::CreateConversation => OpCategory::Add,
            ConversationOperation::UpdateLastReadMessageId
            | ConversationOperation::UpdateConversationName
            | ConversationOperation::UpdateConversationAvatar
            | ConversationOperation::UpdateConversationNameAvatar => OpCategory::Update,
            ConversationOperation::RemoverConversation => OpCategory::Delete,
            ConversationOperation::Unknown => OpCategory::Update,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConversationChange {
    pub version: String,
    pub operation: ConversationOperation,
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "peerId")]
    pub peer_id: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
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
    pub code: i32,
    pub message: String,
    pub data: ConversationSyncData,
}

// ==================== Read Position Request Types ====================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateReadPositionRequest {
    #[serde(rename = "messageId")]
    pub message_id: String,
}

// ==================== Message Types ====================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SendMessageRequest {
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "receiverId", skip_serializing_if = "Option::is_none")]
    pub receiver_id: Option<String>,
    #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
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
    pub code: i32,
    pub message: String,
    pub data: Option<MessageResponseData>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
#[serde(from = "i32", into = "i32")]
pub enum MessageItemType {
    Text = 1,
    Command = 2,
    Unknown,
}

impl From<i32> for MessageItemType {
    fn from(value: i32) -> Self {
        match value {
            1 => MessageItemType::Text,
            2 => MessageItemType::Command,
            _ => MessageItemType::Unknown,
        }
    }
}

impl From<MessageItemType> for i32 {
    fn from(op: MessageItemType) -> Self {
        op as i32
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(from = "i32", into = "i32")]
pub enum MessageCommandType {
    Empty = 0,
    GroupJoin = 1,
    GroupQuit = 2,
    InfoUpdate = 3,
    Unknown,
}

impl From<i32> for MessageCommandType {
    fn from(value: i32) -> Self {
        match value {
            0 => MessageCommandType::Empty,
            1 => MessageCommandType::GroupJoin,
            2 => MessageCommandType::GroupQuit,
            3 => MessageCommandType::InfoUpdate,
            _ => MessageCommandType::Unknown,
        }
    }
}

impl From<MessageCommandType> for i32 {
    fn from(op: MessageCommandType) -> Self {
        op as i32
    }
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
    pub send_timestamp: String,
    #[serde(rename = "messageType")]
    pub message_type: MessageItemType,
    #[serde(rename = "text")]
    pub text: Option<String>,
    #[serde(rename = "fileUrl")]
    pub file_url: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "commandType")]
    pub command_type: MessageCommandType,
    #[serde(rename = "commandData")]
    pub command_data: Option<String>,
}

impl From<&PushMessageRequest> for MessageItem {
    fn from(req: &PushMessageRequest) -> Self {
        match req.payload.as_ref() {
            Some(push_message_request::Payload::MessagePayload(msg_payload)) => {
                let message_data = msg_payload
                    .message_data
                    .as_ref()
                    .expect("MessagePayload must have message_data");

                match &message_data.message {
                    Some(send_message_req::Message::SingleMessageContent(msg_content)) => {
                        MessageItem {
                            conversation_id: message_data.conversation_id.clone(),
                            message_id: message_data.message_id.to_string(),
                            sender_id: message_data.sender_id.to_string(),
                            receiver_id: (message_data.receiver_id != 0)
                                .then(|| message_data.receiver_id.to_string()),
                            group_id: (message_data.group_id != 0)
                                .then(|| message_data.group_id.to_string()),
                            send_timestamp: message_data.send_timestamp.to_string(),
                            message_type: MessageItemType::Text,
                            text: Some(msg_content.text.clone()),
                            file_url: Some(msg_content.file_url.clone()),
                            file_name: Some(msg_content.file_name.clone()),
                            command_type: MessageCommandType::Empty,
                            command_data: None,
                        }
                    }
                    Some(send_message_req::Message::GroupCommandMessageContent(cmd_content)) => {
                        MessageItem {
                            conversation_id: message_data.conversation_id.clone(),
                            message_id: message_data.message_id.to_string(),
                            sender_id: message_data.sender_id.to_string(),
                            receiver_id: (message_data.receiver_id != 0)
                                .then(|| message_data.receiver_id.to_string()),
                            group_id: (message_data.group_id != 0)
                                .then(|| message_data.group_id.to_string()),
                            send_timestamp: message_data.send_timestamp.to_string(),
                            message_type: MessageItemType::Command,
                            text: None,
                            file_url: None,
                            file_name: None,
                            command_type: cmd_content.command_type.into(),
                            command_data: Some(cmd_content.text.clone()),
                        }
                    }
                    None => panic!("SendMessageReq must have a message variant"),
                }
            }
            _ => panic!("PushMessageRequest must have MessagePayload for message storage"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ReadMessagesData {
    pub messages: Vec<MessageItem>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ReadMessagesResponse {
    pub code: i32,
    pub message: String,
    pub data: ReadMessagesData,
}

// ==================== Group Types ====================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CreateGroupRequest {
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "groupName")]
    pub group_name: String,
    #[serde(rename = "groupAvatar", skip_serializing_if = "Option::is_none")]
    pub group_avatar: Option<String>,
    #[serde(rename = "memberIds")]
    pub member_ids: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GroupData {
    #[serde(rename = "groupId")]
    pub group_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CreateGroupResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<GroupData>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateGroupRequest {
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InviteGroupMemberRequest {
    #[serde(rename = "senderId")]
    pub sender_id: String,
    #[serde(rename = "newMemberIds")]
    pub new_member_ids: Vec<String>,
    #[serde(rename = "groupName")]
    pub group_name: String,
    #[serde(rename = "groupAvatar")]
    pub group_avatar: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GroupMemberData {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub name: String,
    pub avatar: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GroupMembersPageData {
    pub members: Vec<GroupMemberData>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
    #[serde(rename = "lastVersion")]
    pub last_version: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetGroupMembersResponse {
    pub code: i32,
    pub message: String,
    pub data: GroupMembersPageData,
}

// ==================== User Groups Types ====================

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(from = "i32", into = "i32")]
pub enum UserGroupOperation {
    Join = 1,
    Quit = 2,
    UpdateGroupName = 3,
    UpdateGroupAvatar = 4,
    Unknown,
}

impl From<i32> for UserGroupOperation {
    fn from(value: i32) -> Self {
        match value {
            1 => UserGroupOperation::Join,
            2 => UserGroupOperation::Quit,
            3 => UserGroupOperation::UpdateGroupName,
            4 => UserGroupOperation::UpdateGroupAvatar,
            _ => UserGroupOperation::Unknown,
        }
    }
}

impl From<UserGroupOperation> for i32 {
    fn from(op: UserGroupOperation) -> Self {
        op as i32
    }
}

impl Categorized for UserGroupOperation {
    fn category(&self) -> OpCategory {
        match self {
            UserGroupOperation::Join => OpCategory::Add,
            UserGroupOperation::Quit => OpCategory::Delete,
            UserGroupOperation::UpdateGroupName | UserGroupOperation::UpdateGroupAvatar => {
                OpCategory::Update
            }
            UserGroupOperation::Unknown => OpCategory::Update,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserGroupData {
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "groupName")]
    pub group_name: String,
    #[serde(rename = "groupAvatar")]
    pub group_avatar: Option<String>,
}

impl From<UserGroupChange> for UserGroupData {
    fn from(value: UserGroupChange) -> Self {
        UserGroupData {
            group_id: value.group_id,
            group_name: value.group_name.unwrap_or_default(),
            group_avatar: value.group_avatar,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserGroupsPageData {
    pub groups: Vec<UserGroupData>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
    #[serde(rename = "lastVersion")]
    pub last_version: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetUserGroupsResponse {
    pub code: i32,
    pub message: String,
    pub data: UserGroupsPageData,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserGroupChange {
    pub version: String,
    pub operation: UserGroupOperation,
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "groupName")]
    pub group_name: Option<String>,
    #[serde(rename = "groupAvatar")]
    pub group_avatar: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserGroupSyncData {
    #[serde(rename = "fullSync")]
    pub full_sync: bool,
    #[serde(rename = "latestVersion")]
    pub latest_version: Option<String>,
    pub changes: Vec<UserGroupChange>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserGroupSyncResponse {
    pub code: i32,
    pub message: String,
    pub data: UserGroupSyncData,
}

// ==================== Group Members Sync Types ====================

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(from = "i32", into = "i32")]
pub enum GroupMemberOperation {
    CreateGroup = 1,
    DeleteGroup = 2,
    MemberJoin = 3,
    MemberQuit = 4,
    MemberUpdateName = 5,
    MemberUpdateAvatar = 6,
    Unknown,
}

impl From<i32> for GroupMemberOperation {
    fn from(value: i32) -> Self {
        match value {
            1 => GroupMemberOperation::CreateGroup,
            2 => GroupMemberOperation::DeleteGroup,
            3 => GroupMemberOperation::MemberJoin,
            4 => GroupMemberOperation::MemberQuit,
            5 => GroupMemberOperation::MemberUpdateName,
            6 => GroupMemberOperation::MemberUpdateAvatar,
            _ => GroupMemberOperation::Unknown,
        }
    }
}

impl From<GroupMemberOperation> for i32 {
    fn from(op: GroupMemberOperation) -> Self {
        op as i32
    }
}

impl Categorized for GroupMemberOperation {
    fn category(&self) -> OpCategory {
        match self {
            GroupMemberOperation::CreateGroup | GroupMemberOperation::MemberJoin => OpCategory::Add,
            GroupMemberOperation::DeleteGroup | GroupMemberOperation::MemberQuit => {
                OpCategory::Delete
            }
            GroupMemberOperation::MemberUpdateName | GroupMemberOperation::MemberUpdateAvatar => {
                OpCategory::Update
            }
            GroupMemberOperation::Unknown => OpCategory::Update,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GroupChangeDetail {
    pub operation: GroupMemberOperation,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub name: Option<String>,
    pub avatar: Option<String>,
    #[serde(rename = "groupName")]
    pub group_name: Option<String>,
    #[serde(rename = "groupAvatar")]
    pub group_avatar: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GroupChange {
    #[serde(rename = "groupId")]
    pub group_id: String,
    pub version: String,
    pub data: Vec<GroupChangeDetail>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GroupSyncData {
    #[serde(rename = "fullSync")]
    pub full_sync: bool,
    #[serde(rename = "latestVersion")]
    pub latest_version: Option<String>,
    pub changes: Vec<GroupChange>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GroupSyncResponse {
    pub code: i32,
    pub message: String,
    pub data: GroupSyncData,
}

// ==================== Attachment Upload Types ====================

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InitiateUploadRequest {
    #[serde(rename = "fileSize")]
    pub file_size: i64,
    #[serde(rename = "fileSha256")]
    pub file_sha256: String,
    #[serde(rename = "originalFilename")]
    pub original_filename: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InitiateUploadData {
    #[serde(rename = "uploadMode")]
    pub upload_mode: i32,
    #[serde(rename = "chunkSize")]
    pub chunk_size: Option<i64>,
    #[serde(rename = "totalChunks")]
    pub total_chunks: Option<i32>,
    #[serde(rename = "startChunkNumber")]
    pub start_chunk_number: Option<i32>,
    #[serde(rename = "objectName")]
    pub object_name: Option<String>,
    #[serde(rename = "fileUrl")]
    pub file_url: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InitiateUploadResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<InitiateUploadData>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CompleteUploadData {
    #[serde(rename = "fileUrl")]
    pub file_url: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SingleUploadResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<CompleteUploadData>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChunkUploadResponse {
    pub code: i32,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CompleteUploadRequest {
    #[serde(rename = "objectName")]
    pub object_name: String,
    #[serde(rename = "totalChunks")]
    pub total_chunks: i32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CompleteUploadResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<CompleteUploadData>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AbortUploadRequest {
    #[serde(rename = "objectName")]
    pub object_name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AbortUploadResponse {
    pub code: i32,
    pub message: String,
}
