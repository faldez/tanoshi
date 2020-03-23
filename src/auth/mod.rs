use serde::{Deserialize, Serialize};

pub mod auth;
use auth::Auth;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserResponse {
    pub claim: Option<Claims>,
    pub token: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Claims {
    pub sub: String,
    pub company: String,
    pub exp: usize,
}
