pub mod filter {
    use super::static_files;
    use warp::{filters::BoxedFilter, Filter, Reply};

    pub fn static_files() -> BoxedFilter<(impl Reply,)> {
        serve().or(serve_index()).boxed()
    }

    fn serve_index() -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(with_encoding())
            .and_then(static_files::serve_index)
            .boxed()
    }

    fn serve() -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .and(warp::path::tail())
            .and(with_encoding())
            .and_then(static_files::serve_tail)
            .boxed()
    }

    fn with_encoding(
    ) -> impl Filter<Extract = (static_files::Encoding,), Error = warp::reject::Rejection> + Clone
    {
        warp::header::header("accept-encoding").map(move |encoding: String| {
            if encoding.is_empty() {
                static_files::Encoding::None
            } else if encoding.contains("br") {
                static_files::Encoding::Br
            } else if encoding.contains("gzip") {
                static_files::Encoding::Gzip
            } else {
                static_files::Encoding::None
            }
        })
    }
}

mod static_files {
    use rust_embed::RustEmbed;
    use warp::http::header::HeaderValue;
    use warp::path::Tail;
    use warp::reply::Response;
    use warp::{Rejection, Reply};

    #[derive(RustEmbed)]
    #[folder = "$CARGO_MANIFEST_DIR/../tanoshi-web/dist"]
    pub struct Asset;

    #[derive(Debug, Clone)]
    pub enum Encoding {
        Br,
        Gzip,
        None,
    }

    pub async fn serve_index(encoding: Encoding) -> Result<impl Reply, Rejection> {
        serve_impl("index.html", encoding)
    }

    pub async fn serve_tail(path: Tail, encoding: Encoding) -> Result<impl Reply, Rejection> {
        serve_impl(path.as_str(), encoding)
    }

    pub fn serve_impl(path: &str, _encoding: Encoding) -> Result<impl Reply, Rejection> {
        let asset = Asset::get(path).ok_or_else(warp::reject::not_found)?;
        let mut res = Response::new(asset.data.into());

        let mime = mime_guess::from_path(path).first_or_octet_stream();
        res.headers_mut().insert(
            "Content-Type",
            HeaderValue::from_str(mime.as_ref()).unwrap(),
        );
        Ok(res)
    }
}
