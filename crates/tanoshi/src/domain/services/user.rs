use rand::RngCore;
use thiserror::Error;

use crate::domain::{
    entities::user::User,
    repositories::user::{UserRepository, UserRepositoryError},
};

#[derive(Debug, Error)]
pub enum UserError {
    #[error("user not found")]
    UserNotFound,
    #[error("incorrect password")]
    WrongPassword,
    #[error("forbidden")]
    Forbidden,
    #[error("insufficient password length")]
    InsufficientPasswordLength,
    #[error("repository error: {0}")]
    RepositoryError(#[from] UserRepositoryError),
    #[error("other: {0}")]
    Other(String),
}

#[derive(Clone)]
pub struct UserService<R>
where
    R: UserRepository,
{
    repo: R,
}

impl<R> UserService<R>
where
    R: UserRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
        is_admin: bool,
    ) -> Result<i64, UserError> {
        if password.len() < 8 {
            return Err(UserError::InsufficientPasswordLength);
        }

        let mut salt: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        let hash = {
            let config = argon2::Config::default();
            argon2::hash_encoded(password.as_bytes(), &salt, &config)
                .map_err(|e| UserError::Other(format!("{e}")))?
        };

        let user = User {
            username: username.to_string(),
            password: hash,
            is_admin,
            ..Default::default()
        };

        Ok(self.repo.insert_user(user).await?)
    }

    pub async fn verify_password(&self, username: &str, password: &str) -> Result<(), UserError> {
        let user = self.repo.get_user_by_username(username.to_owned()).await?;

        if !argon2::verify_encoded(&user.password, password.as_bytes())
            .map_err(|e| UserError::Other(format!("{e}")))?
        {
            return Err(UserError::WrongPassword);
        }

        Ok(())
    }

    pub async fn change_password(
        &self,
        user_id: i64,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), UserError> {
        let user = self.repo.get_user_by_id(user_id).await?;

        if !argon2::verify_encoded(&user.password, old_password.as_bytes())
            .map_err(|e| UserError::Other(format!("{e}")))?
        {
            return Err(UserError::Other("Wrong old password".to_string()));
        }

        if new_password.len() < 8 {
            return Err(UserError::InsufficientPasswordLength);
        }

        let mut salt: [u8; 32] = [0; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        let hash = {
            let config = argon2::Config::default();
            argon2::hash_encoded(new_password.as_bytes(), &salt, &config)
                .map_err(|e| UserError::Other(format!("{e}")))?
        };

        self.repo.update_password(user.id, hash).await?;

        Ok(())
    }

    pub async fn update_profile(
        &self,
        user_id: i64,
        telegram_chat_id: Option<i64>,
        pushover_user_key: Option<String>,
        gotify_token: Option<String>,
    ) -> Result<(), UserError> {
        debug!("update_profile");

        let mut user = self.repo.get_user_by_id(user_id).await?;

        user.telegram_chat_id = telegram_chat_id;
        user.pushover_user_key = pushover_user_key;
        user.gotify_token = gotify_token;

        self.repo.update_user_setting(&user).await?;

        Ok(())
    }

    pub async fn fetch_all_users(&self) -> Result<Vec<User>, UserError> {
        Ok(self.repo.get_users().await?)
    }

    pub async fn fetch_user_by_id(&self, user_id: i64) -> Result<User, UserError> {
        Ok(self.repo.get_user_by_id(user_id).await?)
    }

    pub async fn fetch_user_by_username(&self, username: &str) -> Result<User, UserError> {
        Ok(self.repo.get_user_by_username(username.to_string()).await?)
    }
}
