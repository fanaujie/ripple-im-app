use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub signup_url: String,
    pub oauth2_client_id: String,
    pub oauth2_client_secret: String,
    pub oauth2_auth_url: String,
    pub oauth2_token_url: String,
    pub callback_server_addr: String,
    pub oauth2_redirect_uri: String,
}
