use anyhow::anyhow;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tanoshi_vm::prelude::ExtensionManager;
use thiserror::Error;

use crate::domain::{
    entities::manga::{InputList, Manga},
    repositories::manga::{MangaRepository, MangaRepositoryError},
};

#[derive(Debug, Error)]
pub enum MangaError {
    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl From<MangaRepositoryError> for MangaError {
    fn from(e: MangaRepositoryError) -> Self {
        match e {
            MangaRepositoryError::DbError(e) => Self::Other(anyhow!("{e}")),
        }
    }
}

pub struct MangaService<R>
where
    R: MangaRepository,
{
    repo: R,
    sources: ExtensionManager,
}

impl<R> MangaService<R>
where
    R: MangaRepository,
{
    pub fn new(repo: R, sources: ExtensionManager) -> Self {
        Self { repo, sources }
    }

    pub async fn fetch_source_popular_manga(
        &self,
        source_id: i64,
        page: i64,
    ) -> Result<Vec<Manga>, MangaError> {
        let fetched_manga = self
            .sources
            .get_popular_manga(source_id, page)
            .await?
            .into_par_iter()
            .map(Manga::from)
            .collect();

        Ok(fetched_manga)
    }

    pub async fn fetch_source_latest_manga(
        &self,
        source_id: i64,
        page: i64,
    ) -> Result<Vec<Manga>, MangaError> {
        let fetched_manga = self
            .sources
            .get_latest_manga(source_id, page)
            .await?
            .into_par_iter()
            .map(Manga::from)
            .collect();

        Ok(fetched_manga)
    }

    pub async fn fetch_source_manga(
        &self,
        source_id: i64,
        page: i64,
        query: Option<String>,
        filters: Option<InputList>,
    ) -> Result<Vec<Manga>, MangaError> {
        let fetched_manga = self
            .sources
            .search_manga(source_id, page, query, filters)
            .await?
            .into_par_iter()
            .map(Manga::from)
            .collect();

        Ok(fetched_manga)
    }

    pub async fn fetch_manga_by_source_path(
        &self,
        source_id: i64,
        path: &str,
    ) -> Result<Manga, MangaError> {
        let manga = if let Ok(manga) = self.repo.get_manga_by_source_path(source_id, path).await {
            manga
        } else {
            let mut manga = self
                .sources
                .get_manga_detail(source_id, path.to_string())
                .await?
                .into();

            self.repo.insert_manga(&mut manga).await?;

            manga
        };

        Ok(manga)
    }

    pub async fn fetch_manga_by_id(&self, id: i64, refresh: bool) -> Result<Manga, MangaError> {
        let mut manga = self.repo.get_manga_by_id(id).await?;
        if refresh {
            let mut m = self
                .sources
                .get_manga_detail(manga.source_id, manga.path)
                .await?
                .into();
            self.repo.insert_manga(&mut m).await?;

            manga = self.repo.get_manga_by_id(id).await?;
        }

        Ok(manga)
    }
}
