use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
};

use chrono::NaiveDateTime;
use futures::StreamExt;
use rayon::prelude::*;
use serde::Deserialize;

use tanoshi_lib::prelude::Version;
use tanoshi_vm::extension::SourceBus;

use crate::{
    application::worker::downloads::Command as DownloadCommand,
    domain::{
        entities::chapter::Chapter,
        repositories::{chapter::ChapterRepository, library::LibraryRepository},
    },
    infrastructure::{
        config::GLOBAL_CONFIG, domain::repositories::user::UserRepositoryImpl,
        notification::Notification,
    },
};
use anyhow::anyhow;
use tokio::time::{self, Instant};

use super::downloads::DownloadSender;

#[derive(Debug, Clone, Deserialize)]
pub struct SourceInfo {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub version: String,
    pub icon: String,
    pub nsfw: bool,
}

struct UpdatesWorker<C, L>
where
    C: ChapterRepository + 'static,
    L: LibraryRepository + 'static,
{
    period: u64,
    client: reqwest::Client,
    library_repo: L,
    chapter_repo: C,
    extensions: SourceBus,
    auto_download_chapters: bool,
    download_tx: DownloadSender,
    notifier: Notification<UserRepositoryImpl>,
    cache_path: PathBuf,
}

impl<C, L> UpdatesWorker<C, L>
where
    C: ChapterRepository + 'static,
    L: LibraryRepository + 'static,
{
    fn new<P: AsRef<Path>>(
        period: u64,
        library_repo: L,
        chapter_repo: C,
        extensions: SourceBus,
        download_tx: DownloadSender,
        notifier: Notification<UserRepositoryImpl>,
        cache_path: P,
    ) -> Self {
        #[cfg(not(debug_assertions))]
        let period = if period > 0 && period < 3600 {
            3600
        } else {
            period
        };
        info!("periodic updates every {} seconds", period);

        let auto_download_chapters = GLOBAL_CONFIG
            .get()
            .map(|cfg| cfg.auto_download_chapters)
            .unwrap_or(false);

        Self {
            period,
            client: reqwest::Client::new(),
            library_repo,
            chapter_repo,
            extensions,
            auto_download_chapters,
            download_tx,
            notifier,
            cache_path: PathBuf::new().join(cache_path),
        }
    }

    async fn check_chapter_update(&self) -> Result<(), anyhow::Error> {
        let mut manga_in_library = self.library_repo.get_manga_from_all_users_library().await;

        while let Some(Ok(manga)) = manga_in_library.next().await {
            debug!("Checking updates: {}", manga.title);

            let last_uploaded_chapter = manga
                .last_uploaded_at
                .unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));

            let chapters: Vec<Chapter> = match self
                .extensions
                .get_chapters(manga.source_id, manga.path.clone())
                .await
            {
                Ok(chapters) => chapters
                    .into_par_iter()
                    .map(|ch| {
                        let mut c: Chapter = ch.into();
                        c.manga_id = manga.id;
                        c
                    })
                    .collect(),
                Err(e) => {
                    error!("error fetch new chapters, reason: {}", e);
                    continue;
                }
            };

            self.chapter_repo.insert_chapters(&chapters).await?;

            let chapters: Vec<Chapter> = self
                .chapter_repo
                .get_chapters_by_manga_id(manga.id, None, None, false)
                .await?
                .into_par_iter()
                .filter(|chapter| chapter.uploaded > last_uploaded_chapter)
                .collect();

            if chapters.len() > 0 {
                info!("{} has {} new chapters", manga.title, chapters.len());
            } else {
                debug!("{} has no new chapters", manga.title);
            }

            for chapter in chapters {
                #[cfg(feature = "desktop")]
                self.notifier
                    .send_desktop_notification(Some(manga.title.clone()), &chapter.title)?;

                let users = self
                    .library_repo
                    .get_users_by_manga_id(manga.id)
                    .await
                    .unwrap_or_default();

                for user in users {
                    self.notifier
                        .send_chapter_notification(
                            user.id,
                            &manga.title,
                            &chapter.title,
                            chapter.id,
                        )
                        .await?;
                }

                if self.auto_download_chapters {
                    info!("add chapter to download queue");
                    self.download_tx
                        .send(DownloadCommand::InsertIntoQueueBySourcePath(
                            chapter.source_id,
                            chapter.path,
                        ))
                        .unwrap();
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        Ok(())
    }

    async fn check_extension_update(&self) -> Result<(), anyhow::Error> {
        let url = GLOBAL_CONFIG
            .get()
            .map(|cfg| format!("{}/index.json", cfg.extension_repository))
            .ok_or_else(|| anyhow!("no config set"))?;

        let available_sources_map = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<Vec<SourceInfo>>()
            .await?
            .into_par_iter()
            .map(|source| (source.id, source))
            .collect::<HashMap<i64, SourceInfo>>();

        let installed_sources = self.extensions.list().await?;

        for source in installed_sources {
            if available_sources_map
                .get(&source.id)
                .and_then(|index| Version::from_str(&index.version).ok())
                .map(|v| v > Version::from_str(source.version).unwrap_or_default())
                .unwrap_or(false)
            {
                let message = format!("{} extension update available", source.name);
                if let Err(e) = self.notifier.send_all_to_admins(None, &message).await {
                    error!("failed to send extension update to admin, {}", e);
                }

                #[cfg(feature = "desktop")]
                if let Err(e) = self
                    .notifier
                    .send_desktop_notification(Some("Extension Update".to_string()), &message)
                {
                    error!("failed to send notification, reason {}", e);
                }
            }
        }

        Ok(())
    }

    async fn check_server_update(&self) -> Result<(), anyhow::Error> {
        #[derive(Debug, Deserialize)]
        struct Release {
            pub tag_name: String,
            pub body: String,
        }

        let release: Release = self
            .client
            .get("https://api.github.com/repos/faldez/tanoshi/releases/latest")
            .header(
                "User-Agent",
                format!("Tanoshi/{}", env!("CARGO_PKG_VERSION")).as_str(),
            )
            .send()
            .await?
            .json()
            .await?;

        if Version::from_str(&release.tag_name[1..])?
            > Version::from_str(env!("CARGO_PKG_VERSION"))?
        {
            info!("new server update found!");
            let message = format!("Tanoshi {} Released\n{}", release.tag_name, release.body);
            if let Err(e) = self.notifier.send_all_to_admins(None, &message).await {
                error!("failed to send extension update to admin, {}", e);
            }

            #[cfg(feature = "desktop")]
            if let Err(e) = self
                .notifier
                .send_desktop_notification(Some("Update Available".to_string()), &message)
            {
                error!("failed to send notification, reason {}", e);
            }
        } else {
            info!("no tanoshi update found");
        }

        Ok(())
    }

    async fn clear_cache(&self) -> Result<(), anyhow::Error> {
        let mut read_dir = tokio::fs::read_dir(&self.cache_path).await?;
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            if let Some(created) = entry
                .metadata()
                .await?
                .created()
                .ok()
                .and_then(|created| created.elapsed().ok())
                .map(|elapsed| {
                    chrono::Duration::from_std(elapsed)
                        .unwrap_or_else(|_| chrono::Duration::max_value())
                })
            {
                if created.num_days() >= 10 {
                    info!("removing {}", entry.path().display());
                    if let Err(e) = tokio::fs::remove_file(entry.path()).await {
                        error!("failed to remove {}: {e}", entry.path().display());
                    }
                }
            }
        }

        Ok(())
    }

    async fn run(self) {
        let period = if self.period == 0 { 3600 } else { self.period };
        let mut chapter_update_interval = time::interval(time::Duration::from_secs(period));
        let mut server_update_interval = time::interval(time::Duration::from_secs(86400));
        let mut clear_cache_interval = time::interval(time::Duration::from_secs(10 * 86400));

        loop {
            tokio::select! {
                start = chapter_update_interval.tick() => {
                    if self.period == 0 {
                        continue;
                    }

                    info!("start periodic updates");

                    if let Err(e) = self.check_chapter_update().await {
                        error!("failed check chapter update: {e}")
                    }

                    info!("periodic updates done in {:?}", Instant::now() - start);
                }
                _ = server_update_interval.tick() => {
                    info!("check server update");

                    if let Err(e) = self.check_server_update().await {
                        error!("failed check server update: {e}")
                    }

                    info!("check extension update");

                    if let Err(e) = self.check_extension_update().await {
                        error!("failed check extension update: {e}")
                    }
                }
                _ = clear_cache_interval.tick() => {
                    if let Err(e) = self.clear_cache().await {
                        error!("failed clear cache: {e}")
                    }
                }
            }
        }
    }
}

pub fn start<C, L, P>(
    period: u64,
    library_repo: L,
    chapter_repo: C,
    extensions: SourceBus,
    download_tx: DownloadSender,
    notifier: Notification<UserRepositoryImpl>,
    cache_path: P,
) where
    C: ChapterRepository + 'static,
    L: LibraryRepository + 'static,
    P: AsRef<Path>,
{
    let worker = UpdatesWorker::new(
        period,
        library_repo,
        chapter_repo,
        extensions,
        download_tx,
        notifier,
        cache_path,
    );

    let handle = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        handle.block_on(worker.run());
    });
}
