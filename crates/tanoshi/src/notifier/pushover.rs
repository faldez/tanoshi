use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};

const PUSHOVER_ENDPOINT: &str = "https://api.pushover.net/1/messages.json";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct Payload {
    // your application's API token
    pub token: String,
    // the user/group key (not e-mail address) of your user (or you), viewable when logged into our dashboard
    pub user: String,
    // your message
    pub message: String,
    // an image attachment to send with the message; see attachments for more information on how to upload files
    pub attachment: String,
    // your user's device name to send the message directly to that device, rather than all of the user's devices (multiple devices may be separated by a comma)
    pub device: String,
    // your message's title, otherwise your app's name is used
    pub title: String,
    // a supplementary URL to show with your message
    pub url: String,
    // a title for your supplementary URL, otherwise just the URL is shown
    pub url_title: String,
    // send as -2 to generate no notification/alert, -1 to always send as a quiet notification, 1 to display as high-priority and bypass the user's quiet hours, or 2 to also require confirmation from the user
    pub priority: String,
    // the name of one of the sounds supported by device clients to override the user's default sound choice
    pub sound: String,
    // a Unix timestamp of your message's date and time to display to the user, rather than the time your message is received by our A
    pub timestamp: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct Response {
    pub status: i32,
}

pub struct Pushover {
    client: reqwest::Client,
    token: String,
}

impl Pushover {
    pub fn new(token: String) -> Pushover {
        let client = reqwest::Client::new();
        Pushover { client, token }
    }

    async fn send_payload(&self, payload: &Payload) -> Result<(), Error> {
        let res: Response = self
            .client
            .post(PUSHOVER_ENDPOINT)
            .body(serde_json::to_string(payload)?)
            .send()
            .await?
            .json()
            .await?;

        if res.status != 1 {
            return Err(anyhow!("error push test notification"));
        }

        Ok(())
    }

    pub async fn send_notification(&self, user_key: &str, message: &str) -> Result<(), Error> {
        let payload = Payload {
            token: self.token.clone(),
            user: user_key.to_string(),
            message: message.to_string(),
            ..Default::default()
        };

        self.send_payload(&payload).await?;

        Ok(())
    }

    pub async fn send_notification_with_title(
        &self,
        user_key: &str,
        title: &str,
        message: &str,
    ) -> Result<(), Error> {
        let payload = Payload {
            token: self.token.clone(),
            user: user_key.to_string(),
            title: title.to_string(),
            message: message.to_string(),
            ..Default::default()
        };

        self.send_payload(&payload).await?;

        Ok(())
    }
}
