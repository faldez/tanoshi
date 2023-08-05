use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct Category {
    pub id: Option<i64>,
    pub name: String,
}

impl Default for Category {
    fn default() -> Self {
        Self {
            id: None,
            name: "Default".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LibraryUpdate {
    pub manga_id: i64,
    pub chapter_id: i64,
    pub manga_title: String,
    pub cover_url: String,
    pub chapter_title: String,
    pub uploaded: NaiveDateTime,
}
