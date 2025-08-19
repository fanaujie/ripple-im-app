use crate::ripple::api_paths::ApiPaths;
use crate::ripple::api_response::{
    AvatarUploadResponse, CommonResponse, UpdateNickNameRequest, UserProfileResponse,
};
use crate::ripple::token_store::TokenStore;
use anyhow::anyhow;
use mime::Mime;
use oauth2::reqwest;
use reqwest::{Response, StatusCode};
use sha2::{Digest, Sha256};
use std::future::Future;

pub struct RippleApi {
    api_paths: ApiPaths,
    reqwest_client: reqwest::Client,
    token_store: TokenStore,
}

impl RippleApi {
    pub fn new(
        upload_gateway_url: String,
        api_gateway_url: String,
        reqwest_client: reqwest::Client,
        token_store: TokenStore,
    ) -> Self {
        let api_paths = ApiPaths::new(&upload_gateway_url, &api_gateway_url);
        RippleApi {
            api_paths,
            reqwest_client,
            token_store,
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
            let access_token = self
                .token_store
                .get_access_token()
                .ok_or(anyhow!("Failed to retrieve access token"))?;
            let res = api_call(access_token).await?;
            match res.status() {
                StatusCode::OK => return Ok(res),
                StatusCode::UNAUTHORIZED => {
                    if attempts < unauthorized_max_retries {
                        attempts += 1;
                        match self.token_store.refresh_token().await {
                            Ok(_) => continue,
                            Err(e) => {
                                // If refresh fails, clear the invalid token
                                let _ = self.token_store.clear_token().await;
                                return Err(anyhow!(
                                    "Token refresh failed: {}. Please login again.",
                                    e
                                ));
                            }
                        }
                    } else {
                        // Clear invalid token after max retries
                        let _ = self.token_store.clear_token().await;
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

    pub async fn initialize_token_from_db(&self) -> anyhow::Result<bool> {
        self.token_store.initialize_token_from_db().await
    }

    pub fn oauth_auth_url(&self) -> String {
        self.token_store.auth_url()
    }

    pub fn oauth_state_equal(&self, state: &str) -> bool {
        self.token_store.state_equal(state)
    }

    pub async fn oauth_request_token(&self, code: String) -> anyhow::Result<()> {
        self.token_store.request_token(code).await
    }

    pub async fn upload_avatar(
        &self,
        filename: String,
        mime: Mime,
        image_data: Vec<u8>,
    ) -> anyhow::Result<AvatarUploadResponse> {
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
        Ok(res.json::<AvatarUploadResponse>().await?)
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

    pub async fn delete_user_portrait(&self) -> anyhow::Result<CommonResponse> {
        let res = self
            .execute_with_auth_retry(
                |access_token| async move {
                    self.reqwest_client
                        .delete(&self.api_paths.profile_portrait)
                        .header("Authorization", format!("Bearer {}", access_token))
                        .send()
                        .await
                        .map_err(|e| anyhow!("Failed to delete user portrait: {}", e))
                },
                1,
            )
            .await?;
        Ok(res.json::<CommonResponse>().await?)
    }

    pub async fn clear_invalid_token(&self) -> anyhow::Result<()> {
        self.token_store.clear_token().await
    }
}
