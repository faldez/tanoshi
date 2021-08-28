use crate::context::GlobalContext;
use async_graphql::{Context, Object, Result};
use rand::RngCore;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub is_admin: bool,
    pub exp: usize,
}

use async_graphql::SimpleObject;

#[derive(Debug, Default, SimpleObject)]
pub struct Settings {
    telegram_chat_id: Option<i64>,
}

#[derive(Debug, SimpleObject)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[graphql(skip)]
    pub password: String,
    pub is_admin: bool,
    pub settings: Settings,
}

impl From<crate::db::model::User> for User {
    fn from(val: crate::db::model::User) -> Self {
        Self {
            id: val.id,
            username: val.username,
            password: val.password,
            is_admin: val.is_admin,
            settings: Settings {
                telegram_chat_id: val.telegram_chat_id,
            },
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
            .data_unchecked::<GlobalContext>()
            .userdb
            .get_user_by_username(username)
            .await?;

        if !argon2::verify_encoded(&user.password, password.as_bytes())? {
            return Err("Wrong username or password".into());
        }

        let secret = ctx.data::<GlobalContext>()?.secret.clone();
        let token = jsonwebtoken::encode(
            &Header::default(),
            &Claims {
                sub: user.id,
                username: user.username,
                is_admin: user.is_admin,
                exp: 10000000000,
            },
            &EncodingKey::from_secret(secret.as_bytes()),
        )?;

        Ok(token)
    }

    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let is_admin = check_is_admin(&ctx)?;
        if !is_admin {
            return Err("Forbidden".into());
        };

        let users = ctx.data::<GlobalContext>()?.userdb.get_users().await?;

        Ok(users.into_iter().map(|user| user.into()).collect())
    }

    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let user = get_claims(ctx)?;
        let user = ctx
            .data::<GlobalContext>()?
            .userdb
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
        let userdb = &ctx.data::<GlobalContext>()?.userdb;

        let user_count = userdb.get_users_count().await?;

        if user_count > 0 && !check_is_admin(&ctx)? {
            return Err("Forbidden".into());
        };

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
        let claims = get_claims(ctx)?;

        let userdb = &ctx.data::<GlobalContext>()?.userdb;

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

    async fn update_profile(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "telegram chat id")] telegram_chat_id: Option<i64>,
    ) -> Result<u64> {
        debug!("update_profile");
        let claims = get_claims(ctx)?;

        let userdb = &ctx.data::<GlobalContext>()?.userdb;
        let mut user = userdb.get_user_by_id(claims.sub).await?;
        debug!("update_profile");

        user.telegram_chat_id = telegram_chat_id;

        let row = userdb.update_user_setting(&user).await?;

        debug!("update_profile: {} rows affected", row);

        Ok(row)
    }
}

pub fn get_claims(ctx: &Context<'_>) -> Result<Claims> {
    let token = ctx
        .data::<String>()
        .map_err(|_| "token not exists, please login")?;
    let secret = ctx.data::<GlobalContext>()?.secret.clone();
    let claims = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| format!("failed to decode token, reason: {}", e))?;

    Ok(claims.claims)
}

pub fn check_is_admin(ctx: &Context<'_>) -> Result<bool> {
    Ok(get_claims(ctx)?.is_admin)
}
