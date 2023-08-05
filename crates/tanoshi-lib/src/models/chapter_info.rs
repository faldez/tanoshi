use serde::{Deserialize, Serialize};
/// A type represent chapter, normalized across source
///
#[derive(Debug, Deserialize, Serialize)]
pub struct ChapterInfo {
    pub source_id: i64,
    pub title: String,
    pub path: String,
    pub number: f64,
    pub scanlator: Option<String>,
    pub uploaded: i64,
}
