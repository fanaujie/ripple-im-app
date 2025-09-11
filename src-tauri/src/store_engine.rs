use std::sync::Arc;

pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

pub trait StoreEngine {
    async fn exists_token(&self) -> anyhow::Result<bool>;

    // Retrieve the access token and refresh token if available
    async fn get_token(&self) -> Option<Token>;
    async fn save_token(&self, access_token: &str, refresh_token: &str) -> anyhow::Result<()>;
}

pub struct MemoryStore {
    inner: Arc<tokio::sync::Mutex<InnerStore>>,
}

struct InnerStore {
    access_token: Option<String>,
    refresh_token: Option<String>,
}

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore {
            inner: Arc::new(tokio::sync::Mutex::new(InnerStore {
                access_token: None,
                refresh_token: None,
            })),
        }
    }
}

impl Clone for MemoryStore {
    fn clone(&self) -> Self {
        MemoryStore {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl StoreEngine for MemoryStore {
    async fn exists_token(&self) -> anyhow::Result<bool> {
        // always return false for in-memory store
        Ok(false)
    }

    async fn get_token(&self) -> Option<Token> {
        let inner = self.inner.lock().await;
        if let Some(token) = &inner.access_token {
            Some(Token {
                access_token: inner.access_token.clone().unwrap(),
                refresh_token: inner.refresh_token.clone().unwrap(),
            })
        } else {
            None
        }
    }

    async fn save_token(&self, access_token: &str, refresh_token: &str) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        inner.access_token = Some(access_token.to_string());
        inner.refresh_token = Some(refresh_token.to_string());
        Ok(())
    }
}
