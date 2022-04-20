use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct RecentChapter {
    pub manga_id: i64,
    pub chapter_id: i64,
    pub manga_title: String,
    pub cover_url: String,
    pub chapter_title: String,
    pub read_at: NaiveDateTime,
    pub last_page_read: i64,
}

#[derive(Debug, Clone)]
pub struct RecentUpdate {
    pub manga_id: i64,
    pub chapter_id: i64,
    pub manga_title: String,
    pub cover_url: String,
    pub chapter_title: String,
    pub uploaded: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct Manga {
    pub id: i64,
    pub source_id: i64,
    pub title: String,
    pub author: Vec<String>,
    pub genre: Vec<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub path: String,
    pub cover_url: String,
    pub date_added: NaiveDateTime,
}

impl Default for Manga {
    fn default() -> Self {
        Self {
            id: 0,
            source_id: 0,
            title: "".to_string(),
            author: vec![],
            genre: vec![],
            status: None,
            description: None,
            path: "".to_string(),
            cover_url: "".to_string(),
            date_added: NaiveDateTime::from_timestamp(0, 0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chapter {
    pub id: i64,
    pub source_id: i64,
    pub manga_id: i64,
    pub title: String,
    pub path: String,
    pub number: f64,
    pub scanlator: String,
    pub uploaded: NaiveDateTime,
    pub date_added: NaiveDateTime,
    pub downloaded_path: Option<String>,
}

impl Default for Chapter {
    fn default() -> Self {
        Self {
            id: 0,
            source_id: 0,
            manga_id: 0,
            title: "".to_string(),
            path: "".to_string(),
            number: 0_f64,
            scanlator: "".to_string(),
            uploaded: NaiveDateTime::from_timestamp(0, 0),
            date_added: NaiveDateTime::from_timestamp(0, 0),
            downloaded_path: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub is_admin: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub telegram_chat_id: Option<i64>,
    pub pushover_user_key: Option<String>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: 0,
            username: "".to_string(),
            password: "".to_string(),
            is_admin: false,
            created_at: NaiveDateTime::from_timestamp(0, 0),
            updated_at: NaiveDateTime::from_timestamp(0, 0),
            telegram_chat_id: None,
            pushover_user_key: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReadProgress {
    pub at: NaiveDateTime,
    pub last_page: i64,
    pub is_complete: bool,
}

#[derive(Debug, Clone)]
pub struct UserMangaLibrary {
    pub user_ids: Vec<i64>,
    pub manga: Manga,
}

#[derive(Debug, Clone)]
pub struct DownloadQueue {
    pub id: i64,
    pub source_id: i64,
    pub source_name: String,
    pub manga_id: i64,
    pub manga_title: String,
    pub chapter_id: i64,
    pub chapter_title: String,
    pub rank: i64,
    pub url: String,
    pub priority: i64,
    pub date_added: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct DownloadQueueEntry {
    pub source_id: i64,
    pub source_name: String,
    pub manga_id: i64,
    pub manga_title: String,
    pub chapter_id: i64,
    pub chapter_title: String,
    pub downloaded: i64,
    pub total: i64,
    pub priority: i64,
}

#[derive(Debug, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct TrackingOauthSession {
    pub id: i64,
    pub user_id: i64,
    pub csrf_state: String,
    pub pkce_code_verifier: String,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone)]
pub struct TrackedManga {
    pub manga_id: i64,
    pub tracker: String,
    pub tracker_manga_id: Option<String>,
}
