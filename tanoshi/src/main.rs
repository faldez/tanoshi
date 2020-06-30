extern crate argon2;
extern crate libloading as lib;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use anyhow::{anyhow, Result};
use clap::Clap;
use rust_embed::RustEmbed;

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use warp::{http::header::HeaderValue, path::Tail, reply::Response, Filter, Rejection, Reply};

mod auth;
mod bot;
mod extension;
mod favorites;
mod filters;
mod handlers;
mod history;
mod update;
mod worker;

#[derive(RustEmbed)]
#[folder = "../tanoshi-web/dist/"]
struct Asset;

#[derive(serde::Deserialize)]
struct Config {
    pub base_url: Option<String>,
    pub port: Option<u16>,
    pub database_path: String,
    pub secret: String,
    pub cache_ttl: Option<u64>,
    pub update_interval: Option<u64>,
    pub telegram_token: Option<String>,
    pub plugin_path: Option<String>,
    pub plugin_config: Option<BTreeMap<String, serde_yaml::Value>>,
}

#[derive(Clap)]
struct Opts {
    /// Path to config file
    #[clap(long, default_value = "~/.config/tanoshi/config.yml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let opts: Opts = Opts::parse();

    let slice = match std::fs::read(opts.config) {
        Ok(slice) => slice,
        Err(e) => {
            error!("failed load config file: {}", e);
            return Err(anyhow!("failed load config file"));
        }
    };

    let config: Config = match serde_yaml::from_slice(&slice) {
        Ok(config) => config,
        Err(e) => {
            error!("failed parse config file: {}", e);
            return Err(anyhow!("failed parse config file"));
        }
    };

    {
        let query = include_str!("../migration/tanoshi.sql");
        let conn = match rusqlite::Connection::open(config.database_path.clone()) {
            Ok(conn) => conn,
            Err(e) => {
                error!("failed open database file: {}", e);
                return Err(anyhow!("failed open database file"));
            }
        };

        if !std::path::Path::new(&config.database_path.clone()).exists() {
            conn.execute_batch(query)?;

            let auth = auth::auth::Auth::new(config.database_path.clone());
            auth.register(auth::User {
                username: "admin".to_string(),
                password: Some("admin".to_string()),
                role: "ADMIN".to_string(),
                telegram_chat_id: None,
            })
            .await;
        } else {
            let user_version = conn
                .pragma_query_value(Some(rusqlite::DatabaseName::Main), "user_version", |row| {
                    row.get(0)
                })
                .unwrap_or(0);
            info!("Schema version {}", user_version);
        }
    }

    let secret = config.secret;
    let plugin_config = config.plugin_config.unwrap_or(BTreeMap::new());
    let plugin_path = config
        .plugin_path
        .clone()
        .unwrap_or("~/.tanoshi/plugins".to_string());
    let extensions = Arc::new(RwLock::new(extension::Extensions::new()));
    for entry in std::fs::read_dir(plugin_path.clone())?
        .into_iter()
        .filter(move |path| {
            if let Ok(p) = path {
                let ext = p
                    .clone()
                    .path()
                    .extension()
                    .unwrap_or("".as_ref())
                    .to_owned();
                if ext == "so" || ext == "dll" || ext == "dylib" {
                    return true;
                }
            }
            return false;
        })
    {
        let path = entry?.path();
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string()
            .replace("lib", "");
        info!("load plugin from {:?}", path.clone());
        let config = plugin_config.get(&name);
        let mut exts = extensions.write().unwrap();
        unsafe {
            match exts.load(path, config) {
                Ok(_) => {}
                Err(e) => error!("not a valid extensions {}", e),
            }
        }
    }

    let bot = match config.telegram_token.clone() {
        Some(token) => Some(bot::Bot::new(token)),
        None => None,
    };

    let update_worker = worker::Worker::new();
    update_worker.remove_cache(config.cache_ttl.unwrap_or(0));
    update_worker.check_update(
        config.update_interval.unwrap_or(0),
        config.database_path.clone(),
        config.base_url.unwrap_or("".to_string()),
        extensions.clone(),
        bot.clone().map(|b| b.get_pub()),
    );

    if let Some(bot) = bot.clone() {
        bot.start();
    }

    let static_files = warp::get().and(warp::path::tail()).and_then(serve);
    let index = warp::get().and_then(serve_index);

    let static_files = static_files.or(index);

    let auth = auth::auth::Auth::new(config.database_path.clone());
    let auth_api = filters::auth::authentication(secret.clone(), auth.clone());

    let manga = extension::manga::Manga::new(config.database_path.clone(), extensions.clone());
    let manga_api = filters::manga::manga(secret.clone(), plugin_path.clone(), manga);

    let fav = favorites::Favorites::new(config.database_path.clone());
    let fav_api = filters::favorites::favorites(secret.clone(), fav);

    let history = history::History::new(config.database_path.clone());
    let history_api = filters::history::history(secret.clone(), history.clone());

    let update = update::Update::new(config.database_path.clone());
    let updates_api = filters::updates::updates(secret.clone(), update.clone());

    let api = manga_api
        .or(auth_api)
        .or(fav_api)
        .or(history_api)
        .or(updates_api)
        .recover(filters::handle_rejection);

    let routes = api.or(static_files).with(warp::log("manga"));

    let port = config.port.unwrap_or(80);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    return Ok(());
}

async fn serve_index() -> Result<impl Reply, Rejection> {
    serve_impl("index.html")
}

async fn serve(path: Tail) -> Result<impl Reply, Rejection> {
    serve_impl(path.as_str())
}

fn serve_impl(path: &str) -> Result<impl Reply, Rejection> {
    let asset = Asset::get(path).ok_or_else(warp::reject::not_found)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut res = Response::new(asset.into());
    res.headers_mut().insert(
        "content-type",
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(res)
}
