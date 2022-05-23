use axum::{
    body::Body,
    extract::{Extension, Path, Query},
    http::{Response, StatusCode},
    response::IntoResponse,
};

use serde::Deserialize;

use crate::{
    domain::services::image::ImageService,
    infrastructure::{config::GLOBAL_CONFIG, repositories::image::ImageRepositoryImpl},
};

#[derive(Debug, Deserialize)]
pub struct Params {
    referer: Option<String>,
}

pub async fn fetch_image(
    Path(encrypted_url): Path<String>,
    Query(params): Query<Params>,
    Extension(svc): Extension<ImageService<ImageRepositoryImpl>>,
) -> Result<impl IntoResponse, StatusCode> {
    let secret = GLOBAL_CONFIG
        .get()
        .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR)?
        .secret
        .to_owned();

    let image = svc
        .fetch_image(&secret, &encrypted_url, params.referer.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Response::builder()
        .header("Content-Type", image.content_type)
        .header("Content-Length", image.data.len())
        .body(Body::from(image.data))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}
