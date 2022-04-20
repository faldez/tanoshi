use async_graphql::{Context, Guard, Result};

use crate::auth::Claims;

#[derive(Debug, Default)]
pub struct AdminGuard;

impl AdminGuard {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl Guard for AdminGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        if claims.is_admin {
            return Ok(());
        }

        Err("Forbidden".into())
    }
}
