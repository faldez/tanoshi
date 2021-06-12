extern crate libloading as lib;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate argon2;

// mod auth;
mod assets;
mod catalogue;
mod config;
mod context;
mod db;
mod extension;
mod library;
mod proxy;
mod schema;
mod status;
mod user;

use anyhow::Result;
use clap::Clap;

use crate::context::GlobalContext;
use crate::schema::{MutationRoot, QueryRoot, TanoshiSchema};
use async_graphql::extensions::ApolloTracing;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptySubscription, Schema};
use async_graphql_warp::{BadRequest, Response};
use config::Config;
use std::convert::Infallible;
use warp::http::{Response as HttpResponse, StatusCode};
use warp::{Filter, Rejection};

#[derive(Clap)]
#[clap(version = "0.22.4")]
struct Opts {
    /// Path to config file
    #[clap(long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let opts: Opts = Opts::parse();
    let config = Config::open(opts.config)?;

    let secret = config.secret;
    let mut extensions = extension::Extensions::new(config.plugin_path.clone());
    if extensions.initialize(config.plugin_config).is_err() {
        log::error!("error initialize plugin");
    }

    // let serve_static = filters::static_files::static_files();

    // let routes = api.or(serve_static).with(warp::log("manga"));
    let pool = db::establish_connection(config.database_path).await;
    let mangadb = db::MangaDatabase::new(pool.clone());
    let userdb = db::UserDatabase::new(pool.clone());

    let schema: TanoshiSchema = Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription::default(),
    )
    .extension(ApolloTracing)
    .data(GlobalContext::new(userdb, mangadb, secret, extensions))
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

    let graphql_playground = warp::path!("graphql").and(warp::get()).map(|| {
        HttpResponse::builder()
            .header("content-type", "text/html")
            .body(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
    });

    let static_files = assets::filter::static_files();

    let cors = warp::cors().allow_any_origin().allow_method("POST");
    let routes = proxy::proxy()
        .or(graphql_playground)
        .or(static_files)
        .or(graphql_post)
        .recover(|err: Rejection| async move {
            if let Some(BadRequest(err)) = err.find() {
                return Ok::<_, Infallible>(warp::reply::with_status(
                    err.to_string(),
                    StatusCode::BAD_REQUEST,
                ));
            }

            Ok(warp::reply::with_status(
                "INTERNAL_SERVER_ERROR".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        })
        .with(cors);

    warp::serve(routes).run(([0, 0, 0, 0], config.port)).await;

    return Ok(());
}
