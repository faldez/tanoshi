#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone)]
pub struct TrackingOauthSession {
    pub id: i64,
    pub user_id: i64,
    pub csrf_state: String,
    pub pkce_code_verifier: String,
}

#[derive(Debug, Clone)]
pub struct TrackedManga {
    pub manga_id: i64,
    pub tracker: String,
    pub tracker_manga_id: Option<String>,
}
