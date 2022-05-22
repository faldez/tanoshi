use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

mod manga;
pub use manga::Db as MangaDatabase;

pub mod model;

pub async fn establish_connection(database_path: &str) -> Result<SqlitePool, anyhow::Error> {
    let opts = SqliteConnectOptions::new()
        .create_if_missing(true)
        .filename(database_path);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .idle_timeout(std::time::Duration::from_secs(60))
        .max_lifetime(std::time::Duration::from_secs(3 * 60))
        .connect_with(opts)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
