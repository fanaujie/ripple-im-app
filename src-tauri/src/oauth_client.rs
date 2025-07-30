use crate::app_config::AppConfig;
use crate::errors::CommandError;
use oauth2::basic::{
    BasicClient, BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
    BasicTokenResponse,
};

use oauth2::{
    reqwest, AuthType, AuthUrl, Client, ClientId, ClientSecret, CsrfToken, EndpointNotSet,
    EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, StandardRevocableToken,
    TokenResponse, TokenUrl,
};

#[derive(Clone)]
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
    state: Option<CsrfToken>,
    pkce_verifier: Option<String>,
}

impl OauthClient {
    pub fn new(app_config: &AppConfig) -> Result<Self, CommandError> {
        let client = BasicClient::new(ClientId::new(app_config.oauth2_client_id.clone()))
            .set_client_secret(ClientSecret::new(app_config.oauth2_client_secret.clone()))
            .set_auth_type(AuthType::BasicAuth)
            .set_auth_uri(AuthUrl::new(app_config.oauth2_auth_url.clone())?)
            .set_token_uri(TokenUrl::new(app_config.oauth2_token_url.clone())?)
            .set_redirect_uri(RedirectUrl::new(app_config.oauth2_redirect_uri.clone())?);
        Ok(OauthClient {
            client,
            state: None,
            pkce_verifier: None,
        })
    }

    pub fn auth_url(&mut self) -> String {
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
        self.state = Some(csrf_token);
        self.pkce_verifier = Some(pkce_verifier.secret().to_string());
        auth_url.into()
    }

    pub fn state_equal(&self, state: &str) -> bool {
        match self.state {
            Some(ref csrf_token) => csrf_token.secret() == state,
            None => {
                eprintln!("State is None, cannot compare with provided state.");
                false
            }
        }
    }

    pub async fn request_token(&self, code: String) -> Result<String, CommandError> {
        if self.pkce_verifier.is_none() {
            return Err(CommandError::PkceVerifierError);
        }

        let http_client = reqwest::ClientBuilder::new()
            //.proxy(reqwest::Proxy::http("192.168.50.31:9000")?)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");

        let token = self
            .client
            .exchange_code(oauth2::AuthorizationCode::new(code))
            .set_pkce_verifier(PkceCodeVerifier::new(self.pkce_verifier.clone().unwrap()))
            .request_async(&http_client)
            .await;
        match token {
            Ok(token) => {
                let refresh_token = token.refresh_token().unwrap();
                println!("Access Token: {:?}", token.access_token().secret());
                println!("Refresh Token: {:?}", refresh_token.secret());

                // let new_token = self
                //     .client
                //     .exchange_refresh_token(refresh_token)
                //     .request_async(&http_client)
                //     .await;
                // match new_token {
                //     Ok(new_token) => {
                //         println!("New Access Token: {:?}", new_token.access_token().secret());
                //         println!(
                //             "New Refresh Token: {:?}",
                //             new_token.refresh_token().unwrap().secret()
                //         );
                //     }
                //     _ => {
                //         println!("Failed to refresh token");
                //     }
                // }
                Ok("Token received successfully".to_string())
            }
            Err(e) => {
                eprintln!("Error requesting token: {}", e);
                Err(CommandError::TokenError)
            }
        }
    }
}
