use crate::{
    domain::{
        entities::user::User,
        repositories::user::{UserRepository, UserRepositoryError},
    },
    infrastructure::database::Pool,
};
use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::{sqlite::SqliteArguments, Arguments, Row, SqlitePool};
use tokio_stream::StreamExt;

#[derive(Clone)]
pub struct UserRepositoryImpl {
    pool: Pool,
}

impl UserRepositoryImpl {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn insert_user(&self, user: User) -> Result<i64, UserRepositoryError> {
        let row_id = sqlx::query(
            r#"INSERT INTO user(
                username,
                password,
                is_admin
            ) VALUES (?, ?, ?)"#,
        )
        .bind(&user.username)
        .bind(&user.password)
        .bind(user.is_admin)
        .execute(&self.pool as &SqlitePool)
        .await?
        .last_insert_rowid();

        Ok(row_id)
    }

    async fn update_password(&self, id: i64, password: String) -> Result<u64, UserRepositoryError> {
        let row_id = sqlx::query(
            r#"UPDATE user
                SET password = ?
                WHERE id = ?"#,
        )
        .bind(&password)
        .bind(id)
        .execute(&self.pool as &SqlitePool)
        .await?
        .rows_affected();

        Ok(row_id)
    }

    #[allow(dead_code)]
    async fn update_user_is_admin(
        &self,
        id: i64,
        is_admin: bool,
    ) -> Result<u64, UserRepositoryError> {
        let row_id = sqlx::query(
            r#"UPDATE user
                SET is_admin = ?
                WHERE id = ?"#,
        )
        .bind(&is_admin)
        .bind(id)
        .execute(&self.pool as &SqlitePool)
        .await?
        .rows_affected();

        Ok(row_id)
    }

    async fn get_users(&self) -> Result<Vec<User>, UserRepositoryError> {
        let users = sqlx::query(r#"SELECT * FROM user"#)
            .fetch_all(&self.pool as &SqlitePool)
            .await?
            .into_iter()
            .map(|row| User {
                id: row.get(0),
                username: row.get(1),
                password: row.get(2),
                is_admin: row.get(3),
                created_at: row.get(4),
                updated_at: row.get(5),
                telegram_chat_id: row.get(6),
                pushover_user_key: row.get(7),
            })
            .collect();

        Ok(users)
    }

    async fn get_users_count(&self) -> Result<i64, UserRepositoryError> {
        let row = sqlx::query(r#"SELECT COUNT(1) FROM user"#)
            .fetch_one(&self.pool as &SqlitePool)
            .await?;

        Ok(row.get(0))
    }

    async fn get_admins(&self) -> Result<Vec<User>, UserRepositoryError> {
        let mut stream = sqlx::query(r#"SELECT * FROM user WHERE is_admin = true"#)
            .fetch(&self.pool as &SqlitePool);

        let mut users = vec![];
        while let Some(row) = stream.try_next().await? {
            users.push(User {
                id: row.get(0),
                username: row.get(1),
                password: row.get(2),
                is_admin: row.get(3),
                created_at: row.get(4),
                updated_at: row.get(5),
                telegram_chat_id: row.get(6),
                pushover_user_key: row.get(7),
            });
        }
        Ok(users)
    }

    async fn get_user_by_id(&self, id: i64) -> Result<User, UserRepositoryError> {
        let row = sqlx::query(r#"SELECT * FROM user WHERE id = ?"#)
            .bind(id)
            .fetch_one(&self.pool as &SqlitePool)
            .await?;

        Ok(User {
            id: row.get(0),
            username: row.get(1),
            password: row.get(2),
            is_admin: row.get(3),
            created_at: row.get(4),
            updated_at: row.get(5),
            telegram_chat_id: row.get(6),
            pushover_user_key: row.get(7),
        })
    }

    async fn get_user_by_username(&self, username: String) -> Result<User, UserRepositoryError> {
        let row = sqlx::query(r#"SELECT * FROM user WHERE username = ?"#)
            .bind(&username)
            .fetch_one(&self.pool as &SqlitePool)
            .await?;

        Ok(User {
            id: row.get(0),
            username: row.get(1),
            password: row.get(2),
            is_admin: row.get(3),
            created_at: row.get(4),
            updated_at: row.get(5),
            telegram_chat_id: row.get(6),
            pushover_user_key: row.get(7),
        })
    }

    async fn update_user_setting(&self, user: &User) -> Result<u64, UserRepositoryError> {
        let mut column_to_update = vec![];
        let mut arguments = SqliteArguments::default();

        column_to_update.push("telegram_chat_id = ?");
        column_to_update.push("pushover_user_key = ?");
        arguments.add(user.telegram_chat_id);
        arguments.add(user.pushover_user_key.clone());
        arguments.add(user.id);

        if column_to_update.is_empty() {
            return Err(UserRepositoryError::Other(anyhow!("Nothing to update")));
        }

        let query = format!(
            r#"UPDATE user SET
                {}
                WHERE id = ?"#,
            column_to_update.join(",")
        );

        let rows_affected = sqlx::query_with(&query, arguments)
            .execute(&self.pool as &SqlitePool)
            .await?
            .rows_affected();

        Ok(rows_affected)
    }
}
