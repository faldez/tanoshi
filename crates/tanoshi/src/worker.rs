use std::{collections::HashMap, fmt::Display, str::FromStr};

use serde::Deserialize;
use tanoshi_lib::prelude::Version;
use tanoshi_vm::prelude::ExtensionBus;
use teloxide::{
    adaptors::{AutoSend, DefaultParseMode},
    prelude::Requester,
    Bot,
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::{
    sync::mpsc::unbounded_channel,
    task::JoinHandle,
    time::{self, Instant},
};

use crate::db::{model::Chapter, MangaDatabase, UserDatabase};

pub enum Command {
    TelegramMessage(i64, String),
}

#[derive(Debug, Clone)]
struct ChapterUpdate {
    manga_title: String,
    cover_url: String,
    title: String,
}

impl Display for ChapterUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let manga_title = html_escape::encode_safe(&self.manga_title).to_string();
        let title = html_escape::encode_safe(&self.title).to_string();

        write!(f, "<b>{}</b>\n{}", manga_title, title)
    }
}

struct Worker {
    period: u64,
    mangadb: MangaDatabase,
    userdb: UserDatabase,
    extension_bus: ExtensionBus,
    telegram_bot: Option<DefaultParseMode<AutoSend<Bot>>>,
}

impl Worker {
    fn new(
        period: u64,
        mangadb: MangaDatabase,
        userdb: UserDatabase,
        extension_bus: ExtensionBus,
        telegram_bot: Option<DefaultParseMode<AutoSend<Bot>>>,
    ) -> Self {
        #[cfg(not(debug_assertions))]
        let period = if period > 0 && period < 3600 {
            3600
        } else {
            period
        };
        info!("periodic updates every {} secons", period);
        Self {
            period,
            mangadb,
            userdb,
            extension_bus,
            telegram_bot,
        }
    }

