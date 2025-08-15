use crate::app_config::AppConfig;
use anyhow::anyhow;
use oauth2::basic::{
    BasicClient, BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
    BasicTokenResponse,
};
use std::sync::RwLock;

use oauth2::{
    reqwest, AuthType, AuthUrl, Client, ClientId, ClientSecret, CsrfToken, EndpointNotSet,
    EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, Scope,
    StandardRevocableToken, TokenUrl,
};

pub struct OauthClient {
    client: Client<
        BasicErrorResponse,
        BasicTokenResponse,
        BasicTokenIntrospectionResponse,
        StandardRevocableToken,
        BasicRevocationErrorResponse,
        EndpointSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointSet,
    >,
    reqwest_client: reqwest::Client,
    state: RwLock<Option<CsrfToken>>,
    pkce_verifier: RwLock<Option<String>>,
}

impl OauthClient {
    pub fn new(app_config: &AppConfig, reqwest_client: reqwest::Client) -> anyhow::Result<Self> {
        let client = BasicClient::new(ClientId::new(app_config.oauth2_client_id.clone()))
            .set_client_secret(ClientSecret::new(app_config.oauth2_client_secret.clone()))
            .set_auth_type(AuthType::BasicAuth)
            .set_auth_uri(AuthUrl::new(app_config.oauth2_auth_url.clone())?)
            .set_token_uri(TokenUrl::new(app_config.oauth2_token_url.clone())?)
            .set_redirect_uri(RedirectUrl::new(app_config.oauth2_redirect_uri.clone())?);
        Ok(OauthClient {
            client,
            reqwest_client,
            state: RwLock::new(None),
            pkce_verifier: RwLock::new(None),
        })
    }

    pub fn auth_url(&self) -> String {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        // Generate the full authorization URL.
        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            // Set the desired scopes.
            .add_scope(Scope::new("user".to_string()))
            // Set the PKCE code challenge.
            .set_pkce_challenge(pkce_challenge)
            .url();
        {
            let mut state = self.state.write().unwrap();
            *state = Some(csrf_token);
        }
        {
            let mut pk = self.pkce_verifier.write().unwrap();
            *pk = Some(pkce_verifier.secret().to_string());
        }
        auth_url.into()
    }

    pub fn state_equal(&self, state: &str) -> bool {
        let guard_state = self.state.read().unwrap();
        match *guard_state {
            Some(ref csrf_token) => csrf_token.secret() == state,
            None => {
                eprintln!("State is None, cannot compare with provided state.");
                false
            }
        }
    }

    pub async fn request_token(&self, code: String) -> anyhow::Result<BasicTokenResponse> {
        let pv = {
            let guard_pkce_verifier = self.pkce_verifier.read().unwrap();
            if guard_pkce_verifier.is_none() {
                return Err(anyhow!("PKCE verifier is not set"));
            }
            guard_pkce_verifier.clone().unwrap()
        };
        self.client
            .exchange_code(oauth2::AuthorizationCode::new(code))
            .set_pkce_verifier(PkceCodeVerifier::new(pv))
            .request_async(&self.reqwest_client)
            .await
            .map_err(|e| anyhow!("Failed to request_token: {}", e.to_string()))
    }

    pub async fn refresh_token(
        &self,
        old_refresh_token: String,
    ) -> anyhow::Result<BasicTokenResponse> {
        let refresh_token = RefreshToken::new(old_refresh_token);
        self.client
            .exchange_refresh_token(&refresh_token)
            .request_async(&self.reqwest_client)
            .await
            .map_err(|e| anyhow!("Failed to refresh_token: {}", e.to_string()))
    }
}
