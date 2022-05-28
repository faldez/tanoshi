use std::path::{Path, PathBuf};

use crate::{
    application::worker::downloads::{Command as DownloadCommand, DownloadSender},
    domain::{
        entities::download::{DownloadQueueEntry, DownloadedChapter},
        repositories::download::{DownloadRepository, DownloadRepositoryError},
    },
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("repository error: {0}")]
    RepositoryError(#[from] DownloadRepositoryError),
    #[error("other error: {0}")]
    OtherError(#[from] anyhow::Error),
}

pub struct DownloadService<R>
where
    R: DownloadRepository,
{
    repo: R,
    download_sender: DownloadSender,
}

impl<R> DownloadService<R>
where
    R: DownloadRepository,
{
    pub fn new(repo: R, download_sender: DownloadSender) -> Self {
        Self {
            repo,
            download_sender,
        }
    }

    pub async fn get_downloaded_chapters(
        &self,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        first: Option<usize>,
        last: Option<usize>,
    ) -> Result<Vec<DownloadedChapter>, DownloadError> {
        let chapters = if let Some(first) = first {
            self.repo
                .get_first_downloaded_chapters(
                    after_timestamp,
                    after_id,
                    before_timestamp,
                    before_id,
                    first as i32,
                )
                .await?
        } else if let Some(last) = last {
            self.repo
                .get_last_downloaded_chapters(
                    after_timestamp,
                    after_id,
                    before_timestamp,
                    before_id,
                    last as i32,
                )
                .await?
        } else {
            self.repo
                .get_downloaded_chapters(after_timestamp, after_id, before_timestamp, before_id)
                .await?
        };

        Ok(chapters)
    }

    pub fn get_download_status<P: AsRef<Path>>(&self, download_path: P) -> bool {
        !PathBuf::new().join(download_path).join(".pause").exists()
    }

    pub async fn change_download_status<P: AsRef<Path>>(
        &self,
        download_path: P,
        status: bool,
    ) -> Result<(), DownloadError> {
        let pause_path = PathBuf::new().join(download_path).join(".pause");

        if status {
            let _ = tokio::fs::remove_file(pause_path).await;

            self.download_sender
                .send(DownloadCommand::Download)
                .map_err(|_| {
                    DownloadError::OtherError(anyhow::anyhow!("failed to send download command"))
                })?;
        } else {
            let _ = tokio::fs::write(pause_path, b"").await;
        }

        Ok(())
    }

    pub async fn get_download_queue(&self) -> Result<Vec<DownloadQueueEntry>, DownloadError> {
        let queue = self.repo.get_download_queue().await?;

        Ok(queue)
    }

    pub async fn download_chapters(&self, chapter_ids: Vec<i64>) -> Result<(), DownloadError> {
        for chapter_id in chapter_ids {
            self.download_sender
                .send(DownloadCommand::InsertIntoQueue(chapter_id))
                .map_err(|_| {
                    DownloadError::OtherError(anyhow::anyhow!("failed to send download queue"))
                })?;
        }

        self.download_sender
            .send(DownloadCommand::Download)
            .map_err(|_| {
                DownloadError::OtherError(anyhow::anyhow!("failed to send download command"))
            })?;

        Ok(())
    }

    pub async fn update_chapter_priority(
        &self,
        chapter_id: i64,
        priority: i64,
    ) -> Result<(), DownloadError> {
        self.repo
            .update_download_queue_priority(chapter_id, priority)
            .await?;

        Ok(())
    }

    pub async fn remove_chapters_from_queue(
        &self,
        chapter_ids: Vec<i64>,
    ) -> Result<(), DownloadError> {
        for chapter_id in chapter_ids {
            self.repo
                .delete_download_queue_by_chapter_id(chapter_id)
                .await?;
        }

        Ok(())
    }

    pub async fn remove_downloaded_chapters(
        &self,
        chapter_ids: Vec<i64>,
    ) -> Result<(), DownloadError> {
        for chapter_id in chapter_ids {
            if let Ok(downloaded_path) = self.repo.get_chapter_downloaded_path(chapter_id).await {
                if let Err(e) = tokio::fs::remove_file(&downloaded_path).await {
                    error!("error removing file: {e}");
                }
            }

            self.repo
                .update_chapter_downloaded_path(chapter_id, None)
                .await?;
        }

        Ok(())
    }
}
