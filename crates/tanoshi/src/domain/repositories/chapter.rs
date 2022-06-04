use async_trait::async_trait;

use thiserror::Error;

use crate::domain::entities::chapter::Chapter;

#[derive(Debug, Error)]
pub enum ChapterRepositoryError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
}

#[async_trait]
pub trait ChapterRepository: Send + Sync {
    async fn insert_chapters(&self, chapters: &[Chapter]) -> Result<(), ChapterRepositoryError>;

    async fn get_chapter_by_id(&self, id: i64) -> Result<Chapter, ChapterRepositoryError>;

    async fn get_chapter_by_source_id_path(
        &self,
        source_id: i64,
        path: &str,
    ) -> Result<Chapter, ChapterRepositoryError>;

    async fn get_chapters_by_manga_id(
        &self,
        manga_id: i64,
        limit: Option<i64>,
        order_by: Option<&'static str>,
        asc: bool,
    ) -> Result<Vec<Chapter>, ChapterRepositoryError>;
}
