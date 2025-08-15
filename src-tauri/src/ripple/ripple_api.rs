use crate::ripple::api_response::AvatarUploadResponse;
use crate::ripple::token_store::TokenStore;
use anyhow::anyhow;
use mime::Mime;
use oauth2::reqwest;
use reqwest::{Response, StatusCode};
use sha2::{Digest, Sha256};
use std::future::Future;

pub struct RippleApi {
    upload_gateway_url: String,
    api_gateway_url: String,
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
        RippleApi {
            upload_gateway_url,
            api_gateway_url,
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
                        self.token_store.refresh_token().await?;
                        continue;
                    } else {
                        return Err(anyhow!("Unauthorized access after {} attempts", attempts));
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
                            .put(&format!("{}/api/upload/avatar", self.upload_gateway_url))
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
}
