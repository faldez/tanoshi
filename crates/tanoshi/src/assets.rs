use std::convert::Infallible;

use axum::{
    body::{Bytes, Full},
    handler::get,
    http::{header, Response, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::Router,
};

use mime_guess;
use rust_embed::RustEmbed;

pub async fn index_handler() -> impl IntoResponse {
    static_handler("/index.html".parse::<Uri>().unwrap()).await
}

// static_handler is a handler that serves static files from the
pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    StaticFile(path)
}

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../tanoshi-web/dist"]
struct Asset;
pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        let path = self.0.into();
        match Asset::get(path.as_str()) {
            Some(content) => {
                let body = content.data.into();
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::from("404"))
                .unwrap(),
        }
    }
}
