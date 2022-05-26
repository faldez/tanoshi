#[macro_use]
extern crate log;
extern crate argon2;

use clap::Parser;
use futures::future::OptionFuture;
use tanoshi::{
    application::worker,
    db,
    domain::services::{
        image::ImageService, manga::MangaService, source::SourceService, tracker::TrackerService,
        user::UserService,
    },
    infrastructure::{
        config::{self, Config, GLOBAL_CONFIG},
        notifier,
        repositories::{
            image::ImageRepositoryImpl, manga::MangaRepositoryImpl, source::SourceRepositoryImpl,
            tracker::TrackerRepositoryImpl, user::UserRepositoryImpl,
        },
    },
    presentation::{graphql::local, ServerBuilder},
};
use tanoshi_notifier::{pushover::Pushover, telegram::Telegram};
use tanoshi_tracker::{AniList, MyAnimeList};
use tanoshi_vm::{extension::SourceBus, prelude::Source};

#[derive(Parser)]
struct Opts {
    /// Path to config file
    #[clap(long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
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

    debug!("config: {:?}", config);

    let pool = db::establish_connection(&config.database_path).await?;
    let mangadb = db::MangaDatabase::new(pool.clone());

    let user_repo = UserRepositoryImpl::new(pool.clone().into());
    let user_svc = UserService::new(user_repo.clone());

    let extension_manager = SourceBus::new(&config.plugin_path);

    extension_manager.load_all().await?;

    let source_repo = SourceRepositoryImpl::new(extension_manager.clone());
    let source_svc = SourceService::new(source_repo);

    let manga_repo = MangaRepositoryImpl::new(pool.clone().into());
    let manga_svc = MangaService::new(manga_repo, extension_manager.clone());

    match &config.local_path {
        config::LocalFolders::Single(local_path) => {
            extension_manager
                .insert(Source::from(Box::new(local::Local::new(
                    10000,
                    "Local".to_string(),
                    local_path,
                ))))
                .await?;
        }
        config::LocalFolders::Multiple(local_paths) => {
            for (index, local_path) in local_paths.iter().enumerate() {
                // source id starts from 10000
                let index = index + 10000;
                extension_manager
                    .insert(Source::from(Box::new(local::Local::new(
                        index as i64,
                        local_path.name.clone(),
                        &local_path.path,
                    ))))
                    .await?;
            }
        }
    }

    let mut notifier_builder = notifier::Builder::new(user_repo.clone());

    let mut telegram_bot_fut: OptionFuture<_> = None.into();
    if let Some(telegram_config) = config.telegram.clone() {
        let bot = Telegram::new(telegram_config.token);
        telegram_bot_fut = Some(tanoshi_notifier::telegram::run(bot.clone())).into();
        notifier_builder = notifier_builder.telegram(bot);
    }

    if let Some(pushover_cfg) = config.pushover.as_ref() {
        notifier_builder =
            notifier_builder.pushover(Pushover::new(pushover_cfg.application_key.clone()));
    }

    let notifier = notifier_builder.finish();

    let (download_tx, download_worker_handle) = worker::downloads::start(
        &config.download_path,
        mangadb.clone(),
        extension_manager.clone(),
        notifier.clone(),
    );

    let update_worker_handle = worker::updates::start(
        config.update_interval,
        mangadb.clone(),
        extension_manager.clone(),
        download_tx.clone(),
        notifier.clone(),
    );

    let mal_client = config
        .base_url
        .clone()
        .zip(config.myanimelist.clone())
        .and_then(|(base_url, mal_cfg)| {
            MyAnimeList::new(&base_url, mal_cfg.client_id.clone(), mal_cfg.client_secret).ok()
        });

    let al_client = config
        .base_url
        .clone()
        .zip(config.anilist.clone())
        .and_then(|(base_url, al_cfg)| {
            AniList::new(&base_url, al_cfg.client_id.clone(), al_cfg.client_secret).ok()
        });

    let tracker_repo =
        TrackerRepositoryImpl::new(pool.clone().into(), mal_client.clone(), al_client);
    let tracker_svc = TrackerService::new(tracker_repo);

    let image_repo = ImageRepositoryImpl::new();
    let image_svc = ImageService::new(image_repo);

    let mut server_builder = ServerBuilder::new()
        .with_user_svc(user_svc)
        .with_tracker_svc(tracker_svc)
        .with_source_svc(source_svc)
        .with_manga_svc(manga_svc)
        .with_image_svc(image_svc)
        .with_mangadb(mangadb)
        .with_ext_manager(extension_manager)
        .with_download_tx(download_tx)
        .with_notifier(notifier);

    if config.enable_playground {
        server_builder = server_builder.enable_playground();
    }

    let server_fut = server_builder.build()?.serve(([0, 0, 0, 0], config.port));

    tokio::select! {
        _ = server_fut => {
            info!("server shutdown");
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
