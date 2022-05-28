use thiserror::Error;

use crate::domain::{
    entities::history::HistoryChapter,
    repositories::history::{HistoryRepository, HistoryRepositoryError},
};

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("repository error: {0}")]
    RepositoryError(#[from] HistoryRepositoryError),
}

pub struct HistoryService<R>
where
    R: HistoryRepository,
{
    repo: R,
}

impl<R> HistoryService<R>
where
    R: HistoryRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn get_history_chapters(
        &self,
        user_id: i64,
        after_timestamp: i64,
        before_timestamp: i64,
        first: Option<usize>,
        last: Option<usize>,
    ) -> Result<Vec<HistoryChapter>, HistoryError> {
        let histories = if let Some(first) = first {
            self.repo
                .get_first_history_chapters(
                    user_id,
                    after_timestamp,
                    before_timestamp,
                    first as i32,
                )
                .await?
        } else if let Some(last) = last {
            self.repo
                .get_last_history_chapters(user_id, after_timestamp, before_timestamp, last as i32)
                .await?
        } else {
            self.repo
                .get_history_chapters(user_id, after_timestamp, before_timestamp)
                .await?
        };

        Ok(histories)
    }

    pub async fn insert_chapter_to_history(
        &self,
        user_id: i64,
        chapter_id: i64,
        page: i64,
        is_complete: bool,
    ) -> Result<(), HistoryError> {
        self.repo
            .insert_history_chapter(user_id, chapter_id, page, is_complete)
            .await?;

        Ok(())
    }

    pub async fn insert_chapters_to_history_as_completed(
        &self,
        user_id: i64,
        chapter_ids: Vec<i64>,
    ) -> Result<(), HistoryError> {
        self.repo
            .insert_history_chapters_as_completed(user_id, &chapter_ids)
            .await?;

        Ok(())
    }

    pub async fn delete_chapters_from_history(
        &self,
        user_id: i64,
        chapter_ids: Vec<i64>,
    ) -> Result<(), HistoryError> {
        self.repo
            .delete_chapters_from_history(user_id, &chapter_ids)
            .await?;

        Ok(())
    }
}
