use std::path::Path;

use async_trait::async_trait;

use http::{HeaderMap, HeaderValue};

use crate::domain::{
    entities::image::Image,
    repositories::image::{ImageRepository, ImageRepositoryError},
};

#[derive(Default, Clone)]
pub struct ImageRepositoryImpl {
    client: reqwest::Client,
}

impl ImageRepositoryImpl {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ImageRepository for ImageRepositoryImpl {
    async fn fetch_image_from_url(
        &self,
        url: &str,
        referer: Option<&String>,
    ) -> Result<Image, ImageRepositoryError> {
        debug!("get image from {}", url);
        if url.is_empty() {
            return Err(ImageRepositoryError::Other(
                "url cannot be empty".to_string(),
            ));
        }

        let mut headers = HeaderMap::new();

        if let Some(referer) = referer.and_then(|r| r.parse::<HeaderValue>().ok()) {
            headers.insert("Referer", referer);
        }

        let source_res = self.client.get(url).headers(headers).send().await?;

        let content_type = source_res
            .headers()
            .get("content-type")
            .ok_or_else(|| ImageRepositoryError::Other("not a string".to_string()))?
            .to_str()
            .map_err(|_| ImageRepositoryError::Other("no content type".to_string()))?
            .to_string();

        let data = source_res.bytes().await?;

        Ok(Image { content_type, data })
    }

    async fn fetch_image_from_file<P>(&self, path: P) -> Result<Image, ImageRepositoryError>
    where
        P: AsRef<Path> + std::marker::Send,
    {
        let content_type = mime_guess::from_path(&path)
            .first_or_octet_stream()
            .to_string();
        let data = tokio::fs::read(path)
            .await
            .map_err(|e| ImageRepositoryError::Other(format!("{e}")))?;

        Ok(Image {
            content_type,
            data: data.into(),
        })
    }

    async fn fetch_image_from_archive<P>(
        &self,
        archive: P,
        filename: &str,
    ) -> Result<Image, ImageRepositoryError>
    where
        P: AsRef<Path> + std::marker::Send,
    {
        let filename = filename.to_owned();
        let content_type = mime_guess::from_path(&filename)
            .first_or_octet_stream()
            .to_string();

        let source = std::fs::File::open(archive)
            .map_err(|e| ImageRepositoryError::Other(format!("{e}")))?;
        let (content_type, data) =
            tokio::task::spawn_blocking(move || -> Result<(String, Vec<u8>), anyhow::Error> {
                let mut buf: Vec<u8> = vec![];
                compress_tools::uncompress_archive_file(source, &mut buf, &filename)?;

                Ok((content_type, buf))
            })
            .await
            .map_err(|e| ImageRepositoryError::Other(format!("{e}")))?
            .map_err(|e| ImageRepositoryError::Other(format!("{e}")))?;

        Ok(Image {
            content_type,
            data: data.into(),
        })
    }
}
