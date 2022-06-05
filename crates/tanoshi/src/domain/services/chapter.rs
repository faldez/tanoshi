use std::path::PathBuf;

use crate::{
    domain::{
        entities::chapter::Chapter,
        repositories::chapter::{ChapterRepository, ChapterRepositoryError},
    },
    infrastructure::local,
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tanoshi_vm::prelude::SourceBus;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum ChapterError {
    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl From<ChapterRepositoryError> for ChapterError {
    fn from(e: ChapterRepositoryError) -> Self {
        match e {
            ChapterRepositoryError::DbError(e) => Self::Other(anyhow::anyhow!("{e}")),
        }
    }
}

impl From<JoinError> for ChapterError {
    fn from(e: JoinError) -> Self {
        Self::Other(anyhow::anyhow!("{e}"))
    }
}

pub struct ChapterService<R>
where
    R: ChapterRepository,
{
    repo: R,
    sources: SourceBus,
}

impl<R> ChapterService<R>
where
    R: ChapterRepository,
{
    pub fn new(repo: R, sources: SourceBus) -> Self {
        Self { repo, sources }
    }

    pub async fn fetch_chapter_by_id(&self, id: i64) -> Result<Chapter, ChapterError> {
        let chapter = self.repo.get_chapter_by_id(id).await?;

        Ok(chapter)
    }

    pub async fn fetch_chapters_by_manga_id(
        &self,
        source_id: i64,
        path: &str,
        manga_id: i64,
        refresh: bool,
    ) -> Result<Vec<Chapter>, ChapterError> {
        let mut chapters = self
            .repo
            .get_chapters_by_manga_id(manga_id, None, None, false)
            .await
            .unwrap_or_default()
            .into_par_iter()
            .map(|c| c.into())
            .collect::<Vec<Chapter>>();

        if refresh || chapters.is_empty() {
            let source_chapters: Vec<Chapter> = self
                .sources
                .get_chapters(source_id, path.to_string())
                .await?
                .into_par_iter()
                .map(|c| {
                    let mut c: Chapter = c.into();
                    c.manga_id = manga_id;
                    c
                })
                .collect();

            if !source_chapters.is_empty() {
                self.repo.insert_chapters(&source_chapters).await?;
            }

            chapters = self
                .repo
                .get_chapters_by_manga_id(manga_id, None, None, false)
                .await
                .unwrap_or_default()
                .into_par_iter()
                .map(|c| c.into())
                .collect::<Vec<Chapter>>();
        }

        Ok(chapters)
    }

    pub async fn fetch_chapter_pages(
        &self,
        source_id: i64,
        path: &str,
        downloaded_path: &Option<String>,
    ) -> Result<Vec<String>, ChapterError> {
        let pages = if let Some(downloaded_path) =
            downloaded_path.as_ref().map(|p| PathBuf::new().join(p))
        {
            tokio::task::spawn_blocking(move || {
                local::get_pages_from_archive(downloaded_path.as_path())
            })
            .await??
        } else {
            self.sources.get_pages(source_id, path.to_string()).await?
        };

        Ok(pages)
    }

    pub async fn delete_chapter(&self, chapter_id: i64) -> Result<(), ChapterError> {
        self.repo.delete_chapter_by_id(chapter_id).await?;

        Ok(())
    }
}
