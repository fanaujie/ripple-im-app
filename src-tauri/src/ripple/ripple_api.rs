use crate::ripple::api_paths::ApiPaths;
use crate::ripple::api_response::{
    AvatarUploadResponse, CommonResponse, UpdateNickNameRequest, UserProfileResponse,
};
use crate::ripple::oauth_client::OauthClient;
use crate::store_engine::StoreEngine;
use anyhow::anyhow;
use mime::Mime;
use oauth2::{reqwest, TokenResponse};
use reqwest::{Response, StatusCode};
use sha2::{Digest, Sha256};
use std::future::Future;

pub struct RippleApi<E>
where
    E: StoreEngine + Sync + Send,
{
    api_paths: ApiPaths,
    reqwest_client: reqwest::Client,
    oauth_client: OauthClient,
    store_engine: E,
}

impl<E> RippleApi<E>
where
    E: StoreEngine + Sync + Send,
{
    pub fn new(
        upload_gateway_url: String,
        api_gateway_url: String,
        reqwest_client: reqwest::Client,
        oauth_client: OauthClient,
        store_engine: E,
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
            let token = self
                .store_engine
                .get_token()
                .await
                .ok_or(anyhow!("Failed to retrieve access token"))?;
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
            Err(e) => Err(anyhow!("Failed to request token: {}", e.to_string())),
        }
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
}
