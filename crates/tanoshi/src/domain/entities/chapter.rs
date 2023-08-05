use chrono::{NaiveDateTime, Utc};

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
    pub next: Option<i64>,
    pub prev: Option<i64>,
}

impl From<tanoshi_lib::models::ChapterInfo> for Chapter {
    fn from(ch: tanoshi_lib::models::ChapterInfo) -> Self {
        Self {
            id: 0,
            source_id: ch.source_id,
            manga_id: 0,
            title: ch.title,
            path: ch.path,
            number: ch.number,
            scanlator: ch.scanlator.unwrap_or_default(),
            uploaded: NaiveDateTime::from_timestamp_opt(ch.uploaded, 0).unwrap_or_default(),
            date_added: Utc::now().naive_utc(),
            downloaded_path: None,
            next: None,
            prev: None,
        }
    }
}
