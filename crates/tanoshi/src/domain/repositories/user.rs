use async_trait::async_trait;
use thiserror::Error;

use crate::domain::entities::user::User;

#[derive(Debug, Error)]
pub enum UserRepositoryError {
    #[error("query return nothing")]
    NotFound,
    #[error("database return error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert_user(&self, user: User) -> Result<i64, UserRepositoryError>;

    async fn update_password(&self, id: i64, password: String) -> Result<u64, UserRepositoryError>;

    async fn update_user_is_admin(
        &self,
        id: i64,
        is_admin: bool,
    ) -> Result<u64, UserRepositoryError>;

    async fn get_users(&self) -> Result<Vec<User>, UserRepositoryError>;

    async fn get_users_count(&self) -> Result<i64, UserRepositoryError>;

    async fn get_admins(&self) -> Result<Vec<User>, UserRepositoryError>;

    async fn get_user_by_id(&self, id: i64) -> Result<User, UserRepositoryError>;

    async fn get_user_by_username(&self, username: String) -> Result<User, UserRepositoryError>;

    async fn update_user_setting(&self, user: &User) -> Result<u64, UserRepositoryError>;
}
