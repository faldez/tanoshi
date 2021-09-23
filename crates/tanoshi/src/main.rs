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
mod routes;
mod schema;
mod status;
mod user;
mod utils;
mod worker;

use crate::{
    config::Config,
    notifier::pushover::Pushover,
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
use async_graphql_warp::{BadRequest, Response};
use std::{convert::Infallible, sync::Arc};
use teloxide::prelude::RequesterExt;
use warp::{
    http::{Response as HttpResponse, StatusCode},
    Filter, Rejection,
};

#[derive(Clap)]
struct Opts {
    /// Path to config file
    #[clap(long)]
    config: Option<String>,
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

    let (worker_handle, worker_tx) = worker::start(
        config.update_interval,
        mangadb.clone(),
        userdb.clone(),
        extension_bus.clone(),
        telegram_bot,
        pushover,
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

    let graphql_post = warp::header::optional::<String>("Authorization")
        .and(async_graphql_warp::graphql(schema.clone()))
        .and_then(
            |token: Option<String>,
             (schema, mut request): (TanoshiSchema, async_graphql::Request)| async move {
                if let Some(token) = token {
                    if let Some(token) = token.strip_prefix("Bearer ").map(|t| t.to_string()) {
                        request = request.data(token);
                    }
                }
                let resp = schema.execute(request).await;
                Ok::<_, Infallible>(Response::from(resp))
            },
        );

    let health_check = warp::path!("health").and(warp::get()).map(warp::reply);

    let static_files = assets::filter::static_files();
    let image_proxy = proxy::proxy(config.secret.clone());

    let server_fut = if config.enable_playground {
        info!("enable graphql playground");
        let graphql_playground = warp::path!("graphql").and(warp::get()).map(|| {
            HttpResponse::builder()
                .header("content-type", "text/html")
                .body(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
        });
        bind_routes!(
            config.port,
            health_check,
            image_proxy,
            graphql_playground,
            static_files,
            graphql_post
        )
    } else {
        bind_routes!(
            config.port,
            health_check,
            image_proxy,
            static_files,
            graphql_post
        )
    };

    tokio::select! {
        _ = server_fut => {
            info!("server shutdown");
        }
        _ = worker_handle => {
            info!("worker quit");
        }
        Some(_) = telegram_bot_fut => {
            info!("worker shutdown");
        }
    }

    info!("closing database...");
    pool.close().await;

    Ok(())
}
