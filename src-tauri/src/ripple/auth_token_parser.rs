use anyhow::anyhow;
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};

use serde::{Deserialize, Serialize};
use serde_json;
/*
{
  "sub": "100262221746193601371",
  "aud": "ripple-im-desktop",
  "nbf": 1754119775,
  "scope": [
    "user"
  ],
  "iss": "http://localhost:8080",
  "exp": 1754123375,
  "iat": 1754119775,
  "jti": "7051c9fb-639a-4174-96b7-5dcaba315bff"
}
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    aud: String,        // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    nbf: usize, // Optional. Not Before (as UTC timestamp)
    sub: String, // Optional. Subject (whom token refers to)
    jti: String, // Optional. JWT ID (unique identifier for the token)
    scope: Vec<String>, // Optional. Scopes granted by the token
}

pub struct AuthTokenParser();

impl AuthTokenParser {
    pub fn decode_jwt_payload(token: &str) -> anyhow::Result<Claims> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Failed to split the JWT into parts"));
        }
        println!("Decoding JWT payload: {}", parts[1]);
        let payload = STANDARD_NO_PAD.decode(parts[1])?;
        let claims: Claims = serde_json::from_slice(&payload)?;
        Ok(claims)
    }
}
