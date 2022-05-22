use super::guard::AdminGuard;
use crate::{
    db::UserDatabase,
    infrastructure::auth::{self, Claims},
};
use async_graphql::{Context, InputObject, Object, Result};
use rand::RngCore;
use tanoshi_tracker::{anilist, myanimelist};

#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub is_admin: bool,
    telegram_chat_id: Option<i64>,
    pushover_user_key: Option<String>,
}

impl From<crate::db::model::User> for User {
    fn from(val: crate::db::model::User) -> Self {
        Self {
            id: val.id,
            username: val.username,
            password: val.password,
            is_admin: val.is_admin,
            telegram_chat_id: val.telegram_chat_id,
            pushover_user_key: val.pushover_user_key,
        }
    }
}

impl From<User> for crate::db::model::User {
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

    async fn myanimelist_status(&self, ctx: &Context<'_>) -> Result<bool> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        Ok(ctx
            .data::<UserDatabase>()?
            .get_user_tracker_token(myanimelist::NAME, user.sub)
            .await
            .is_ok())
    }

    async fn anilist_status(&self, ctx: &Context<'_>) -> Result<bool> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        Ok(ctx
            .data::<UserDatabase>()?
            .get_user_tracker_token(anilist::NAME, user.sub)
            .await
            .is_ok())
    }
}

#[derive(InputObject)]
struct ProfileInput {
    pub telegram_chat_id: Option<i64>,
    pub pushover_user_key: Option<String>,
}

#[derive(Default)]
pub struct UserRoot;

#[Object]
impl UserRoot {
    async fn login(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "username")] username: String,
        #[graphql(desc = "password")] password: String,
    ) -> Result<String> {
        let user = ctx
            .data::<UserDatabase>()?
            .get_user_by_username(username)
            .await?;

        if !argon2::verify_encoded(&user.password, password.as_bytes())? {
            return Err("Wrong username or password".into());
        }

        let current_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
        let token = auth::encode_jwt(&Claims {
            sub: user.id,
            username: user.username,
            is_admin: user.is_admin,
            exp: (current_time + std::time::Duration::from_secs(2678400)).as_secs() as usize, // 31 days
        })?;

        Ok(token)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let users = ctx.data::<UserDatabase>()?.get_users().await?;

        Ok(users.into_iter().map(|user| user.into()).collect())
    }

    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        let user = ctx
            .data::<UserDatabase>()?
            .get_user_by_id(user.sub)
            .await
            .map_err(|_| "user not exist, please relogin")?;

        Ok(user.into())
    }
}

#[derive(Default)]
pub struct UserMutationRoot;

#[Object]
impl UserMutationRoot {
    async fn register(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "username")] username: String,
        #[graphql(desc = "password")] password: String,
        #[graphql(desc = "role", default = false)] is_admin: bool,
    ) -> Result<i64> {
        let userdb = ctx.data::<UserDatabase>()?;

        let user_count = userdb.get_users_count().await?;
        if let Ok(claim) = ctx.data::<Claims>() {
            if user_count > 0 && !claim.is_admin {
                return Err("Forbidden".into());
            }
        }

        if password.len() < 8 {
            return Err("Password less than 8 character".into());
        }

        let mut salt: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        let config = argon2::Config::default();
        let hash = argon2::hash_encoded(password.as_bytes(), &salt, &config).unwrap();

        // If first user, make it admin else make it reader by default
        let is_admin = if user_count == 0 { true } else { is_admin };

        let user = crate::db::model::User {
            id: 0,
            username,
            password: hash,
            is_admin,
            ..Default::default()
        };

        let user_id = userdb.insert_user(user).await?;

        Ok(user_id)
    }

    async fn change_password(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "old password")] old_password: String,
        #[graphql(desc = "new password")] new_password: String,
    ) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let userdb = ctx.data::<UserDatabase>()?;

        let user = userdb.get_user_by_id(claims.sub).await?;

        if !argon2::verify_encoded(&user.password, old_password.as_bytes())? {
            return Err("Wrong old password".into());
        }

        if new_password.len() < 8 {
            return Err("Password less than 8 character".into());
        }

        let mut salt: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        let config = argon2::Config::default();
        let hash = argon2::hash_encoded(new_password.as_bytes(), &salt, &config).unwrap();

        let affected = userdb.update_password(user.id, hash).await?;

        Ok(affected)
    }

    async fn update_profile(&self, ctx: &Context<'_>, input: ProfileInput) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let userdb = ctx.data::<UserDatabase>()?;
        let mut user = userdb.get_user_by_id(claims.sub).await?;
        debug!("update_profile");

        user.telegram_chat_id = input.telegram_chat_id;
        user.pushover_user_key = input.pushover_user_key;

        let row = userdb.update_user_setting(&user).await?;

        debug!("update_profile: {} rows affected", row);

        Ok(row)
    }

    async fn tracker_logout(&self, ctx: &Context<'_>, tracker: String) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        Ok(ctx
            .data::<UserDatabase>()?
            .delete_user_tracker_login(&tracker, claims.sub)
            .await?)
    }
}
