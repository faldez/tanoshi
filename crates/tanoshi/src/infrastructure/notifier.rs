use crate::{domain::repositories::user::UserRepository, infrastructure::config::GLOBAL_CONFIG};
use tanoshi_notifier::{pushover::Pushover, telegram::Telegram};

pub struct Builder<R>
where
    R: UserRepository,
{
    user_repo: R,
    pushover: Option<Pushover>,
    telegram: Option<Telegram>,
}

impl<R> Builder<R>
where
    R: UserRepository,
{
    pub fn new(user_repo: R) -> Self {
        Self {
            user_repo,
            pushover: None,
            telegram: None,
        }
    }

    pub fn telegram(self, telegram: Telegram) -> Self {
        Self {
            telegram: Some(telegram),
            ..self
        }
    }

    pub fn pushover(self, pushover: Pushover) -> Self {
        Self {
            pushover: Some(pushover),
            ..self
        }
    }

    pub fn finish(self) -> Notifier<R> {
        Notifier {
            user_repo: self.user_repo,
            telegram: self.telegram,
            pushover: self.pushover,
        }
    }
}

#[derive(Clone)]
pub struct Notifier<R>
where
    R: UserRepository,
{
    user_repo: R,
    pushover: Option<Pushover>,
    telegram: Option<Telegram>,
}

impl<R> Notifier<R>
where
    R: UserRepository,
{
    pub async fn send_all_to_user(
        &self,
        user_id: i64,
        title: Option<String>,
        body: &str,
    ) -> Result<(), anyhow::Error> {
        let user = self.user_repo.get_user_by_id(user_id).await?;
        if let Some(user_key) = user.pushover_user_key {
            let _ = self
                .send_message_to_pushover(&user_key, title.clone(), body)
                .await;
        }
        if let Some(chat_id) = user.telegram_chat_id {
            let mut message = "".to_string();
            if let Some(title) = title {
                message = format!("<b>{}</b>\n", title);
            }
            message = format!("{}{}", message, body);
            let _ = self.send_message_to_telegram(chat_id, &message).await;
        }

        Ok(())
    }

    pub async fn send_all_to_admins(
        &self,
        title: Option<String>,
        body: &str,
    ) -> Result<(), anyhow::Error> {
        let admins = self.user_repo.get_admins().await?;
        for user in admins {
            let _ = self.send_all_to_user(user.id, title.clone(), body).await;
        }

        Ok(())
    }

    pub async fn send_chapter_notification(
        &self,
        user_id: i64,
        manga_title: &str,
        chapter_title: &str,
        chapter_id: i64,
    ) -> Result<(), anyhow::Error> {
        let user = self.user_repo.get_user_by_id(user_id).await?;
        if let Some((user_key, pushover)) = user.pushover_user_key.zip(self.pushover.as_ref()) {
            if let Some(base_url) = GLOBAL_CONFIG.get().and_then(|cfg| cfg.base_url.clone()) {
                pushover
                    .send_notification_with_title_and_url(
                        &user_key,
                        manga_title,
                        chapter_title,
                        &format!("{base_url}/chapter/{chapter_id}"),
                        "Open",
                    )
                    .await?;
            } else {
                pushover
                    .send_notification_with_title(&user_key, manga_title, chapter_title)
                    .await?;
            }
        }

        if let Some(chat_id) = user.telegram_chat_id {
            let mut message = format!("<b>{manga_title}</b>\n");
            if let Some(base_url) = GLOBAL_CONFIG.get().and_then(|cfg| cfg.base_url.clone()) {
                message = format!(
                    "{message}<a href=\"{base_url}/chapter/{chapter_id}\">{chapter_title}</a>"
                );
            } else {
                message = format!("{message}{chapter_title}");
            }

            let _ = self.send_message_to_telegram(chat_id, &message).await;
        }
        Ok(())
    }

    pub async fn send_message_to_telegram(
        &self,
        chat_id: i64,
        message: &str,
    ) -> Result<(), anyhow::Error> {
        self.telegram
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("telegram bot not set"))?
            .send_message(chat_id, message)
            .await?;
        Ok(())
    }

    pub async fn send_message_to_pushover(
        &self,
        user_key: &str,
        title: Option<String>,
        body: &str,
    ) -> Result<(), anyhow::Error> {
        let pushover = self
            .pushover
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("pushover not set"))?;
        if let Some(title) = title {
            pushover
                .send_notification_with_title(user_key, &title, body)
                .await?;
        } else {
            pushover.send_notification(user_key, body).await?;
        }

        Ok(())
    }

    #[cfg(feature = "desktop")]
    pub fn send_desktop_notification(
        &self,
        title: Option<String>,
        body: &str,
    ) -> Result<(), anyhow::Error> {
        use tauri::api::notification::Notification;

        Notification::new("com.faldez.tanoshi")
            .title(title.unwrap_or_else(|| "Notification".to_string()))
            .body(body)
            .show()?;

        Ok(())
    }
}
