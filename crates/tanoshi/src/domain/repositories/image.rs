use std::path::Path;

use async_trait::async_trait;

use thiserror::Error;

use crate::domain::entities::image::Image;

#[derive(Debug, Error)]
pub enum ImageRepositoryError {
    #[error("error request image: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("other error: {0}")]
    Other(String),
}

#[async_trait]
pub trait ImageRepository {
    async fn fetch_image_from_url(
        &self,
        url: &str,
        referer: Option<&String>,
    ) -> Result<Image, ImageRepositoryError>;
    async fn fetch_image_from_file<P>(&self, path: P) -> Result<Image, ImageRepositoryError>
    where
        P: AsRef<Path> + std::marker::Send;
    async fn fetch_image_from_archive<P>(
        &self,
        archive: P,
        filename: &str,
    ) -> Result<Image, ImageRepositoryError>
    where
        P: AsRef<Path> + std::marker::Send;
}
