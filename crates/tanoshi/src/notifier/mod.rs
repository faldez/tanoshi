pub mod telegram;

use crate::{context::GlobalContext, user, worker::Command as WorkerCommand};
use async_graphql::{Context, Object, Result};

#[derive(Default)]
pub struct NotificationRoot;

#[Object]
impl NotificationRoot {
    async fn test_telegram(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "telegram chat id")] chat_id: i64,
    ) -> Result<bool> {
        let _ = user::get_claims(ctx)?;
        let ctx = ctx.data::<GlobalContext>()?;
        ctx.worker_tx.send(WorkerCommand::TelegramMessage(
            chat_id,
            "Test Notification".to_string(),
        ))?;

        Ok(true)
    }
}
