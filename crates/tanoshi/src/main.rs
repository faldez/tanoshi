#[macro_use]
extern crate log;
extern crate argon2;

mod assets;
mod catalogue;
mod config;
mod db;
mod library;
mod local;
mod notifier;
mod proxy;
mod schema;
mod status;
mod user;
mod utils;
mod worker;

use crate::{
    config::Config,
    notifier::pushover::Pushover,
    proxy::Proxy,
    schema::{MutationRoot, QueryRoot, TanoshiSchema},
    user::Secret,
};
use clap::Clap;
use futures::future::OptionFuture;
use tanoshi_vm::{bus::ExtensionBus, vm};

use async_graphql::{
    // extensions::ApolloTracing,
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription,
    Schema,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::{Extension, FromRequest, RequestParts, TypedHeader};
use axum::{async_trait, handler::get};
use axum::{
    handler::post,
    response::{self, IntoResponse},
};
use axum::{AddExtensionLayer, Router, Server};
use headers::{authorization::Bearer, Authorization};
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
};
use teloxide::prelude::RequesterExt;

#[derive(Clap)]
struct Opts {
    /// Path to config file
    #[clap(long)]
    config: Option<String>,
}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        info!("rust_log: {}", rust_log);
    } else if let Ok(tanoshi_log) = std::env::var("TANOSHI_LOG") {
        info!("tanoshi_log: {}", tanoshi_log);
        std::env::set_var(
            "RUST_LOG",
            format!("tanoshi={},tanoshi_vm={}", tanoshi_log, tanoshi_log),
        );
    }

    env_logger::init();

    let opts: Opts = Opts::parse();
    let config = Config::open(opts.config)?;

    let pool = db::establish_connection(&config.database_path).await?;
    let mangadb = db::MangaDatabase::new(pool.clone());
    let userdb = db::UserDatabase::new(pool.clone());

    let (_, extension_tx) = vm::start(&config.plugin_path);
    vm::load(&config.plugin_path, extension_tx.clone()).await?;

    let extension_bus = ExtensionBus::new(&config.plugin_path, extension_tx);

    extension_bus
        .insert(local::ID, Arc::new(local::Local::new(config.local_path)))
        .await?;

    let mut telegram_bot = None;
    let mut telegram_bot_fut: OptionFuture<_> = None.into();
    if let Some(telegram_config) = config.telegram {
        let bot = teloxide::Bot::new(telegram_config.token)
            .auto_send()
            .parse_mode(teloxide::types::ParseMode::Html);
        telegram_bot_fut = Some(notifier::telegram::run(telegram_config.name, bot.clone())).into();
        telegram_bot = Some(bot);
    }

    let pushover = config
        .pushover
        .map(|pushover_cfg| Pushover::new(pushover_cfg.application_key));

    let (worker_handle, worker_tx) = worker::worker::start(telegram_bot, pushover);

    let update_worker_handle = worker::updates::start(
        config.update_interval,
        userdb.clone(),
        mangadb.clone(),
        extension_bus.clone(),
        worker_tx.clone(),
    );

    let schema: TanoshiSchema = Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription::default(),
    )
    // .extension(ApolloTracing)
    .data(userdb)
    .data(mangadb)
    .data(Secret(config.secret.clone()))
    .data(extension_bus)
    .data(worker_tx)
    .finish();

    let proxy = Proxy::new(config.secret.clone());

    let mut app = Router::new()
        .nest("/", get(assets::static_handler))
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

    let addr = SocketAddr::from((IpAddr::from_str("::0")?, config.port));
    let server_fut = Server::bind(&addr).serve(app.into_make_service());

    tokio::select! {
        _ = server_fut => {
            info!("server shutdown");
        }
        _ = worker_handle => {
            info!("worker quit");
        }
        _ = update_worker_handle => {
            info!("update worker quit");
        }
        Some(_) = telegram_bot_fut => {
            info!("worker shutdown");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("ctrl+c signal");
        }
    }

    info!("closing database...");
    pool.close().await;

    Ok(())
}
