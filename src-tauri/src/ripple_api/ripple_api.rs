use crate::ripple_api::api_paths::ApiPaths;
use crate::ripple_api::api_response::{
    AddFriendRequest, BlockUserRequest, CommonResponse, ConversationSummariesResponse,
    ConversationSyncResponse, ConversationsResponse, CreateGroupRequest, CreateGroupResponse,
    GetGroupMembersResponse, GetUserGroupsResponse, GroupSyncResponse, InviteGroupMemberRequest,
    MessageResponse, ReadMessagesResponse, RelationsPageResponse, RelationsSyncResponse,
    SendMessageRequest, UpdateBlockedUserRequest, UpdateFriendRequest, UpdateGroupRequest,
    UpdateProfileRequest, UpdateReadPositionRequest, UserGroupSyncResponse, UserProfileResponse,
};
use crate::ripple_api::oauth_client::OauthClient;
use crate::store_engine::StoreEngine;
use anyhow::anyhow;
use mime::Mime;
use oauth2::{reqwest, TokenResponse};
use reqwest::{Response, StatusCode};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::future::Future;

#[derive(Clone)]
pub struct RippleApi<E>
where
    E: StoreEngine,
{
    api_paths: ApiPaths,
    reqwest_client: reqwest::Client,
    oauth_client: OauthClient,
    store_engine: E,
}

