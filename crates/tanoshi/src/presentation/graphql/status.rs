use async_graphql::{Context, Object, Result, SimpleObject};

use crate::db::UserDatabase;

#[derive(Debug, SimpleObject)]
struct Status {
    activated: bool,
    version: String,
}

#[derive(Default)]
pub struct StatusRoot;

#[Object]
impl StatusRoot {
    async fn server_status(&self, ctx: &Context<'_>) -> Result<Status> {
        let activated = ctx.data::<UserDatabase>()?.get_users_count().await? > 0;
        let version = env!("CARGO_PKG_VERSION").to_string();

        Ok(Status { activated, version })
    }
}
