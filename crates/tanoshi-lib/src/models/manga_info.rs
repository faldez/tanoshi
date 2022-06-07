use serde::{Deserialize, Serialize};

/// A type represent manga details, normalized across source
#[derive(Debug, Deserialize, Serialize)]
pub struct MangaInfo {
    pub source_id: i64,
    pub title: String,
    pub author: Vec<String>,
    pub genre: Vec<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub path: String,
    pub cover_url: String,
}
