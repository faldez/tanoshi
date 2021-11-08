#[macro_use]
extern crate log;
extern crate argon2;

#[cfg(feature = "embed")]
mod assets;
mod catalogue;
mod config;
mod db;
mod downloads;
mod library;
mod local;
mod notifier;
mod proxy;
mod schema;
mod server;
mod status;
mod user;
mod utils;
mod worker;

use crate::{
    config::{Config, GLOBAL_CONFIG},
    notifier::pushover::Pushover,
};
use clap::Parser;
use futures::future::OptionFuture;
use tanoshi_vm::{bus::ExtensionBus, vm};

use std::sync::Arc;
use teloxide::prelude::RequesterExt;

#[derive(Parser)]
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
    let config =
        GLOBAL_CONFIG.get_or_init(|| Config::open(opts.config).expect("failed to init config"));

    let pool = db::establish_connection(&config.database_path).await?;
    let mangadb = db::MangaDatabase::new(pool.clone());
    let userdb = db::UserDatabase::new(pool.clone());

    let (vm_handle, extension_tx) = vm::start();
    vm::load(&config.plugin_path, extension_tx.clone()).await?;

    let extension_bus = ExtensionBus::new(&config.plugin_path, extension_tx);

    extension_bus
        .insert(
            local::ID,
            Arc::new(local::Local::new(config.local_path.clone())),
        )
        .await?;

    let mut telegram_bot = None;
    let mut telegram_bot_fut: OptionFuture<_> = None.into();
    if let Some(telegram_config) = config.telegram.clone() {
        let bot = teloxide::Bot::new(telegram_config.token)
            .auto_send()
            .parse_mode(teloxide::types::ParseMode::Html);
        telegram_bot_fut = Some(notifier::telegram::run(telegram_config.name, bot.clone())).into();
        telegram_bot = Some(bot);
    }

    let pushover = config
        .pushover
        .clone()
        .map(|pushover_cfg| Pushover::new(pushover_cfg.application_key));

    let (download_tx, download_worker_handle) =
        worker::downloads::start(&config.download_path, mangadb.clone());

    let (worker_handle, worker_tx) = worker::start(telegram_bot, pushover, download_tx);

    let update_worker_handle = worker::updates::start(
        config.update_interval,
        userdb.clone(),
        mangadb.clone(),
        extension_bus.clone(),
        worker_tx.clone(),
    );

    let server_fut = server::serve::<()>(userdb, mangadb, config, extension_bus, worker_tx);

    tokio::select! {
        _ = server_fut => {
            info!("server shutdown");
        }
        _ = vm_handle => {
            info!("vm quit");
        }
        _ = worker_handle => {
            info!("worker quit");
        }
        _ = update_worker_handle => {
            info!("update worker quit");
        }
        _ = download_worker_handle => {
            info!("download worker quit");
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
