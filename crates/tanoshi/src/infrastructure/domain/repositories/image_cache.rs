use async_trait::async_trait;
use std::path::{Path, PathBuf};

use crate::domain::{
    entities::image::Image,
    repositories::image_cache::{ImageCacheRepository, ImageCacheRepositoryError},
};

#[derive(Clone)]
pub struct ImageCacheRepositoryImpl {
    path: PathBuf,
}

impl ImageCacheRepositoryImpl {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: PathBuf::new().join(path),
        }
    }
}

#[async_trait]
impl ImageCacheRepository for ImageCacheRepositoryImpl {
    async fn set(&self, key: &str, image: &Image) -> Result<(), ImageCacheRepositoryError> {
        let path = self.path.join(key);

        let encoded = bincode::serialize(&image)?;

        tokio::fs::write(&path, &encoded).await?;

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Image, ImageCacheRepositoryError> {
        let path = self.path.join(key);

        let encoded = tokio::fs::read(path).await?;

        let decoded = bincode::deserialize(&encoded)?;

        Ok(decoded)
    }
}
