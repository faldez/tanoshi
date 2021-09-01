use anyhow::anyhow;
use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
};
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

use crate::db::{model::Chapter, MangaDatabase};

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
    local_manga_path: PathBuf,
    mangadb: MangaDatabase,
    extension_bus: ExtensionBus,
    telegram_bot: Option<DefaultParseMode<AutoSend<Bot>>>,
}

impl Worker {
    fn new<P: AsRef<Path>>(
        period: u64,
        local_manga_path: P,
        mangadb: MangaDatabase,
        extension_bus: ExtensionBus,
        telegram_bot: Option<DefaultParseMode<AutoSend<Bot>>>,
    ) -> Self {
        #[cfg(not(debug_assertions))]
        let period = if period < 3600 { 3600 } else { period };
        info!("periodic updates every {} secons", period);
        Self {
            period,
            local_manga_path: PathBuf::new().join(local_manga_path),
            mangadb,
            extension_bus,
            telegram_bot,
        }
    }

    async fn check_new_chapters(&self) -> Result<(), anyhow::Error> {
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
                // dont need to check again
                return Ok(());
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
                    return Err(anyhow!("{}", e));
                }
            };

            self.mangadb.insert_chapters(&chapters).await?;

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
        }

        info!("users' new chapters: {:?}", new_users_chapters);

        if let Some(bot) = self.telegram_bot.as_ref() {
            for (chat_id, chapters) in new_users_chapters.into_iter() {
                for chapter in chapters {
                    bot.send_message(chat_id, chapter.to_string()).await?;
                }
            }
        }

        Ok(())
    }

    async fn scan_local_manga(&self) -> Result<(), anyhow::Error> {
        info!("scan {} for manga", self.local_manga_path.display());
        let local_scanner =
            super::scanner::Scanner::new(self.local_manga_path.clone(), self.mangadb.clone());

        local_scanner.scan().await?;

        Ok(())
    }

    async fn run(&self, rx: UnboundedReceiver<Command>) {
        let mut rx = rx;
        let mut interval = time::interval(time::Duration::from_secs(self.period));

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
                start = interval.tick() => {
                    info!("start periodic updates");

                    if let Err(e) = self.check_new_chapters().await {
                        error!("failed to check_new_chapters, reason: {}", e);
                    }

                    if let Err(e) = self.scan_local_manga().await {
                        error!("failed to scan_local_manga, reason: {}", e);
                    }

                    info!("periodic updates done in {:?}", Instant::now() - start);

                }
            }
        }
    }
}

pub fn start<P: AsRef<Path>>(
    period: u64,
    local_manga_path: P,
    mangadb: MangaDatabase,
    extension_bus: ExtensionBus,
    telegram_bot: Option<DefaultParseMode<AutoSend<Bot>>>,
) -> (JoinHandle<()>, UnboundedSender<Command>) {
    let (tx, rx) = unbounded_channel();
    let worker = Worker::new(
        period,
        local_manga_path,
        mangadb,
        extension_bus,
        telegram_bot,
    );

    let handle = tokio::spawn(async move {
        worker.run(rx).await;
    });

    (handle, tx)
}
