use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    #[serde(skip, default)]
    pub is_dev: bool,
    pub signup_url: String,
    pub oauth2_client_id: String,
    pub oauth2_client_secret: String,
    pub oauth2_auth_url: String,
    pub oauth2_token_url: String,
    pub callback_server_addr: String,
    pub oauth2_redirect_uri: String,
    pub upload_gateway_url: String,
    pub api_gateway_url: String,
    pub ws_gateway_url: String,
}
