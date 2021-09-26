use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};

mod manga;
pub use manga::Db as MangaDatabase;

mod user;
pub use user::Db as UserDatabase;

pub mod model;

pub async fn establish_connection(
    database_path: &str,
) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    if !Sqlite::database_exists(database_path).await? {
        Sqlite::create_database(database_path).await?;
    }

    let pool = SqlitePool::connect(database_path).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
