#[cfg(feature = "server")]
use axum::{
    body::Body,
    extract::{Extension, Path, Query},
    http::{HeaderMap, HeaderValue, Response, StatusCode},
    response::IntoResponse,
};
use fancy_regex::Regex;
use serde::Deserialize;

use crate::infrastructure::utils;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Params {
    referer: Option<String>,
}

#[derive(Clone)]
pub struct Image {
    client: reqwest::Client,
    secret: String,
}

impl Image {
    pub fn new(secret: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            secret,
        }
    }

    pub async fn image(
        headers: HeaderMap,
        Path(url): Path<String>,
        Query(params): Query<Params>,
        state: Extension<Self>,
    ) -> impl IntoResponse {
        debug!("encrypted image url: {}", url);
        let url = match utils::decrypt_url(&state.secret, &url) {
            Ok(url) => url,
            Err(e) => {
                error!("error validate url: {}", e);
                "".to_string()
            }
        };

        debug!("get image from {}", url);
        let res: Response<Body> = match state.get_image(headers, &url, params).await {
            Ok(body) => body,
            Err(status) => http::Response::builder()
                .status(status)
                .body(Body::empty())
                .unwrap(),
        };

        res
    }

    #[cfg(feature = "server")]
    pub async fn get_image(
        &self,
        headers: HeaderMap,
        url: &str,
        params: Params,
    ) -> Result<http::Response<Body>, StatusCode> {
        match url {
            url if url.starts_with("http") => {
                Ok(self.get_image_from_url_stream(headers, url, params).await?)
            }
            url if !url.is_empty() => {
                let (content_type, data) = self
                    .get_image_from_file(url)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                Ok(http::Response::builder()
                    .header("Content-Type", content_type)
                    .body(Body::from(data))
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
            }
            _ => Err(StatusCode::BAD_REQUEST),
        }
    }

    async fn get_image_from_file(&self, file: &str) -> Result<(String, Vec<u8>), anyhow::Error> {
        let path = std::path::PathBuf::from(file);

        // if file is already a file, serve it
        if path.is_file() {
            let content_type = mime_guess::from_path(&file)
                .first_or_octet_stream()
                .to_string();
            let data = tokio::fs::read(file).await?;
            Ok((content_type, data))
        } else {
            // else if its combination of archive files and path inside the archive
            // extract the file from archive
            let full_path = file.to_string();
            let re = Regex::new(r#"\.(cbz|cbr)[\/|\\]"#)?;

            if let Some(matches) = re.find(file)? {
                let archive_path = full_path[0..matches.end() - 1].to_owned();
                let filename = full_path[matches.end()..full_path.len()].to_owned();

                debug!("{archive_path}\t{filename}");

                let content_type = mime_guess::from_path(&path)
                    .first_or_octet_stream()
                    .to_string();

                tokio::task::spawn_blocking(move || -> Result<(String, Vec<u8>), anyhow::Error> {
                    let source = std::fs::File::open(archive_path)?;

                    let mut buf: Vec<u8> = vec![];
                    compress_tools::uncompress_archive_file(source, &mut buf, &filename)?;

                    Ok((content_type, buf))
                })
                .await?
            } else {
                Err(anyhow::anyhow!("invalid file url"))
            }
        }
    }

    #[cfg(feature = "server")]
    async fn get_image_from_url_stream(
        &self,
        mut headers: HeaderMap,
        url: &str,
        params: Params,
    ) -> Result<http::Response<Body>, StatusCode> {
        debug!("get image from {}", url);
        if url.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        headers.remove("host");
        if let Some(referer) = params.referer.and_then(|r| r.parse::<HeaderValue>().ok()) {
            headers.insert("referer", referer);
        } else {
            headers.remove("referer");
        }

        let source_res = match self.client.get(url).headers(headers).send().await {
            Ok(res) => res,
            Err(e) => {
                error!("error fetch image, reason: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let mut res = http::Response::builder().status(source_res.status());

        for (name, value) in source_res.headers() {
            res = res.header(name, value);
        }

        Ok(res
            .body(Body::wrap_stream(source_res.bytes_stream()))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
    }

    #[allow(dead_code)]
    async fn get_image_from_url(&self, url: &str) -> Result<(String, Vec<u8>), anyhow::Error> {
        debug!("get image from {}", url);
        if url.is_empty() {
            return Err(anyhow::anyhow!("url is empty"));
        }

        let data = self.client.get(url).send().await?.bytes().await?.to_vec();

        let content_type = mime_guess::from_path(url)
            .first_or_octet_stream()
            .to_string();

        Ok((content_type, data))
    }

    #[allow(dead_code)]
    pub async fn get_image_raw(&self, url: &str) -> Result<(String, Vec<u8>), anyhow::Error> {
        let url = match utils::decrypt_url(&self.secret, url) {
            Ok(url) => url,
            Err(e) => {
                error!("error validate url: {}", e);
                "".to_string()
            }
        };

        match url {
            url if url.starts_with("http") => Ok(self.get_image_from_url(&url).await?),
            url if !url.is_empty() => Ok(self.get_image_from_file(&url).await?),
            _ => Err(anyhow::anyhow!("failed to get image")),
        }
    }
}
