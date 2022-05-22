use axum::response;

pub async fn health_check() -> impl response::IntoResponse {
    response::Html("OK")
}
