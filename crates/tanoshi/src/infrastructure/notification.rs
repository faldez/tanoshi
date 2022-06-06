use crate::domain::repositories::user::UserRepository;
use tanoshi_notifier::{gotify::Gotify, pushover::Pushover, telegram::Telegram, Notifier};

pub struct Builder<R>
where
    R: UserRepository,
{
    user_repo: R,
    pushover: Option<Pushover>,
    telegram: Option<Telegram>,
    gotify: Option<Gotify>,
    base_url: Option<String>,
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
            gotify: None,
            base_url: None,
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

    pub fn gotify(self, gotify: Gotify) -> Self {
        Self {
            gotify: Some(gotify),
            ..self
        }
    }

    pub fn base_url(self, base_url: String) -> Self {
        Self {
            base_url: Some(base_url),
            ..self
        }
    }

    pub fn finish(self) -> Notification<R> {
        Notification {
            user_repo: self.user_repo,
            telegram: self.telegram,
            pushover: self.pushover,
            gotify: self.gotify,
            base_url: self.base_url,
        }
    }
}

#[derive(Clone)]
pub struct Notification<R>
where
    R: UserRepository,
{
    user_repo: R,
    pushover: Option<Pushover>,
    telegram: Option<Telegram>,
    base_url: Option<String>,
    gotify: Option<Gotify>,
}

impl<R> Notification<R>
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

        let url = self
            .base_url
            .as_ref()
            .map(|base_url| format!("{base_url}/chapter/{chapter_id}"));

        if let Some((user_key, pushover)) = user.pushover_user_key.zip(self.pushover.as_ref()) {
            if let Some(url) = &url {
                pushover
                    .send_notification_with_title_and_url(
                        &user_key,
                        manga_title,
                        chapter_title,
                        url,
                        "Read",
                    )
                    .await?;
            } else {
                pushover
                    .send_notification_with_title(&user_key, manga_title, chapter_title)
                    .await?;
            }
        }

        if let Some((chat_id, telegram)) = user.telegram_chat_id.zip(self.telegram.as_ref()) {
            let chat_id = &format!("{chat_id}");
            if let Some(url) = &url {
                telegram
                    .send_notification_with_title_and_url(
                        chat_id,
                        manga_title,
                        chapter_title,
                        url,
                        "Read",
                    )
                    .await?;
            } else {
                telegram
                    .send_notification_with_title(chat_id, manga_title, chapter_title)
                    .await?;
            }
        }

        if let Some((token, gotify)) = user.gotify_token.zip(self.gotify.as_ref()) {
            if let Some(url) = &url {
                gotify
                    .send_notification_with_title_and_url(
                        &token,
                        manga_title,
                        chapter_title,
                        url,
                        "Read",
                    )
                    .await?;
            } else {
                gotify
                    .send_notification_with_title(&token, manga_title, chapter_title)
                    .await?;
            }
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

    pub async fn send_message_to_gotify(
        &self,
        token: &str,
        title: Option<String>,
        body: &str,
    ) -> Result<(), anyhow::Error> {
        let gotify = self
            .gotify
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("gotify not set"))?;
        if let Some(title) = title {
            gotify
                .send_notification_with_title(token, &title, body)
                .await?;
        } else {
            gotify.send_notification(token, body).await?;
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
