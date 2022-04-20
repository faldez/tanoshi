use anyhow::{anyhow, Result};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::config::GLOBAL_CONFIG;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub is_admin: bool,
    pub exp: usize,
}

pub fn decode_jwt(token: &str) -> Result<Claims> {
    let secret = GLOBAL_CONFIG
        .get()
        .map(|cfg| cfg.secret.to_owned())
        .ok_or_else(|| anyhow!("no secret in config"))?;

    Ok(jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?
    .claims)
}

pub fn encode_jwt(claims: &Claims) -> Result<String> {
    let secret = GLOBAL_CONFIG
        .get()
        .map(|cfg| cfg.secret.to_owned())
        .ok_or_else(|| anyhow!("no secret in config"))?;

    Ok(jsonwebtoken::encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}
