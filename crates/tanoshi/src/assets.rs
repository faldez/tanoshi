use std::convert::Infallible;

use axum::{
    body::{Body, Bytes, Full},
    http::{header, Response, StatusCode},
    response::IntoResponse,
};

use http::Request;
use mime_guess;
use rust_embed::RustEmbed;

// static_handler is a handler that serves static files from the
pub async fn static_handler(req: Request<Body>) -> impl IntoResponse {
    let path = req.uri().path().trim_start_matches('/').to_string();

    let asset = Asset::get(path.as_str());
    let accept = req.headers().get("accept").and_then(|v| v.to_str().ok());
    match (asset, accept) {
        (None, Some(header)) if header.contains("*/*") || header.contains("text/html") => {
            StaticFile("index.html".to_string())
        }
        _ => StaticFile(path),
    }
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
