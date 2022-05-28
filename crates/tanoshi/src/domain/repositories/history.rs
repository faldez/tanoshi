use async_trait::async_trait;

use thiserror::Error;

use crate::domain::entities::history::HistoryChapter;

#[derive(Debug, Error)]
pub enum HistoryRepositoryError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
}

#[async_trait]
pub trait HistoryRepository {
    async fn get_first_history_chapters(
        &self,
        user_id: i64,
        after_timestamp: i64,
        before_timestamp: i64,
        first: i32,
    ) -> Result<Vec<HistoryChapter>, HistoryRepositoryError>;

    async fn get_last_history_chapters(
        &self,
        user_id: i64,
        after_timestamp: i64,
        before_timestamp: i64,
        last: i32,
    ) -> Result<Vec<HistoryChapter>, HistoryRepositoryError>;

    async fn get_history_chapters(
        &self,
        user_id: i64,
        after_timestamp: i64,
        before_timestamp: i64,
    ) -> Result<Vec<HistoryChapter>, HistoryRepositoryError>;

    async fn insert_history_chapter(
        &self,
        user_id: i64,
        chapter_id: i64,
        page: i64,
        is_complete: bool,
    ) -> Result<(), HistoryRepositoryError>;

    async fn insert_history_chapters_as_completed(
        &self,
        user_id: i64,
        chapter_ids: &[i64],
    ) -> Result<(), HistoryRepositoryError>;

    async fn delete_chapters_from_history(
        &self,
        user_id: i64,
        chapter_ids: &[i64],
    ) -> Result<(), HistoryRepositoryError>;
}
