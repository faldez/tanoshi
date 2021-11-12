use crate::{ config::Config, proxy::Proxy, schema::TanoshiSchema};

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{async_trait, handler::get};
use axum::{
    extract::{Extension, FromRequest, RequestParts, TypedHeader},
    routing::BoxRoute,
};
use axum::{
    handler::post,
    response::{self, IntoResponse},
};
use axum::{AddExtensionLayer, Router, Server};
use headers::{authorization::Bearer, Authorization};
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

struct Token(String);

#[async_trait]
impl<B> FromRequest<B> for Token
where
    B: Send,
{
    type Rejection = ();

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let token = TypedHeader::<Authorization<Bearer>>::from_request(req)
            .await
            .map(|TypedHeader(Authorization(bearer))| Token(bearer.token().to_string()))
            .unwrap_or_else(|_| Token("".to_string()));

        Ok(token)
    }
}

async fn graphql_handler(
    token: Token,
    schema: Extension<TanoshiSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let req = req.into_inner();
    let req = req.data(token.0);
    schema.execute(req).await.into()
}

#[allow(dead_code)]
async fn graphql_playground() -> impl IntoResponse {
    response::Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

async fn health_check() -> impl IntoResponse {
    response::Html("OK")
}

fn init_app(config: &Config, schema: TanoshiSchema) -> Router<BoxRoute> {
    let proxy = Proxy::new(config.secret.clone());

    let mut app = Router::new().boxed();

    #[cfg(feature = "embed")]
    {
        app = app.nest("/", get(crate::assets::static_handler)).boxed();
    }

    app = app
        .route("/image/:url", get(Proxy::proxy))
        .route("/health", get(health_check))
        .layer(AddExtensionLayer::new(proxy))
        .boxed();
    if config.enable_playground {
        app = app
            .nest("/graphql", get(graphql_playground).post(graphql_handler))
            .layer(AddExtensionLayer::new(schema))
            .boxed();
    } else {
        app = app
            .nest("/graphql", post(graphql_handler))
            .layer(AddExtensionLayer::new(schema))
            .boxed();
    }

    app
}

pub async fn serve<T>(config: &Config, schema: TanoshiSchema) -> Result<(), anyhow::Error> {
    let app = init_app(config, schema);

    let addr = SocketAddr::from((IpAddr::from_str("0.0.0.0")?, config.port));
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}
