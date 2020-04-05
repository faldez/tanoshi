use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use serde_json::json;
use warp::Filter;

use crate::auth::auth::Auth;
use crate::auth::Claims;
use crate::handlers::auth as auth_handler;

pub mod auth;
pub mod favorites;
pub mod history;
pub mod manga;
pub mod updates;

#[derive(Debug)]
pub struct ExpiredOrInvalidToken;

impl warp::reject::Reject for ExpiredOrInvalidToken {}

pub fn with_db(
    db: Arc<Mutex<Connection>>,
) -> impl Filter<Extract = (Arc<Mutex<Connection>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub fn with_authorization(
    secret: String,
) -> impl Filter<Extract = (Claims,), Error = warp::reject::Rejection> + Clone {
    warp::header::header("authorization")
        .map(move |token: String| auth_handler::validate(secret.clone(), token.to_string()))
        .and_then(|claim: Option<Claims>| async move {
            match claim {
                Some(claim) => Ok(claim),
                None => Err(warp::reject::custom(ExpiredOrInvalidToken)),
            }
        })
}

pub async fn handle_rejection(
    err: warp::reject::Rejection,
) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = warp::http::StatusCode::NOT_FOUND;
        message = "Resource not found";
    } else if let Some(ExpiredOrInvalidToken) = err.find() {
        code = warp::http::StatusCode::UNAUTHORIZED;
        message = "Unauthorized";
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        message = "Unhandled";
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&json!({ "message": message })),
        code,
    ))
}
