use axum::{
    body::{Bytes, Full},
    extract::{Extension, Path},
    handler::get,
    http::{header, Response, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::Router,
};
use serde::Deserialize;
use std::convert::Infallible;

use crate::{utils, State};

pub async fn proxy(Path(url): Path<String>, state: Extension<State>) -> impl IntoResponse {
    debug!("encrypted image url: {}", url);
    let url = match utils::decrypt_url(&state.secret, &url) {
        Ok(url) => url,
        Err(e) => {
            error!("error validate url: {}", e);
            "".to_string()
        }
    };
    debug!("get image from {}", url);
    let res: Response<Full<Bytes>> = match get_image(&url).await {
        Ok(body) => {
            let mime = mime_guess::from_ext(&url).first_or_octet_stream();
            let body = body.into();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(body)
                .unwrap()
        }
        Err(status) => Response::builder()
            .status(status)
            .body(Full::from("Error"))
            .unwrap(),
    };

    res
}

pub async fn get_image(url: &str) -> Result<Bytes, StatusCode> {
    match url {
        url if url.starts_with("http") => Ok(get_image_from_url(url).await?),
        url if !url.is_empty() => Ok(get_image_from_file(url).await?),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn get_image_from_file(file: &str) -> Result<Bytes, StatusCode> {
    let file = std::path::PathBuf::from(file);
    // if file is already a file, serve it
    if file.is_file() {
        match std::fs::read(file) {
            Ok(buf) => Ok(Bytes::from(buf)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        // else if its combination of archive files and path inside the archive
        // extract the file from archive
        let filename = file.parent().unwrap().to_str().unwrap();
        let path = file.file_name().unwrap().to_str().unwrap();
        match libarchive_rs::extract_archive_file(filename, path) {
            Ok(buf) => Ok(Bytes::from(buf)),
            Err(_) => Err(StatusCode::BAD_REQUEST),
        }
    }
}

pub async fn get_image_from_url(url: &str) -> Result<Bytes, StatusCode> {
    debug!("get image from {}", url);
    if url.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let res = match reqwest::get(url).await {
        Ok(res) => res,
        Err(e) => {
            error!("error fetch image, reason: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let content_type = res
        .headers()
        .into_iter()
        .find_map(|(header_name, header_value)| {
            if header_name.to_string().to_lowercase().eq("content-type") {
                header_value.to_str().ok()
            } else {
                None
            }
        });

    let content_type = match content_type {
        Some(content_type) => content_type.to_string(),
        None => match url.split('.').rev().take(1).next() {
            Some(ext) => ["image", ext].join("/"),
            None => "application/octet-stream".to_string(),
        },
    };

    match res.bytes().await {
        Ok(bytes) => Ok(bytes),
        Err(e) => {
            error!("error fetch image, reason: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
