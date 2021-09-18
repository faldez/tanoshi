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

use crate::{
    db::{
        model::{Chapter, User},
        MangaDatabase, UserDatabase,
    },
    notifier::pushover::Pushover,
};

pub enum Command {
    TelegramMessage(i64, String),
    PushoverMessage(String, String),
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
    pushover: Option<Pushover>,
}

impl Worker {
    fn new(
        period: u64,
        mangadb: MangaDatabase,
        userdb: UserDatabase,
        extension_bus: ExtensionBus,
        telegram_bot: Option<DefaultParseMode<AutoSend<Bot>>>,
        pushover: Option<Pushover>,
    ) -> Self {
        #[cfg(not(debug_assertions))]
        let period = if period > 0 && period < 3600 {
            3600
        } else {
            period
        };
        info!("periodic updates every {} seconds", period);
        Self {
            period,
            mangadb,
            userdb,
            extension_bus,
            telegram_bot,
            pushover,
        }
    }

    async fn check_chapter_update(&self) -> Result<(), anyhow::Error> {
        let manga_in_library = self.mangadb.get_all_user_library().await?;

        let mut new_manga_chapter: HashMap<i64, Vec<ChapterUpdate>> = HashMap::new();
        let mut manga_user_map: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut user_map: HashMap<i64, User> = HashMap::new();

        for item in manga_in_library {
            manga_user_map.insert(item.manga.id, item.user_ids.clone());
            for user_id in item.user_ids.iter() {
                if user_map.get(&user_id).is_none() {
                    let user = self.userdb.get_user_by_id(*user_id).await?;
                    user_map.insert(*user_id, user);
                }
            }

            let last_uploaded_chapter = self
                .mangadb
                .get_last_uploaded_chapters_by_manga_id(item.manga.id)
                .await
                .map(|ch| ch.uploaded);

            info!("Checking updates: {}", item.manga.title);

            let chapters = match self
                .extension_bus
                .get_chapters(item.manga.source_id, item.manga.path.clone())
                .await
            {
                Ok(chapters) => {
                    let chapters: Vec<Chapter> = chapters
                        .into_iter()
                        .map(|ch| {
                            let mut c: Chapter = ch.into();
                            c.manga_id = item.manga.id;
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
                    .filter(|ch| ch.uploaded > last_uploaded_chapter)
                    .collect()
            } else {
                chapters
            };

            if chapters.is_empty() {
                continue;
            }

            let chapters = chapters
                .iter()
                .map(|ch| ChapterUpdate {
                    manga_title: item.manga.title.clone(),
                    cover_url: item.manga.cover_url.clone(),
                    title: ch.title.clone(),
                })
                .collect();

            new_manga_chapter.insert(item.manga.id, chapters);

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        debug!("new chapters: {:?}", new_manga_chapter);

        if self.telegram_bot.is_none() && self.pushover.is_none() {
            return Ok(());
        }

        for (manga_id, updates) in new_manga_chapter.iter() {
            let user_ids = if let Some(user_ids) = manga_user_map.get(&manga_id) {
                user_ids
                    .clone()
                    .iter()
                    .filter_map(|user_id| user_map.get(user_id).cloned())
                    .map(|user| (user.telegram_chat_id, user.pushover_user_key.clone()))
                    .collect::<Vec<(Option<i64>, Option<String>)>>()
            } else {
                continue;
            };

            for (telegram_chat_id, pushover_user_key) in user_ids {
                for update in updates {
                    if let Some((bot, chat_id)) = self.telegram_bot.as_ref().zip(telegram_chat_id) {
                        if let Err(e) = bot.send_message(chat_id, update.to_string()).await {
                            error!("failed to send message, reason: {}", e);
                        }
                    }

                    if let Some((pushover, user_key)) =
                        self.pushover.as_ref().zip(pushover_user_key.as_ref())
                    {
                        if let Err(e) = pushover
                            .send_notification_with_title(
                                user_key,
                                &update.manga_title,
                                &update.title,
                            )
                            .await
                        {
                            error!("failed to send PushoverMessage, reason: {}", e);
                        }
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
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

        if updates.is_empty() {
            info!("no extension updates found");
        }

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
            info!("no tanoshi update found");
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
                        Command::PushoverMessage(user_key, message) => {
                            if let Some(pushover) = self.pushover.as_ref() {
                                if let Err(e) = pushover.send_notification(&user_key, &message).await {
                                    error!("failed to send PushoverMessage, reason: {}", e);
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
    pushover: Option<Pushover>,
) -> (JoinHandle<()>, UnboundedSender<Command>) {
    let (tx, rx) = unbounded_channel();
    let worker = Worker::new(
        period,
        mangadb,
        userdb,
        extension_bus,
        telegram_bot,
        pushover,
    );

    let handle = tokio::spawn(async move {
        worker.run(rx).await;
    });

    (handle, tx)
}
