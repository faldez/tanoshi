use async_graphql::{Context, Object, Result, SimpleObject};

use crate::{
    domain::services::user::UserService,
    infrastructure::domain::repositories::user::UserRepositoryImpl,
};

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
        let activated = ctx
            .data::<UserService<UserRepositoryImpl>>()?
            .fetch_all_users()
            .await?
            .len()
            > 0;
        let version = env!("CARGO_PKG_VERSION").to_string();

        Ok(Status { activated, version })
    }
}
