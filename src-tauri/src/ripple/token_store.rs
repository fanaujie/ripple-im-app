use crate::db::DB;
use crate::ripple::oauth_client::OauthClient;
use anyhow::anyhow;
use oauth2::TokenResponse;
use std::sync::RwLock;

pub struct TokenStore {
    oauth_client: OauthClient,
    db: DB,
    token: RwLock<Option<(String, String)>>, // (access_token, refresh_token)
}

impl TokenStore {
    pub fn new(oauth_client: OauthClient, db: DB) -> Self {
        TokenStore {
            oauth_client,
            db,
            token: RwLock::new(None),
        }
    }

    pub fn auth_url(&self) -> String {
        self.oauth_client.auth_url()
    }

    pub fn state_equal(&self, state: &str) -> bool {
        self.oauth_client.state_equal(state)
    }

    pub async fn request_token(&self, code: String) -> anyhow::Result<()> {
        match self.oauth_client.request_token(code).await {
            Ok(token) => {
                let access_token = token.access_token().secret();
                let refresh_token = token.refresh_token().unwrap().secret();
                {
                    let mut guard_token = self.token.write().unwrap();
                    *guard_token = Some((access_token.to_string(), refresh_token.to_string()));
                }
                self.db
                    .save_token(access_token, refresh_token)
                    .await
                    .map(|_| ())
            }
            Err(e) => Err(anyhow!("Failed to request token: {}", e.to_string())),
        }
    }

    pub async fn initialize_token_from_db(&self) -> anyhow::Result<bool> {
        match self.db.exists_token().await {
            Ok(exist) => {
                if !exist {
                    // No token exists in the database
                    return Ok(false);
                }
                // Token exists in the database, load it
                match self.db.get_token().await {
                    Ok(token) => {
                        let mut guard_token = self.token.write().unwrap();
                        *guard_token = Some(token);
                        Ok(true)
                    }
                    Err(e) => {
                        eprintln!("Failed to load token from DB: {}", e);
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to check token existence in DB: {}", e);
                Err(anyhow!("Failed to initialize token from DB: {}", e))
            }
        }
    }

    pub fn get_access_token(&self) -> Option<String> {
        let guard_token = self.token.read().unwrap();
        if guard_token.is_none() {
            return None;
        }
        Some(guard_token.as_ref().unwrap().0.clone())
    }
    pub async fn refresh_token(&self) -> anyhow::Result<()> {
        let refresh_token = {
            let guard_token = self.token.read().unwrap();
            if guard_token.is_none() {
                return Err(anyhow!("No token available to refresh"));
            }
            guard_token.as_ref().unwrap().1.clone()
        };
        match self.oauth_client.refresh_token(refresh_token).await {
            Ok(token) => {
                let new_access_token = token.access_token().secret().to_string();
                let new_refresh_token = token.refresh_token().unwrap().secret().to_string();
                
                // Save to database first
                match self.db.save_token(&new_access_token, &new_refresh_token).await {
                    Ok(_) => {
                        // Update memory state only if database save succeeds
                        let mut guard_token = self.token.write().unwrap();
                        *guard_token = Some((new_access_token, new_refresh_token));
                        Ok(())
                    }
                    Err(e) => Err(anyhow!("Failed to save refreshed token to database: {}", e)),
                }
            }
            Err(e) => Err(anyhow!("Failed to refresh token: {}", e.to_string())),
        }
    }

    pub async fn clear_token(&self) -> anyhow::Result<()> {
        // Clear memory state
        {
            let mut guard_token = self.token.write().unwrap();
            *guard_token = None;
        }
        
        // Clear database tokens
        self.db.clear_tokens().await?;
        
        Ok(())
    }
}
