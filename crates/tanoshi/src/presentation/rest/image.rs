use axum::{
    body::Body,
    extract::{Extension, Path, Query},
    http::{Response, StatusCode},
    response::IntoResponse,
};

use serde::Deserialize;

use crate::{
    domain::services::image::ImageService,
    infrastructure::{
        config::Config,
        domain::repositories::{image::ImageRepositoryImpl, image_cache::ImageCacheRepositoryImpl},
    },
};

#[derive(Debug, Deserialize)]
pub struct Params {
    referer: Option<String>,
}

pub async fn fetch_image(
    Path(encrypted_url): Path<String>,
    Query(params): Query<Params>,
    Extension(config): Extension<Config>,
    Extension(svc): Extension<ImageService<ImageCacheRepositoryImpl, ImageRepositoryImpl>>,
) -> Result<impl IntoResponse, StatusCode> {
    let image = svc
        .fetch_image(&config.secret, &encrypted_url, params.referer.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Response::builder()
        .header("Content-Type", image.content_type)
        .header("Content-Length", image.data.len())
        .header("Cache-Control", "max-age=864000")
        .body(Body::from(image.data))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}
