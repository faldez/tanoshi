#[macro_use]
extern crate log;
extern crate argon2;

use clap::Parser;
use futures::future::OptionFuture;
use tanoshi::{
    application::worker,
    domain::services::{
        chapter::ChapterService, download::DownloadService, history::HistoryService,
        image::ImageService, library::LibraryService, manga::MangaService, source::SourceService,
        tracker::TrackerService, user::UserService,
    },
    infrastructure::{
        config::{self, Config},
        database,
        domain::repositories::{
            chapter::ChapterRepositoryImpl, download::DownloadRepositoryImpl,
            history::HistoryRepositoryImpl, image::ImageRepositoryImpl,
            image_cache::ImageCacheRepositoryImpl, library::LibraryRepositoryImpl,
            manga::MangaRepositoryImpl, source::SourceRepositoryImpl,
            tracker::TrackerRepositoryImpl, user::UserRepositoryImpl,
        },
        local, notification,
    },
    presentation::{graphql::loader::DatabaseLoader, ServerBuilder},
};
use tanoshi_notifier::{gotify::Gotify, pushover::Pushover, telegram::Telegram};
use tanoshi_tracker::{AniList, MyAnimeList};
use tanoshi_vm::{extension::ExtensionManager, prelude::Source};

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
    let config = Config::open(opts.config)?;

    debug!("config: {:?}", config);

    let pool =
        database::establish_connection(&config.database_path, config.create_database).await?;

    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_svc = UserService::new(user_repo.clone());

    let extension_manager = ExtensionManager::new(&config.plugin_path);

    extension_manager.load_all().await?;

    let source_repo = SourceRepositoryImpl::new(extension_manager.clone());
    let source_svc = SourceService::new(source_repo);

    let manga_repo = MangaRepositoryImpl::new(pool.clone());
    let manga_svc = MangaService::new(manga_repo.clone(), extension_manager.clone());

    let chapter_repo = ChapterRepositoryImpl::new(pool.clone());
    let chapter_svc = ChapterService::new(chapter_repo.clone(), extension_manager.clone());

    let library_repo = LibraryRepositoryImpl::new(pool.clone());
    let libary_svc = LibraryService::new(library_repo.clone());

    let history_repo = HistoryRepositoryImpl::new(pool.clone());
    let history_svc = HistoryService::new(chapter_repo.clone(), history_repo.clone());

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

    let mut notifier_builder = notification::Builder::new(user_repo.clone());

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

    if let Some(gotify_cfg) = config.gotify.as_ref() {
        notifier_builder = notifier_builder.gotify(Gotify::new(gotify_cfg.base_url.clone()));
    }

    if let Some(base_url) = config.base_url.as_ref() {
        notifier_builder = notifier_builder.base_url(base_url.clone());
    }

    let notifier = notifier_builder.finish();

    let (download_sender, download_receiver) = worker::downloads::channel();

    let download_repo = DownloadRepositoryImpl::new(pool.clone());
    let download_svc = DownloadService::new(download_repo.clone(), download_sender.clone());

    let download_worker_handle = worker::downloads::start(
        &config.download_path,
        chapter_repo.clone(),
        manga_repo.clone(),
        download_repo.clone(),
        extension_manager.clone(),
        notifier.clone(),
        download_sender.clone(),
        download_receiver,
    );

    worker::updates::start(
        config.update_interval,
        library_repo.clone(),
        chapter_repo.clone(),
        extension_manager.clone(),
        download_sender.clone(),
        config.auto_download_chapters,
        notifier.clone(),
        config.extension_repository.clone(),
        &config.cache_path,
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

    let tracker_repo = TrackerRepositoryImpl::new(pool.clone(), mal_client.clone(), al_client);
    let tracker_svc = TrackerService::new(tracker_repo.clone());

    let image_repo = ImageRepositoryImpl::new();
    let image_cache_repo = ImageCacheRepositoryImpl::new(&config.cache_path);
    let image_svc = ImageService::new(image_repo, image_cache_repo);

    let loader = DatabaseLoader::new(history_repo, library_repo, manga_repo, tracker_repo);

    let mut server_builder = ServerBuilder::new()
        .with_config(config.clone())
        .with_user_svc(user_svc)
        .with_tracker_svc(tracker_svc)
        .with_source_svc(source_svc)
        .with_manga_svc(manga_svc)
        .with_chapter_svc(chapter_svc)
        .with_image_svc(image_svc)
        .with_library_svc(libary_svc)
        .with_history_svc(history_svc)
        .with_download_svc(download_svc)
        .with_ext_manager(extension_manager)
        .with_download_tx(download_sender)
        .with_notifier(notifier)
        .with_loader(loader);

    if config.enable_playground {
        server_builder = server_builder.enable_playground();
    }

    let server_fut = server_builder.build()?.serve(([0, 0, 0, 0], config.port));

    tokio::select! {
        _ = server_fut => {
            info!("server shutdown");
        }
        // _ = update_worker_handle => {
        //     info!("update worker quit");
        // }
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
