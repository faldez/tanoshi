use std::{collections::HashMap, str::FromStr};

use chrono::NaiveDateTime;
use rayon::prelude::*;
use serde::Deserialize;

use tanoshi_lib::prelude::Version;
use tanoshi_vm::extension::SourceBus;

use crate::{
    config::GLOBAL_CONFIG,
    db::{model::Chapter, MangaDatabase},
    notifier::Notifier,
    worker::downloads::Command as DownloadCommand,
};
use anyhow::anyhow;
use tokio::{
    task::JoinHandle,
    time::{self, Instant},
};

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

struct UpdatesWorker {
    period: u64,
    client: reqwest::Client,
    mangadb: MangaDatabase,
    extensions: SourceBus,
    auto_download_chapters: bool,
    download_tx: DownloadSender,
    notifier: Notifier,
}

impl UpdatesWorker {
    fn new(
        period: u64,
        mangadb: MangaDatabase,
        extensions: SourceBus,
        download_tx: DownloadSender,
        notifier: Notifier,
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
            mangadb,
            extensions,
            auto_download_chapters,
            download_tx,
            notifier,
        }
    }

    async fn check_chapter_update(&self) -> Result<(), anyhow::Error> {
        let manga_in_library = self.mangadb.get_all_user_library().await?;

        for item in manga_in_library {
            debug!("Checking updates: {}", item.manga.title);

            let last_uploaded_chapter = self
                .mangadb
                .get_last_uploaded_chapters_by_manga_id(item.manga.id)
                .await
                .map(|ch| ch.uploaded)
                .unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));

            let chapters: Vec<Chapter> = match self
                .extensions
                .get_chapters(item.manga.source_id, item.manga.path.clone())
                .await
            {
                Ok(chapters) => chapters
                    .into_par_iter()
                    .map(|ch| {
                        let mut c: Chapter = ch.into();
                        c.manga_id = item.manga.id;
                        c
                    })
                    .collect(),
                Err(e) => {
                    error!("error fetch new chapters, reason: {}", e);
                    continue;
                }
            };

            if let Err(e) = self.mangadb.insert_chapters(&chapters).await {
                error!("error insert chapters, reason: {}", e);
                continue;
            }

            let chapters = match self
                .mangadb
                .get_chapters_by_manga_id_after(item.manga.id, last_uploaded_chapter)
                .await
            {
                Ok(chapters) => chapters,
                Err(e) => {
                    error!("error insert chapters, reason: {}", e);
                    continue;
                }
            };

            if chapters.len() > 0 {
                info!("{} has {} new chapters", item.manga.title, chapters.len());
            } else {
                debug!("{} has {} new chapters", item.manga.title, chapters.len());
            }

            for chapter in chapters {
                #[cfg(feature = "desktop")]
                if let Err(e) = self
                    .notifier
                    .send_desktop_notification(Some(item.manga.title.clone()), &chapter.title)
                {
                    error!("failed to send desktop notification, reason {}", e);
                }

                for user_id in item.user_ids.iter() {
                    if let Err(e) = self
                        .notifier
                        .send_chapter_notification(
                            *user_id,
                            &item.manga.title,
                            &chapter.title,
                            chapter.id,
                        )
                        .await
                    {
                        error!("failed to send notification, reason {}", e);
                    }
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

    async fn run(&self) {
        let period = if self.period == 0 { 3600 } else { self.period };
        let mut chapter_update_interval = time::interval(time::Duration::from_secs(period));
        let mut server_update_interval = time::interval(time::Duration::from_secs(86400));

        loop {
            tokio::select! {
                start = chapter_update_interval.tick() => {
                    if self.period == 0 {
                        continue;
                    }

                    info!("start periodic updates");

                    if let Err(e) = self.check_chapter_update().await {
                        error!("failed check chapter update: {}", e)
                    }

                    info!("periodic updates done in {:?}", Instant::now() - start);
                }
                _ = server_update_interval.tick() => {
                    info!("check server update");

                    if let Err(e) = self.check_server_update().await {
                        error!("failed check server update: {}", e)
                    }

                    info!("check extension update");

                    if let Err(e) = self.check_extension_update().await {
                        error!("failed check extension update: {}", e)
                    }
                }
            }
        }
    }
}

pub fn start(
    period: u64,
    mangadb: MangaDatabase,
    extensions: SourceBus,
    download_tx: DownloadSender,
    notifier: Notifier,
) -> JoinHandle<()> {
    let worker = UpdatesWorker::new(period, mangadb, extensions, download_tx, notifier);

    tokio::spawn(async move {
        worker.run().await;
    })
}
