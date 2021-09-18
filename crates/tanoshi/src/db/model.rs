use chrono::NaiveDateTime;

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
    pub prev: Option<i64>,
    pub next: Option<i64>,
    pub uploaded: NaiveDateTime,
    pub date_added: NaiveDateTime,
    pub pages: Vec<String>,
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
            prev: None,
            next: None,
            uploaded: NaiveDateTime::from_timestamp(0, 0),
            date_added: NaiveDateTime::from_timestamp(0, 0),
            pages: vec![],
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
