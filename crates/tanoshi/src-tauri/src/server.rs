// from https://github.com/tauri-apps/tauri-plugin-localhost

use tanoshi_vm::prelude::{Source, SourceBus};
use tauri::{
  plugin::{Plugin, Result as PluginResult},
  AppHandle, Runtime,
};

use tanoshi::{
  application::worker,
  db,
  domain::services::{
    chapter::ChapterService, history::HistoryService, image::ImageService, library::LibraryService,
    manga::MangaService, source::SourceService, tracker::TrackerService, user::UserService,
  },
  infrastructure::{
    config::{self, GLOBAL_CONFIG},
    notifier,
    repositories::{
      chapter::ChapterRepositoryImpl, history::HistoryRepositoryImpl, image::ImageRepositoryImpl,
      library::LibraryRepositoryImpl, manga::MangaRepositoryImpl, source::SourceRepositoryImpl,
      tracker::TrackerRepositoryImpl, user::UserRepositoryImpl,
    },
  },
  presentation::{graphql::local, ServerBuilder},
};
use tanoshi_tracker::{AniList, MyAnimeList};

pub struct Server {
  port: u16,
}

impl Server {
  pub fn new() -> Self {
    let port = portpicker::pick_unused_port().unwrap();
    Self { port }
  }
}

impl<R: Runtime> Plugin<R> for Server {
  fn name(&self) -> &'static str {
    "graphql"
  }

  fn initialization_script(&self) -> Option<String> {
    Some(format!("window.__TANOSHI_PORT__ = {};", self.port))
  }

  fn initialize(&mut self, _app: &AppHandle<R>, _config: serde_json::Value) -> PluginResult<()> {
    let port = self.port;

    tauri::async_runtime::spawn(async move {
      let config = GLOBAL_CONFIG.get().unwrap();

      let pool = match db::establish_connection(&config.database_path).await {
        Ok(pool) => pool,
        Err(_) => {
          return;
        }
      };

      let mangadb = db::MangaDatabase::new(pool.clone());

      let user_repo = UserRepositoryImpl::new(pool.clone());
      let user_svc = UserService::new(user_repo.clone());

      let extension_manager = SourceBus::new(&config.plugin_path);

      let _ = extension_manager.load_all().await;

      let source_repo = SourceRepositoryImpl::new(extension_manager.clone());
      let source_svc = SourceService::new(source_repo);

      let manga_repo = MangaRepositoryImpl::new(pool.clone());
      let manga_svc = MangaService::new(manga_repo, extension_manager.clone());

      let chapter_repo = ChapterRepositoryImpl::new(pool.clone());
      let chapter_svc = ChapterService::new(chapter_repo, extension_manager.clone());

      let library_repo = LibraryRepositoryImpl::new(pool.clone());
      let libary_svc = LibraryService::new(library_repo);

      let history_repo = HistoryRepositoryImpl::new(pool.clone());
      let history_svc = HistoryService::new(history_repo);

      match &config.local_path {
        config::LocalFolders::Single(local_path) => {
          let _ = extension_manager
            .insert(Source::from(Box::new(local::Local::new(
              10000,
              "Local".to_string(),
              local_path,
            ))))
            .await;
        }
        config::LocalFolders::Multiple(local_paths) => {
          for (index, local_path) in local_paths.iter().enumerate() {
            // source id starts from 10000
            let index = index + 10000;
            let _ = extension_manager
              .insert(Source::from(Box::new(local::Local::new(
                index as i64,
                local_path.name.clone(),
                &local_path.path,
              ))))
              .await;
          }
        }
      }

      let notifier = notifier::Builder::new(user_repo.clone()).finish();

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

      let tracker_repo = TrackerRepositoryImpl::new(pool.clone(), mal_client.clone(), al_client);
      let tracker_svc = TrackerService::new(tracker_repo);

      let image_repo = ImageRepositoryImpl::new();
      let image_svc = ImageService::new(image_repo);

      let mut server_builder = ServerBuilder::new()
        .with_user_svc(user_svc)
        .with_tracker_svc(tracker_svc)
        .with_source_svc(source_svc)
        .with_manga_svc(manga_svc)
        .with_chapter_svc(chapter_svc)
        .with_image_svc(image_svc)
        .with_library_svc(libary_svc)
        .with_history_svc(history_svc)
        .with_mangadb(mangadb)
        .with_ext_manager(extension_manager)
        .with_download_tx(download_tx)
        .with_notifier(notifier);

      if config.enable_playground {
        server_builder = server_builder.enable_playground();
      }

      let server_fut = match server_builder.build() {
        Ok(server) => server.serve(([127, 0, 0, 1], port)),
        Err(_) => {
          return;
        }
      };

      tokio::select! {
          _ = server_fut => {
              println!("server shutdown");
          }
          _ = update_worker_handle => {
              println!("update worker quit");
          }
          _ = download_worker_handle => {
              println!("download worker quit");
          }
          _ = tokio::signal::ctrl_c() => {
              println!("ctrl+c signal");
          }
      }

      pool.close().await;
    });

    Ok(())
  }
}
