use async_trait::async_trait;

use crate::Notifier;

#[derive(Clone)]
pub struct Gotify {
    client: reqwest::Client,
    base_url: String,
}

impl Gotify {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }
}

#[async_trait]
impl Notifier for Gotify {
    async fn send_notification(&self, token: &str, message: &str) -> Result<(), anyhow::Error> {
        self.client
            .post(&format!("{}/message", self.base_url))
            .query(&[("token", token)])
            .json(&serde_json::json!({ "message": message }))
            .send()
            .await?;

        Ok(())
    }

    async fn send_notification_with_title(
        &self,
        token: &str,
        title: &str,
        message: &str,
    ) -> Result<(), anyhow::Error> {
        self.client
            .post(&format!("{}/message", self.base_url))
            .query(&[("token", token)])
            .json(&serde_json::json!({ "message": message, "title": title }))
            .send()
            .await?;

        Ok(())
    }

    async fn send_notification_with_title_and_url(
        &self,
        token: &str,
        title: &str,
        message: &str,
        url: &str,
        _: &str,
    ) -> Result<(), anyhow::Error> {
        self.client
            .post(&format!("{}/message", self.base_url))
            .query(&[("token", token)])
            .json(&serde_json::json!({
                "message": message,
                "title": title,
                "extras": {
                    "client::notification": {
                        "click": { "url": url }
                    }
                }
            }))
            .send()
            .await?;

        Ok(())
    }
}
