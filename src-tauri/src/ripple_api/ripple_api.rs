use crate::ripple_api::api_paths::ApiPaths;
use crate::ripple_api::api_response::{
    AddFriendRequest, BlockUserRequest, CommonResponse, ConversationSyncResponse,
    ConversationVersionResponse, ConversationsResponse, MessageResponse, ReadMessagesResponse,
    RelationVersionResponse, RelationsPageResponse, RelationsSyncResponse, SendMessageRequest,
    UpdateFriendDisplayNameRequest, UpdateNickNameRequest, UserProfileResponse,
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
                        // Clear invalid token after max retries

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
                    let filename = filename.clone();
                    let mime = mime.clone();
                    async move {
                        let part = reqwest::multipart::Part::bytes(image_data)
                            .file_name(filename)
                            .mime_str(mime.to_string().as_str())?;
                        let form = reqwest::multipart::Form::new()
                            .text("hash", hex_hash)
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

    pub async fn get_user_profile(&self) -> anyhow::Result<UserProfileResponse> {
        let res = self
            .execute_with_auth_retry(
                |access_token| async move {
                    self.reqwest_client
                        .get(&self.api_paths.profile)
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
        #[derive(Serialize)]
        struct Params {
            #[serde(rename = "userId")]
            user_id: String,
        }
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let user_id_clone = user_id.clone();
                    async move {
                        let params = Params {
                            user_id: user_id_clone,
                        };
                        self.reqwest_client
                            .get(&self.api_paths.profile)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .query(&params)
                            .send()
                            .await
                            .map_err(|e| {
                                anyhow!("Failed to get user id={} profile: {}", params.user_id, e)
                            })
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<UserProfileResponse>().await?)
    }

    pub async fn update_nickname(&self, nickname: String) -> anyhow::Result<CommonResponse> {
        let request_body = UpdateNickNameRequest {
            nick_name: nickname,
        };

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let request_body = request_body.clone();
                    async move {
                        self.reqwest_client
                            .put(&self.api_paths.profile_nickname)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to update nickname: {}", e))
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
                        .delete(&self.api_paths.profile_avatar)
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

    pub async fn update_friend_remark_name(
        &self,
        friend_id: String,
        remark_name: String,
    ) -> anyhow::Result<CommonResponse> {
        let request_body = UpdateFriendDisplayNameRequest { remark_name };
        let url = format!("{}/{}/remark-name", &self.api_paths.friends, friend_id);

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let request_body = request_body.clone();
                    let url = url.clone();
                    async move {
                        self.reqwest_client
                            .put(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to update friend display name: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

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

    pub async fn hide_blocked_user(
        &self,
        target_user_id: String,
    ) -> anyhow::Result<CommonResponse> {
        let url = format!("{}/{}/hide", &self.api_paths.blocked_users, target_user_id);
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let url = url.clone();
                    async move {
                        self.reqwest_client
                            .patch(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to hide blocked user: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

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
                        let request = self
                            .reqwest_client
                            .get(&self.api_paths.relations_sync)
                            .header("Authorization", format!("Bearer {}", access_token));

                        let params = Params {
                            version: last_version_clone,
                        };
                        request
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

    pub async fn get_relation_version(&self) -> anyhow::Result<RelationVersionResponse> {
        let res = self
            .execute_with_auth_retry(
                |access_token| async move {
                    self.reqwest_client
                        .get(&self.api_paths.relations_version)
                        .header("Authorization", format!("Bearer {}", access_token))
                        .send()
                        .await
                        .map_err(|e| anyhow!("Failed to get relation version: {}", e))
                },
                1,
            )
            .await?;
        Ok(res.json::<RelationVersionResponse>().await?)
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
                        let request = self
                            .reqwest_client
                            .get(&self.api_paths.conversations_sync)
                            .header("Authorization", format!("Bearer {}", access_token));

                        let params = Params {
                            version: last_version_clone,
                        };
                        request
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

    pub async fn get_conversation_version(&self) -> anyhow::Result<ConversationVersionResponse> {
        let res = self
            .execute_with_auth_retry(
                |access_token| async move {
                    self.reqwest_client
                        .get(&self.api_paths.conversations_version)
                        .header("Authorization", format!("Bearer {}", access_token))
                        .send()
                        .await
                        .map_err(|e| anyhow!("Failed to get conversation version: {}", e))
                },
                1,
            )
            .await?;
        Ok(res.json::<ConversationVersionResponse>().await?)
    }

    pub async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> anyhow::Result<MessageResponse> {
        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let request = request.clone();
                    async move {
                        self.reqwest_client
                            .post(&self.api_paths.send_message)
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
            #[serde(rename = "conversationId")]
            conversation_id: String,
            #[serde(rename = "messageId")]
            message_id: String,
            #[serde(rename = "readSize")]
            read_size: u32,
        }

        let params = ReadMessagesParams {
            conversation_id,
            message_id,
            read_size,
        };

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let params = params.clone();
                    async move {
                        self.reqwest_client
                            .get(&self.api_paths.read_messages)
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

    pub async fn mark_last_read_message_id(
        &self,
        conversation_id: String,
        message_id: String,
    ) -> anyhow::Result<CommonResponse> {
        let url = format!(
            "{}/{}/message/{}/mark-read",
            &self.api_paths.conversations, conversation_id, message_id
        );

        let res = self
            .execute_with_auth_retry(
                |access_token| {
                    let url = url.clone();
                    async move {
                        self.reqwest_client
                            .put(&url)
                            .header("Authorization", format!("Bearer {}", access_token))
                            .send()
                            .await
                            .map_err(|e| anyhow!("Failed to mark last read message: {}", e))
                    }
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }
}