    async fn check_chapter_update(&self) -> Result<(), anyhow::Error> {
        let manga_in_library = self.mangadb.get_all_user_library().await?;

        let mut new_manga_chapter: HashMap<i64, Vec<ChapterUpdate>> = HashMap::new();
        let mut new_users_chapters: HashMap<i64, Vec<ChapterUpdate>> = HashMap::new();

        for (telegram_chat_id, manga) in manga_in_library {
            if let Some(chapters) = new_manga_chapter.get(&manga.id) {
                if let Some(telegram_chat_id) = telegram_chat_id {
                    match new_users_chapters.get_mut(&telegram_chat_id) {
                        Some(user_chapters) => {
                            user_chapters.extend_from_slice(chapters);
                        }
                        None => {
                            new_users_chapters.insert(telegram_chat_id, chapters.clone());
                        }
                    }
                }
                continue;
            }

            let last_uploaded_chapter = self
                .mangadb
                .get_last_uploaded_chapters_by_manga_id(manga.id)
                .await;
            let chapters = match self
                .extension_bus
                .get_chapters(manga.source_id, manga.path.clone())
                .await
            {
                Ok(chapters) => {
                    let chapters: Vec<Chapter> = chapters
                        .into_iter()
                        .map(|ch| {
                            let mut c: Chapter = ch.into();
                            c.manga_id = manga.id;
                            c
                        })
                        .collect();
                    chapters
                }
                Err(e) => {
                    error!("error fetch new chapters, reason: {}", e);
                    continue;
                }
            };

            if let Err(e) = self.mangadb.insert_chapters(&chapters).await {
                error!("error inserting new chapters, reason: {}", e);
                continue;
            }

            let chapters = if let Some(last_uploaded_chapter) = last_uploaded_chapter {
                chapters
                    .into_iter()
                    .filter(|ch| ch.uploaded > last_uploaded_chapter.uploaded)
                    .collect()
            } else {
                chapters
            };

            let chapters: Vec<ChapterUpdate> = chapters
                .iter()
                .map(|ch| ChapterUpdate {
                    manga_title: manga.title.clone(),
                    cover_url: manga.cover_url.clone(),
                    title: ch.title.clone(),
                })
                .collect();

            new_manga_chapter.insert(manga.id, chapters.clone());
            if let Some(telegram_chat_id) = telegram_chat_id {
                match new_users_chapters.get_mut(&telegram_chat_id) {
                    Some(user_chapters) => {
                        user_chapters.extend_from_slice(&chapters);
                    }
                    None => {
                        new_users_chapters.insert(telegram_chat_id, chapters);
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        info!("users' new chapters: {:?}", new_users_chapters);

        if let Some(bot) = self.telegram_bot.as_ref() {
            for (chat_id, chapters) in new_users_chapters.into_iter() {
                for chapter in chapters {
                    if let Err(e) = bot.send_message(chat_id, chapter.to_string()).await {
                        error!("failed to send message, reason: {}", e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }

        Ok(())
    }

    async fn check_extension_update(&self) -> Result<(), anyhow::Error> {
        #[derive(Debug, Clone, Deserialize)]
        pub struct SourceIndex {
            pub id: i64,
            pub name: String,
            pub path: String,
            pub version: String,
            pub icon: String,
        }

        let available_sources_map = {
            let url = "https://raw.githubusercontent.com/faldez/tanoshi-extensions/repo/index.json"
                .to_string();
            let available_sources = reqwest::get(url).await?.json::<Vec<SourceIndex>>().await?;
            let mut available_sources_map = HashMap::new();
            for source in available_sources {
                available_sources_map.insert(source.id, source);
            }
            available_sources_map
        };

        let updates = {
            let installed_sources = self
                .extension_bus
                .list()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            let mut updates: Vec<String> = vec![];
            for source in installed_sources {
                if let Some(index) = available_sources_map.get(&source.id) {
                    if Version::from_str(&index.version)? > source.version {
                        updates.push(format!("<b>{}</b> extension update available", source.name));
                    }
                }
            }

            updates
        };

        for update in updates {
            info!("new extension update found!");
            if let Some(bot) = self.telegram_bot.as_ref() {
                let admins = self.userdb.get_admins().await?;
                for admin in admins {
                    if let Some(chat_id) = admin.telegram_chat_id {
                        bot.send_message(chat_id, &update).await?;
                    }
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

        let client = reqwest::Client::new();
        let release: Release = client
            .get("https://api.github.com/repos/faldez/tanoshi/releases/latest")
            .header(
                "User-Agent",
                format!("Tanoshi/{}", env!("CARGO_PKG_VERSION")),
            )
            .send()
            .await?
            .json()
            .await?;

        if Version::from_str(&release.tag_name[1..])?
            > Version::from_str(env!("CARGO_PKG_VERSION"))?
        {
            info!("new server update found!");
            if let Some(bot) = self.telegram_bot.as_ref() {
                let admins = self.userdb.get_admins().await?;
                for admin in admins {
                    if let Some(chat_id) = admin.telegram_chat_id {
                        bot.send_message(
                            chat_id,
                            format!(
                                "<b>Tanoshi {} Released</b>\n{}",
                                release.tag_name, release.body
                            ),
                        )
                        .await?;
                    }
                }
            }
        } else {
            info!("no update found");
        }

        Ok(())
    }

    async fn run(&self, rx: UnboundedReceiver<Command>) {
        let mut rx = rx;
        let period = if self.period == 0 { 3600 } else { self.period };
        let mut chapter_update_interval = time::interval(time::Duration::from_secs(period));
        let mut server_update_interval = time::interval(time::Duration::from_secs(86400));

        loop {
            tokio::select! {
                Some(cmd) = rx.recv() => {
                    match cmd {
                        Command::TelegramMessage(chat_id, message) => {
                            if let Some(bot) = self.telegram_bot.as_ref() {
                                if let Err(e) = bot.send_message(chat_id, message).await {
                                    error!("failed to send TelegramMessage, reason: {}", e);
                                }
                            }
                        }
                    }
                }
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
    userdb: UserDatabase,
    extension_bus: ExtensionBus,
    telegram_bot: Option<DefaultParseMode<AutoSend<Bot>>>,
) -> (JoinHandle<()>, UnboundedSender<Command>) {
    let (tx, rx) = unbounded_channel();
    let worker = Worker::new(period, mangadb, userdb, extension_bus, telegram_bot);

    let handle = tokio::spawn(async move {
        worker.run(rx).await;
    });

    (handle, tx)
}
