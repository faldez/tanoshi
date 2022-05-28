use async_trait::async_trait;

use thiserror::Error;

use crate::domain::entities::download::{DownloadQueue, DownloadQueueEntry, DownloadedChapter};

#[derive(Debug, Error)]
pub enum DownloadRepositoryError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
}

#[async_trait]
pub trait DownloadRepository {
    async fn get_first_downloaded_chapters(
        &self,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        first: i32,
    ) -> Result<Vec<DownloadedChapter>, DownloadRepositoryError>;

    async fn get_last_downloaded_chapters(
        &self,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        last: i32,
    ) -> Result<Vec<DownloadedChapter>, DownloadRepositoryError>;

    async fn get_downloaded_chapters(
        &self,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
    ) -> Result<Vec<DownloadedChapter>, DownloadRepositoryError>;

    async fn get_chapter_downloaded_path(
        &self,
        chapter_id: i64,
    ) -> Result<String, DownloadRepositoryError>;

    async fn update_chapter_downloaded_path(
        &self,
        chapter_id: i64,
        path: Option<String>,
    ) -> Result<(), DownloadRepositoryError>;

    async fn insert_download_queue(
        &self,
        items: &[DownloadQueue],
    ) -> Result<(), DownloadRepositoryError>;

    async fn get_single_download_queue(
        &self,
    ) -> Result<Option<DownloadQueue>, DownloadRepositoryError>;

    async fn get_single_chapter_download_status(
        &self,
        chapter_id: i64,
    ) -> Result<bool, DownloadRepositoryError>;

    async fn mark_single_download_queue_as_completed(
        &self,
        id: i64,
    ) -> Result<(), DownloadRepositoryError>;

    async fn get_download_queue_last_priority(
        &self,
    ) -> Result<Option<i64>, DownloadRepositoryError>;

    async fn get_download_queue(&self) -> Result<Vec<DownloadQueueEntry>, DownloadRepositoryError>;

    async fn delete_single_chapter_download_queue(
        &self,
        chapter_id: i64,
    ) -> Result<(), DownloadRepositoryError>;

    async fn delete_download_queue_by_chapter_id(
        &self,
        id: i64,
    ) -> Result<(), DownloadRepositoryError>;

    async fn update_download_queue_priority(
        &self,
        chapter_id: i64,
        priority: i64,
    ) -> Result<(), DownloadRepositoryError>;
}
