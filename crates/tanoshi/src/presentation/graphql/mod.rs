pub mod catalogue;
pub mod categories;
pub mod chapter;
pub mod common;
pub mod downloads;
pub mod guard;
pub mod library;
pub mod loader;
pub mod manga;
pub mod notification;
pub mod recent;
pub mod schema;
pub mod source;
pub mod status;
pub mod tracking;
pub mod user;

use crate::infrastructure::{auth, config::Config};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig, ALL_WEBSOCKET_PROTOCOLS};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    extract::{Extension, WebSocketUpgrade},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

use self::schema::TanoshiSchema;

use super::token::Token;

pub async fn graphql_handler(
    token: Token,
    Extension(config): Extension<Config>,
    Extension(schema): Extension<TanoshiSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();

    if let Ok(claims) = auth::decode_jwt(&config.secret, &token.0) {
        req = req.data(claims);
    }

    schema.execute(req).await.into()
}

pub async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/ws"),
    ))
}

pub async fn graphql_ws_handler(
    Extension(config): Extension<Config>,
    Extension(schema): Extension<TanoshiSchema>,
    protocol: GraphQLProtocol,
    websocket: WebSocketUpgrade,
) -> Response {
    websocket
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema.clone(), protocol)
                .on_connection_init(|value| async move {
                    #[derive(Deserialize)]
                    struct Payload {
                        token: String,
                    }

                    if let Ok(payload) = serde_json::from_value::<Payload>(value) {
                        let mut data = async_graphql::Data::default();
                        if let Ok(claims) = auth::decode_jwt(&config.secret, &payload.token) {
                            data.insert(claims);
                        }
                        Ok(data)
                    } else {
                        Err("Token is required".into())
                    }
                })
                .serve()
        })
}
