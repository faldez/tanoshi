use sqlx::{
    migrate::MigrateDatabase,
    sqlite::{SqlitePool, SqlitePoolOptions},
    Sqlite,
};

mod manga;
pub use manga::Db as MangaDatabase;

mod user;
pub use user::Db as UserDatabase;

pub mod model;

pub async fn establish_connection(database_path: &str) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    if !Sqlite::database_exists(database_path).await? {
        Sqlite::create_database(database_path).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(25)
        .connect(&database_path)
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}