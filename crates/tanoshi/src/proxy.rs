use bytes::Bytes;
use serde::Deserialize;
use std::convert::Infallible;
use warp::{filters::BoxedFilter, hyper::Response, Filter, Reply};

use crate::utils;

#[derive(Deserialize)]
pub struct Image {
    pub url: String,
}

pub fn proxy(secret: String) -> BoxedFilter<(impl Reply,)> {
    warp::path!("image" / String)
        .and(warp::get())
        .and(with_secret(secret))
        .and_then(get_image)
        .boxed()
}

fn with_secret(
    secret: String,
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || secret.clone())
}

pub async fn get_image(url: String, secret: String) -> Result<impl warp::Reply, Infallible> {
    debug!("encrypted image url: {}", url);
    let url = match utils::decrypt_url(&secret, &url) {
        Ok(url) => url,
        Err(e) => {
            error!("error validate url: {}", e);
            "".to_string()
        }
    };
    debug!("get image from {}", url);
    match url {
        url if url.starts_with("http") => Ok(get_image_from_url(url).await?),
        url if !url.is_empty() => Ok(get_image_from_file(url).await?),
        _ => Ok(empty_response(400)),
    }
}

pub async fn get_image_from_file(file: String) -> Result<Response<Bytes>, Infallible> {
    let file = std::path::PathBuf::from(file);
    // if file is already a file, serve it
    if file.is_file() {
        match std::fs::read(file) {
            Ok(buf) => Ok(warp::http::Response::builder()
                .status(200)
                .body(Bytes::from(buf))
                .unwrap()),
            Err(_) => Ok(empty_response(500)),
        }
    } else {
        // else if its combination of archive files and path inside the archive
        // extract the file from archive
        let filename = file.parent().unwrap().to_str().unwrap();
        let path = file.file_name().unwrap().to_str().unwrap();
        match libarchive_rs::extract_archive_file(filename, path) {
            Ok(buf) => Ok(warp::http::Response::builder()
                .status(200)
                .body(Bytes::from(buf))
                .unwrap()),
            Err(_) => Ok(empty_response(400)),
        }
    }
}

fn empty_response(status: u16) -> Response<Bytes> {
    warp::http::Response::builder()
        .status(status)
        .body(bytes::Bytes::new())
        .unwrap_or_default()
}

pub async fn get_image_from_url(url: String) -> Result<Response<Bytes>, Infallible> {
    debug!("get image from {}", url);
    if url.is_empty() {
        return Ok(empty_response(400));
    }

    let res = match reqwest::get(&url).await {
        Ok(res) => res,
        Err(e) => {
            error!("error fetch image, reason: {}", e);
            return Ok(empty_response(500));
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

    let bytes = match res.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("error fetch image, reason: {}", e);
            return Ok(empty_response(500));
        }
    };

    match warp::http::Response::builder()
        .header("Content-Type", content_type)
        .header("Content-Length", bytes.len())
        .header("Cache-Control", "max-age=315360000")
        .body(bytes)
    {
        Ok(res) => Ok(res),
        Err(e) => {
            error!("error create response, reason: {}", e);
            return Ok(warp::http::Response::builder()
                .status(500)
                .body(bytes::Bytes::new())
                .unwrap_or_default());
        }
    }
}
