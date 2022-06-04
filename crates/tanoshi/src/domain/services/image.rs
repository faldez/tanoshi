use crate::domain::{
    entities::image::{Image, ImageUri},
    repositories::{
        image::{ImageRepository, ImageRepositoryError},
        image_cache::{ImageCacheRepository, ImageCacheRepositoryError},
    },
};
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("error request image")]
    RequestError,
    #[error("repository error: {0}")]
    RepositoryError(#[from] ImageRepositoryError),
    #[error("cache error: {0}")]
    CacheError(#[from] ImageCacheRepositoryError),
    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct ImageService<C, R>
where
    C: ImageCacheRepository,
    R: ImageRepository,
{
    repo: R,
    cache_repo: C,
}

impl<C, R> ImageService<C, R>
where
    C: ImageCacheRepository,
    R: ImageRepository,
{
    pub fn new(repo: R, cache_repo: C) -> Self {
        Self { repo, cache_repo }
    }

    pub async fn fetch_image(
        &self,
        secret: &str,
        encrypted_url: &str,
        referer: Option<&String>,
    ) -> Result<Image, ImageError> {
        if let Ok(image) = self.cache_repo.get(encrypted_url).await {
            return Ok(image);
        }

        let uri = ImageUri::from_encrypted(secret, encrypted_url)
            .map_err(|e| ImageError::Other(anyhow::anyhow!("{e}")))?;

        let image = match uri {
            ImageUri::Remote(url) => {
                let image = self.repo.fetch_image_from_url(&url, referer).await?;
                if let Err(e) = self.cache_repo.set(encrypted_url, &image).await {
                    error!("error cache image {encrypted_url}: {e}");
                }

                image
            }
            ImageUri::File(path) => self.repo.fetch_image_from_file(&path).await?,
            ImageUri::Archive(archive, filename) => {
                self.repo
                    .fetch_image_from_archive(&archive, &filename)
                    .await?
            }
        };

        Ok(image)
    }

    pub fn encrypt_image_url(&self, secret: &str, url: &str) -> Result<String, ImageError> {
        let image_uri = ImageUri::try_from(url.to_string())?;

        Ok(image_uri.into_encrypted(secret)?)
    }
}
