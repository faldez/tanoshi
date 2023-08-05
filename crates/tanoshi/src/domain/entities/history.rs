use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct HistoryChapter {
    pub manga_id: i64,
    pub chapter_id: i64,
    pub manga_title: String,
    pub cover_url: String,
    pub chapter_title: String,
    pub read_at: NaiveDateTime,
    pub last_page_read: i64,
    pub is_complete: bool,
}
