use async_trait::async_trait;
use thiserror::Error;

use crate::domain::entities::image::Image;

#[derive(Debug, Error)]
pub enum ImageCacheRepositoryError {
    #[error("io error: {0}")]
    FileError(#[from] std::io::Error),
    #[error("io error: {0}")]
    SerializeError(#[from] bincode::Error),
    #[error("other error: {0}")]
    Other(String),
}

#[async_trait]
pub trait ImageCacheRepository {
    async fn set(&self, key: &str, image: &Image) -> Result<(), ImageCacheRepositoryError>;

    async fn get(&self, key: &str) -> Result<Image, ImageCacheRepositoryError>;
}
