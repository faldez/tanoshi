use crate::domain::entities::manga::Manga;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MangaRepositoryError {
    #[error("database return error: {0}")]
    DbError(#[from] sqlx::Error),
}

#[async_trait]
pub trait MangaRepository: Send + Sync {
    async fn get_manga_by_id(&self, id: i64) -> Result<Manga, MangaRepositoryError>;
    async fn get_manga_by_ids(&self, ids: &[i64]) -> Result<Vec<Manga>, MangaRepositoryError>;
    async fn get_manga_by_source_path(
        &self,
        source_id: i64,
        path: &str,
    ) -> Result<Manga, MangaRepositoryError>;
    async fn insert_manga(&self, manga: &mut Manga) -> Result<(), MangaRepositoryError>;
}
