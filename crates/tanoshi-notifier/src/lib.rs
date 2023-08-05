#[macro_use]
extern crate log;

pub mod gotify;
pub mod pushover;
pub mod telegram;

use async_trait::async_trait;

#[async_trait]
pub trait Notifier {
    async fn send_notification(&self, user_key: &str, message: &str) -> Result<(), anyhow::Error>;

    async fn send_notification_with_title(
        &self,
        user_key: &str,
        title: &str,
        message: &str,
    ) -> Result<(), anyhow::Error>;

    async fn send_notification_with_title_and_url(
        &self,
        user_key: &str,
        title: &str,
        message: &str,
        url: &str,
        url_title: &str,
    ) -> Result<(), anyhow::Error>;
}
