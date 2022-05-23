use crate::domain::{
    entities::image::{Image, ImageUri},
    repositories::image::{ImageRepository, ImageRepositoryError},
};

pub enum ImageError {
    RequestError,
    Other(String),
}

impl From<ImageRepositoryError> for ImageError {
    fn from(e: ImageRepositoryError) -> Self {
        match e {
            ImageRepositoryError::RequestError(_) => Self::RequestError,
            ImageRepositoryError::Other(msg) => Self::Other(msg),
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
            .map_err(|e| ImageError::Other(format!("{e}")))?;

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
}
