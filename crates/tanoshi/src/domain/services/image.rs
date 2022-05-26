use crate::domain::{
    entities::image::{Image, ImageUri},
    repositories::image::{ImageRepository, ImageRepositoryError},
};
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("error request image")]
    RequestError,
    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl From<ImageRepositoryError> for ImageError {
    fn from(e: ImageRepositoryError) -> Self {
        match e {
            ImageRepositoryError::RequestError(_) => Self::RequestError,
            ImageRepositoryError::Other(msg) => Self::Other(anyhow::anyhow!("{msg}")),
        }
    }
}

#[derive(Clone)]
pub struct ImageService<R>
where
    R: ImageRepository,
{
    repo: R,
}

impl<R> ImageService<R>
where
    R: ImageRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn fetch_image(
        &self,
        secret: &str,
        encrypted_url: &str,
        referer: Option<&String>,
    ) -> Result<Image, ImageError> {
        let uri = ImageUri::from_encrypted(secret, encrypted_url)
            .map_err(|e| ImageError::Other(anyhow::anyhow!("{e}")))?;

        let image = match uri {
            ImageUri::Remote(url) => self.repo.fetch_image_from_url(&url, referer).await?,
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
