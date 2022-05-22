pub mod catalogue;
pub mod downloads;
pub mod guard;
pub mod library;
pub mod loader;
pub mod local;
pub mod notification;
pub mod schema;
pub mod status;
pub mod tracking;
pub mod user;

use crate::infrastructure::auth;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{self, IntoResponse},
};

use self::schema::TanoshiSchema;

use super::token::Token;

pub async fn graphql_handler(
    token: Token,
    schema: Extension<TanoshiSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();

    if let Ok(claims) = auth::decode_jwt(&token.0) {
        req = req.data(claims);
    }

    schema.execute(req).await.into()
}

pub async fn graphql_playground() -> impl IntoResponse {
    response::Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}
