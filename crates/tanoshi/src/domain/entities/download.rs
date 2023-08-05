use chrono::NaiveDateTime;

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
pub struct DownloadedChapter {
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
