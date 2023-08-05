use super::guard::AdminGuard;
use crate::{
    domain::services::{tracker::TrackerService, user::UserService},
    infrastructure::{
        auth::{self, Claims},
        config::Config,
        domain::repositories::{tracker::TrackerRepositoryImpl, user::UserRepositoryImpl},
    },
};
use async_graphql::{Context, InputObject, Object, Result};
use tanoshi_tracker::{anilist, myanimelist};

#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub is_admin: bool,
    telegram_chat_id: Option<i64>,
    pushover_user_key: Option<String>,
    gotify_token: Option<String>,
}

impl From<crate::domain::entities::user::User> for User {
    fn from(val: crate::domain::entities::user::User) -> Self {
        Self {
            id: val.id,
            username: val.username,
            password: val.password,
            is_admin: val.is_admin,
            telegram_chat_id: val.telegram_chat_id,
            pushover_user_key: val.pushover_user_key,
            gotify_token: val.gotify_token,
        }
    }
}

impl From<User> for crate::domain::entities::user::User {
    fn from(val: User) -> Self {
        Self {
            id: val.id,
            username: val.username,
            password: val.password,
            is_admin: val.is_admin,
            ..Default::default()
        }
    }
}

#[Object]
impl User {
    async fn id(&self) -> i64 {
        self.id
    }

    async fn username(&self) -> String {
        self.username.clone()
    }

    async fn is_admin(&self) -> bool {
        self.is_admin
    }

    async fn telegram_chat_id(&self) -> Option<i64> {
        self.telegram_chat_id
    }

    async fn pushover_user_key(&self) -> Option<String> {
        self.pushover_user_key.clone()
    }

    async fn gotify_token(&self) -> Option<String> {
        self.gotify_token.clone()
    }

    async fn myanimelist_status(&self, ctx: &Context<'_>) -> Result<bool> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        Ok(ctx
            .data::<TrackerService<TrackerRepositoryImpl>>()?
            .check_tracker_login(myanimelist::NAME, user.sub)
            .await
            .is_ok())
    }

    async fn anilist_status(&self, ctx: &Context<'_>) -> Result<bool> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        Ok(ctx
            .data::<TrackerService<TrackerRepositoryImpl>>()?
            .check_tracker_login(anilist::NAME, user.sub)
            .await
            .is_ok())
    }
}

#[derive(InputObject)]
struct ProfileInput {
    pub telegram_chat_id: Option<i64>,
    pub pushover_user_key: Option<String>,
    pub gotify_token: Option<String>,
}

#[derive(InputObject)]
struct LoginInput {
    username: String,
    #[graphql(secret)]
    password: String,
}

#[derive(InputObject)]
struct ChangePasswordInput {
    #[graphql(secret)]
    old_password: String,
    #[graphql(secret)]
    new_password: String,
}

#[derive(Default)]
pub struct UserRoot;

#[Object]
impl UserRoot {
    async fn login(&self, ctx: &Context<'_>, login: LoginInput) -> Result<String> {
        let user_svc = ctx.data::<UserService<UserRepositoryImpl>>()?;

        user_svc
            .verify_password(&login.username, &login.password)
            .await?;

        let user = user_svc.fetch_user_by_username(&login.username).await?;

        let secret = &ctx.data::<Config>()?.secret;
        let current_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
        let claims = Claims {
            sub: user.id,
            username: user.username,
            is_admin: user.is_admin,
            exp: (current_time + std::time::Duration::from_secs(2678400)).as_secs() as usize, // 31 days
        };
        let token = auth::encode_jwt(secret, &claims)?;

        Ok(token)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let users = ctx
            .data::<UserService<UserRepositoryImpl>>()?
            .fetch_all_users()
            .await?;

        Ok(users.into_iter().map(|user| user.into()).collect())
    }

    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let claim = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let user = ctx
            .data::<UserService<UserRepositoryImpl>>()?
            .fetch_user_by_id(claim.sub)
            .await?
            .into();

        Ok(user)
    }
}

#[derive(Default)]
pub struct UserMutationRoot;

#[Object]
impl UserMutationRoot {
    async fn register(
        &self,
        ctx: &Context<'_>,
        login: LoginInput,
        #[graphql(desc = "role", default = false)] is_admin: bool,
    ) -> Result<i64> {
        let user_svc = ctx.data::<UserService<UserRepositoryImpl>>()?;

        let user_count = user_svc.fetch_all_users().await?.len();
        if let Ok(claim) = ctx.data::<Claims>() {
            if user_count > 0 && !claim.is_admin {
                return Err("Forbidden".into());
            }
        }

        Ok(user_svc
            .create_user(&login.username, &login.password, is_admin)
            .await?)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn delete_user(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "user id")] user_id: i64,
    ) -> Result<i64> {
        ctx.data::<UserService<UserRepositoryImpl>>()?
            .delete_user(user_id)
            .await?;

        Ok(1)
    }

    async fn change_password(&self, ctx: &Context<'_>, input: ChangePasswordInput) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<UserService<UserRepositoryImpl>>()?
            .change_password(claims.sub, &input.old_password, &input.new_password)
            .await?;

        Ok(1)
    }

    async fn update_profile(&self, ctx: &Context<'_>, input: ProfileInput) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<UserService<UserRepositoryImpl>>()?
            .update_profile(
                claims.sub,
                input.telegram_chat_id,
                input.pushover_user_key,
                input.gotify_token,
            )
            .await?;

        Ok(1)
    }

    async fn tracker_logout(&self, ctx: &Context<'_>, tracker: String) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<TrackerService<TrackerRepositoryImpl>>()?
            .logout_tracker(claims.sub, &tracker)
            .await?;

        Ok(1)
    }
}
