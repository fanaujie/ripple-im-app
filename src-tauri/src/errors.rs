use oauth2::{reqwest, url};

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error("Failed to get the token")]
    TokenError,
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error("Invalid pkce verifier")]
    PkceVerifierError,
}

impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