impl<S> RippleApi<S>
where
    S: StoreEngine,
{
    pub fn new(
        upload_gateway_url: String,
        api_gateway_url: String,
        reqwest_client: reqwest::Client,
        oauth_client: OauthClient,
        store_engine: S,
    ) -> Self {
        let api_paths = ApiPaths::new(&upload_gateway_url, &api_gateway_url);
        RippleApi {
            api_paths,
            reqwest_client,
            oauth_client,
            store_engine,
        }
    }

    async fn execute_with_auth_retry<F, Fut>(
        &self,
        api_call: F,
        unauthorized_max_retries: u8,
    ) -> anyhow::Result<Response>
    where
        F: Fn(String) -> Fut,
        Fut: Future<Output = anyhow::Result<Response>>,
    {
        let mut attempts = 0u8;
        loop {
            let token = match self.store_engine.get_token().await? {
                Some(t) => t,
                None => {
                    anyhow::bail!("No authentication token found. Please login.");
                }
            };
            let res = api_call(token.access_token).await?;
            match res.status() {
                StatusCode::OK => return Ok(res),
                StatusCode::UNAUTHORIZED => {
                    if attempts < unauthorized_max_retries {
                        attempts += 1;
                        match self.oauth_client.refresh_token(token.refresh_token).await {
                            Ok(token_response) => {
                                self.store_engine
                                    .save_token(
                                        token_response.access_token().secret(),
                                        token_response.refresh_token().unwrap().secret(),
                                    )
                                    .await?;
                                continue;
                            }
                            Err(e) => {
                                return Err(anyhow!(
                                    "Token refresh failed: {}. Please login again.",
                                    e
                                ));
                            }
                        }
                    } else {
                        return Err(anyhow!("Authentication failed. Please login again."));
                    }
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "API call failed with status: {}",
                        res.status()
                    ));
                }
            }
        }
    }

    pub fn oauth_auth_url(&self) -> String {
        self.oauth_client.auth_url()
    }

    pub fn oauth_state_equal(&self, state: &str) -> bool {
        self.oauth_client.state_equal(state)
    }

    pub async fn oauth_request_token(&self, code: String) -> anyhow::Result<()> {
        match self.oauth_client.request_token(code).await {
            Ok(token_response) => {
                self.store_engine
                    .save_token(
                        token_response.access_token().secret(),
                        token_response.refresh_token().unwrap().secret(),
                    )
                    .await
            }
            Err(e) => Err(anyhow!("Failed to request token: {:#?}", e)),
        }
    }

    pub async fn upload_avatar(
        &self,
        filename: String,
        mime: Mime,
        image_data: Vec<u8>,
    ) -> anyhow::Result<CommonResponse> {
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let image_data = image_data.clone();
                    let hash = Sha256::digest(&image_data);
                    let hex_hash = base16ct::lower::encode_string(&hash);
                    let clone_filename = filename.clone();
                    let clone_mime = mime.clone();
                    async move {
                        let part = reqwest::multipart::Part::bytes(image_data)
                            .file_name(clone_filename.clone())
                            .mime_str(clone_mime.to_string().as_str())?;
                        let form = reqwest::multipart::Form::new()
                            .text("hash", hex_hash)
                            .text("originalFilename", clone_filename)
                            .part("avatar", part);
                        self.reqwest_client
                            .put(&self.api_paths.upload_avatar)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .multipart(form)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to upload avatar: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    // ==================== Profile APIs ====================

    pub async fn get_user_profile(&self) -> anyhow::Result<UserProfileResponse> {
        let res = self
            .execute_with_auth_retry(
                |access_token| async move {
                    self.reqwest_client
                        .get(&self.api_paths.my_profile)
                        .header("Authorization", format!("Bearer {}", access_token))
                        .send()
                        .await
                        .map_err(|e| anyhow!("Failed to get user profile: {}", e))
                },
                1,
            )
            .await?;
        Ok(res.json::<UserProfileResponse>().await?)
    }

    pub async fn get_user_profile_by_id(
        &self,
        user_id: String,
    ) -> anyhow::Result<UserProfileResponse> {
        let url = format!("{}/{}/profile", &self.api_paths.user_profile, user_id);
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let url = url.clone();
                    async move {
                        self.reqwest_client
                            .get(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to get user profile: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<UserProfileResponse>().await?)
    }

    pub async fn update_profile(&self, nickname: Option<String>) -> anyhow::Result<CommonResponse> {
        let request_body = UpdateProfileRequest { nickname };

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let request_body = request_body.clone();
                    async move {
                        self.reqwest_client
                            .patch(&self.api_paths.my_profile)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to update profile: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    pub async fn delete_user_avatar(&self) -> anyhow::Result<CommonResponse> {
        let res = self
            .execute_with_auth_retry(
                |access_token| async move {
                    self.reqwest_client
                        .delete(&self.api_paths.my_avatar)
                        .header("Authorization", format!("Bearer {}", access_token))
                        .send()
                        .await
                        .map_err(|e| anyhow!("Failed to delete user avatar: {}", e))
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    // ==================== Friend APIs ====================

    pub async fn add_friend(&self, target_user_id: String) -> anyhow::Result<CommonResponse> {
        let request_body = AddFriendRequest { target_user_id };

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let request_body = request_body.clone();
                    async move {
                        self.reqwest_client
                            .post(&self.api_paths.friends)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to add friend: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    pub async fn remove_friend(&self, friend_id: String) -> anyhow::Result<CommonResponse> {
        let url = format!("{}/{}", &self.api_paths.friends, friend_id);
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let url = url.clone();
                    async move {
                        self.reqwest_client
                            .delete(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to remove friend: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    pub async fn update_friend(
        &self,
        friend_id: String,
        remark_name: Option<String>,
    ) -> anyhow::Result<CommonResponse> {
        let request_body = UpdateFriendRequest { remark_name };
        let url = format!("{}/{}", &self.api_paths.friends, friend_id);

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let request_body = request_body.clone();
                    let url = url.clone();
                    async move {
                        self.reqwest_client
                            .patch(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to update friend: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    // ==================== Block User APIs ====================

    pub async fn block_user(&self, target_user_id: String) -> anyhow::Result<CommonResponse> {
        let request_body = BlockUserRequest { target_user_id };

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let request_body = request_body.clone();
                    async move {
                        self.reqwest_client
                            .post(&self.api_paths.blocked_users)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to block user: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    pub async fn unblock_user(&self, target_user_id: String) -> anyhow::Result<CommonResponse> {
        let url = format!("{}/{}", &self.api_paths.blocked_users, target_user_id);
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let url = url.clone();
                    async move {
                        self.reqwest_client
                            .delete(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to unblock user: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    pub async fn update_blocked_user(
        &self,
        target_user_id: String,
        hidden: Option<bool>,
    ) -> anyhow::Result<CommonResponse> {
        let url = format!("{}/{}", &self.api_paths.blocked_users, target_user_id);
        let request_body = UpdateBlockedUserRequest { hidden };

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let url = url.clone();
                    let request_body = request_body.clone();
                    async move {
                        self.reqwest_client
                            .patch(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to update blocked user: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    // ==================== Relations APIs ====================

    pub async fn get_relations(
        &self,
        next_page_token: Option<String>,
        page_size: u32,
    ) -> anyhow::Result<RelationsPageResponse> {
        #[derive(Serialize)]
        struct Params {
            #[serde(rename = "nextPageToken", skip_serializing_if = "Option::is_none")]
            next_page_token: Option<String>,
            #[serde(rename = "pageSize")]
            page_size: u32,
        }
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let next_page_token_clone = next_page_token.clone();
                    async move {
                        let params = Params {
                            next_page_token: next_page_token_clone,
                            page_size,
                        };
                        self.reqwest_client
                            .get(&self.api_paths.relations)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .query(&params)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to get relations: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<RelationsPageResponse>().await?)
    }

    pub async fn sync_relations(
        &self,
        last_version: Option<String>,
    ) -> anyhow::Result<RelationsSyncResponse> {
        #[derive(Serialize)]
        struct Params {
            #[serde(skip_serializing_if = "Option::is_none")]
            version: Option<String>,
        }
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let last_version_clone = last_version.clone();
                    async move {
                        let params = Params {
                            version: last_version_clone,
                        };
                        self.reqwest_client
                            .get(&self.api_paths.relations_sync)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .query(&params)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to sync relations: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<RelationsSyncResponse>().await?)
    }

    // ==================== Conversation APIs ====================

    pub async fn get_conversations(
        &self,
        next_page_token: Option<String>,
        page_size: u32,
    ) -> anyhow::Result<ConversationsResponse> {
        #[derive(Serialize)]
        struct Params {
            #[serde(rename = "nextPageToken", skip_serializing_if = "Option::is_none")]
            next_page_token: Option<String>,
            #[serde(rename = "pageSize")]
            page_size: u32,
        }
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let next_page_token_clone = next_page_token.clone();
                    async move {
                        let params = Params {
                            next_page_token: next_page_token_clone,
                            page_size,
                        };
                        self.reqwest_client
                            .get(&self.api_paths.conversations)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .query(&params)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to get conversations: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<ConversationsResponse>().await?)
    }

    pub async fn sync_conversations(
        &self,
        last_version: Option<String>,
    ) -> anyhow::Result<ConversationSyncResponse> {
        #[derive(Serialize)]
        struct Params {
            #[serde(skip_serializing_if = "Option::is_none")]
            version: Option<String>,
        }
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let last_version_clone = last_version.clone();
                    async move {
                        let params = Params {
                            version: last_version_clone,
                        };
                        self.reqwest_client
                            .get(&self.api_paths.conversations_sync)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .query(&params)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to sync conversations: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<ConversationSyncResponse>().await?)
    }

    pub async fn get_conversation_summaries(
        &self,
        conversation_ids: Vec<String>,
    ) -> anyhow::Result<ConversationSummariesResponse> {
        #[derive(Serialize, Clone)]
        struct GetConversationSummariesRequest {
            #[serde(rename = "conversationIds")]
            conversation_ids: Vec<String>,
        }

        let request = GetConversationSummariesRequest { conversation_ids };

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let request = request.clone();
                    async move {
                        self.reqwest_client
                            .post(&self.api_paths.conversations_summary)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to get conversation summaries: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<ConversationSummariesResponse>().await?)
    }

    // ==================== Message APIs ====================

    pub async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> anyhow::Result<MessageResponse> {
        let url = format!("{}/messages", &self.api_paths.conversations);

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let request = request.clone();
                    let url = url.clone();
                    async move {
                        self.reqwest_client
                            .post(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to send message: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<MessageResponse>().await?)
    }

    pub async fn read_messages(
        &self,
        conversation_id: String,
        message_id: String,
        read_size: u32,
    ) -> anyhow::Result<ReadMessagesResponse> {
        #[derive(Serialize, Clone)]
        struct ReadMessagesParams {
            #[serde(rename = "messageId")]
            message_id: String,
            #[serde(rename = "readSize")]
            read_size: u32,
        }

        let params = ReadMessagesParams {
            message_id,
            read_size,
        };

        let url = format!(
            "{}/{}/messages",
            &self.api_paths.conversations, conversation_id
        );

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let url = url.clone();
                    let params = params.clone();
                    async move {
                        self.reqwest_client
                            .get(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .query(&params)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to read messages: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<ReadMessagesResponse>().await?)
    }

    pub async fn update_read_position(
        &self,
        conversation_id: String,
        message_id: String,
    ) -> anyhow::Result<CommonResponse> {
        let url = format!(
            "{}/{}/read-position",
            &self.api_paths.conversations, conversation_id
        );
        let request_body = UpdateReadPositionRequest { message_id };

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let url = url.clone();
                    let request_body = request_body.clone();
                    async move {
                        self.reqwest_client
                            .patch(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to update read position: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    // ==================== Group APIs ====================

    pub async fn create_group(
        &self,
        sender_id: String,
        group_name: String,
        member_ids: Vec<String>,
    ) -> anyhow::Result<CreateGroupResponse> {
        let request_body = CreateGroupRequest {
            sender_id,
            group_name,
            group_avatar: None,
            member_ids,
        };

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let request_body = request_body.clone();
                    async move {
                        self.reqwest_client
                            .post(&self.api_paths.groups)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to create group: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CreateGroupResponse>().await?)
    }

    pub async fn update_group(
        &self,
        group_id: String,
        sender_id: String,
        name: Option<String>,
        avatar: Option<String>,
    ) -> anyhow::Result<CommonResponse> {
        let url = format!("{}/{}", &self.api_paths.groups, group_id);
        let request_body = UpdateGroupRequest {
            sender_id,
            name,
            avatar,
        };

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let request_body = request_body.clone();
                    let url = url.clone();
                    async move {
                        self.reqwest_client
                            .patch(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to update group: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    pub async fn add_group_members(
        &self,
        group_id: String,
        sender_id: String,
        member_ids: Vec<String>,
        group_name: String,
        group_avatar: Option<String>,
    ) -> anyhow::Result<CommonResponse> {
        let url = format!("{}/{}/members", &self.api_paths.groups, group_id);
        let request_body = InviteGroupMemberRequest {
            sender_id,
            new_member_ids: member_ids,
            group_name,
            group_avatar,
        };

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let request_body = request_body.clone();
                    let url = url.clone();
                    async move {
                        self.reqwest_client
                            .post(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to add group members: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    pub async fn get_group_members(
        &self,
        group_id: String,
        next_page_token: Option<String>,
        page_size: u32,
    ) -> anyhow::Result<GetGroupMembersResponse> {
        #[derive(Serialize)]
        struct Params {
            #[serde(rename = "nextPageToken", skip_serializing_if = "Option::is_none")]
            next_page_token: Option<String>,
            #[serde(rename = "pageSize")]
            page_size: u32,
        }
        let url = format!("{}/{}/members", &self.api_paths.groups, group_id);

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let url = url.clone();
                    let next_page_token_clone = next_page_token.clone();
                    async move {
                        let params = Params {
                            next_page_token: next_page_token_clone,
                            page_size,
                        };
                        self.reqwest_client
                            .get(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .query(&params)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to get group members: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<GetGroupMembersResponse>().await?)
    }

    pub async fn leave_group(&self, group_id: String) -> anyhow::Result<CommonResponse> {
        let url = format!("{}/{}/members/me", &self.api_paths.groups, group_id);

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let url = url.clone();
                    async move {
                        self.reqwest_client
                            .delete(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to leave group: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    pub async fn sync_group_members(
        &self,
        group_id: String,
        last_version: Option<String>,
    ) -> anyhow::Result<GroupSyncResponse> {
        #[derive(Serialize)]
        struct Params {
            #[serde(skip_serializing_if = "Option::is_none")]
            version: Option<String>,
        }

        let url = format!("{}/{}/members/sync", &self.api_paths.groups, group_id);

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let url = url.clone();
                    let last_version_clone = last_version.clone();
                    async move {
                        let params = Params {
                            version: last_version_clone,
                        };
                        self.reqwest_client
                            .get(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .query(&params)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to sync group members: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<GroupSyncResponse>().await?)
    }

    // ==================== User Groups APIs ====================

    pub async fn get_user_groups(
        &self,
        next_page_token: Option<String>,
        page_size: u32,
    ) -> anyhow::Result<GetUserGroupsResponse> {
        #[derive(Serialize)]
        struct Params {
            #[serde(rename = "nextPageToken", skip_serializing_if = "Option::is_none")]
            next_page_token: Option<String>,
            #[serde(rename = "pageSize")]
            page_size: u32,
        }
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let next_page_token_clone = next_page_token.clone();
                    async move {
                        let params = Params {
                            next_page_token: next_page_token_clone,
                            page_size,
                        };
                        self.reqwest_client
                            .get(&self.api_paths.user_groups)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .query(&params)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to get user groups: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<GetUserGroupsResponse>().await?)
    }

    pub async fn sync_user_groups(
        &self,
        last_version: Option<String>,
    ) -> anyhow::Result<UserGroupSyncResponse> {
        #[derive(Serialize)]
        struct Params {
            #[serde(skip_serializing_if = "Option::is_none")]
            version: Option<String>,
        }
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let last_version_clone = last_version.clone();
                    async move {
                        let params = Params {
                            version: last_version_clone,
                        };
                        self.reqwest_client
                            .get(&self.api_paths.user_groups_sync)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .query(&params)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to sync user groups: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<UserGroupSyncResponse>().await?)
    }
}
