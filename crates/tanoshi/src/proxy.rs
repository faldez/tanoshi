use bytes::Bytes;
use serde::Deserialize;
use std::{convert::Infallible, io::Read};
use warp::{filters::BoxedFilter, hyper::Response, Filter, Reply};

use crate::utils;

#[derive(Deserialize)]
pub struct Image {
    pub url: String,
}

pub fn proxy(secret: String) -> BoxedFilter<(impl Reply,)> {
    warp::path("image")
        .and(warp::path::param())
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

    let content_type = mime_guess::from_path(&url)
        .first_or_octet_stream()
        .to_string();

    let res = match url {
        url if url.starts_with("http") => get_image_from_url(url).await,
        url if !url.is_empty() => get_image_from_file(url).await,
        _ => Err(400),
    };

    match res {
        Ok(bytes) => Ok(warp::http::Response::builder()
            .header("Content-Type", content_type)
            .header("Content-Length", bytes.len())
            .header("Cache-Control", "max-age=315360000")
            .body(bytes)
            .unwrap()),
        Err(status) => Ok(empty_response(status)),
    }
}

pub async fn get_image_from_file(file: String) -> Result<Bytes, u16> {
    let file = std::path::PathBuf::from(file);
    // if file is already a file, serve it
    if file.is_file() {
        match std::fs::read(file) {
            Ok(buf) => Ok(Bytes::from(buf)),
            Err(_) => Err(500),
        }
    } else {
        // else if its combination of archive files and path inside the archive
        // extract the file from archive
        let filename = file.parent().unwrap().to_str().unwrap();
        let path = file.file_name().unwrap().to_str().unwrap();
        match libarchive_rs::extract_archive_file(filename, path) {
            Ok(buf) => Ok(Bytes::from(buf)),
            Err(_) => Err(400),
        }
    }
}

fn empty_response(status: u16) -> Response<Bytes> {
    warp::http::Response::builder()
        .status(status)
        .body(bytes::Bytes::new())
        .unwrap_or_default()
}

pub async fn get_image_from_url(url: String) -> Result<Bytes, u16> {
    debug!("get image from {}", url);
    if url.is_empty() {
        return Err(400);
    }

    let res = match ureq::get(&url).call() {
        Ok(res) => {
            let mut bytes = Vec::new();
            let mut reader = res.into_reader();
            if let Err(_) = reader.read_to_end(&mut bytes) {
                return Err(500);
            }
            bytes
        }
        Err(e) => {
            error!("error fetch image, reason: {}", e);
            return Err(500);
        }
    };

    Ok(Bytes::from(res))
}
