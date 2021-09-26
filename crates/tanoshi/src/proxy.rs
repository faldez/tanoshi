use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Extension, Path},
    http::{Response, StatusCode},
    response::IntoResponse,
};

use crate::utils;

pub struct Proxy {
    client: reqwest::Client,
    secret: String,
}

impl Proxy {
    pub fn new(secret: String) -> Arc<Self> {
        Arc::new(Self {
            client: reqwest::Client::new(),
            secret,
        })
    }

    pub async fn proxy(Path(url): Path<String>, state: Extension<Arc<Self>>) -> impl IntoResponse {
        debug!("encrypted image url: {}", url);
        let url = match utils::decrypt_url(&state.as_ref().secret, &url) {
            Ok(url) => url,
            Err(e) => {
                error!("error validate url: {}", e);
                "".to_string()
            }
        };
        debug!("get image from {}", url);
        let res: Response<Body> = match state.as_ref().get_image(&url).await {
            Ok(body) => body,
            Err(status) => http::Response::builder()
                .status(status)
                .body(Body::empty())
                .unwrap(),
        };

        res
    }

    pub async fn get_image(&self, url: &str) -> Result<http::Response<Body>, StatusCode> {
        match url {
            url if url.starts_with("http") => Ok(self.get_image_from_url(url).await?),
            url if !url.is_empty() => Ok(self.get_image_from_file(url).await?),
            _ => Err(StatusCode::BAD_REQUEST),
        }
    }

    async fn get_image_from_file(&self, file: &str) -> Result<http::Response<Body>, StatusCode> {
        let file = std::path::PathBuf::from(file);

        // if file is already a file, serve it
        if file.is_file() {
            let content_type = mime_guess::from_path(&file)
                .first_or_octet_stream()
                .to_string();
            match std::fs::read(file) {
                Ok(buf) => Ok(http::Response::builder()
                    .header("Content-Type", content_type)
                    .body(Body::from(buf))
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        } else {
            // else if its combination of archive files and path inside the archive
            // extract the file from archive
            let filename = file.parent().unwrap().to_str().unwrap();
            let path = file.file_name().unwrap().to_str().unwrap();
            let content_type = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            match libarchive_rs::extract_archive_file(filename, path) {
                Ok(buf) => Ok(http::Response::builder()
                    .header("Content-Type", content_type)
                    .body(Body::from(buf))
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?),
                Err(_) => Err(StatusCode::BAD_REQUEST),
            }
        }
    }

    async fn get_image_from_url(&self, url: &str) -> Result<http::Response<Body>, StatusCode> {
        debug!("get image from {}", url);
        if url.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        let source_res = match self.client.get(url).send().await {
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
}
