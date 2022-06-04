use crate::{
    domain::{
        entities::manga::Manga,
        repositories::manga::{MangaRepository, MangaRepositoryError},
    },
    infrastructure::database::Pool,
};
use async_trait::async_trait;
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct MangaRepositoryImpl {
    pool: Pool,
}

impl MangaRepositoryImpl {
    pub fn new<P: Into<Pool>>(pool: P) -> Self {
        Self { pool: pool.into() }
    }
}

#[async_trait]
impl MangaRepository for MangaRepositoryImpl {
    async fn get_manga_by_id(&self, id: i64) -> Result<Manga, MangaRepositoryError> {
        let row = sqlx::query(r#"SELECT * FROM manga WHERE id = ?"#)
            .bind(id)
            .fetch_one(&self.pool as &SqlitePool)
            .await?;

        Ok(Manga {
            id: row.get(0),
            source_id: row.get(1),
            title: row.get(2),
            author: serde_json::from_str(row.get::<String, _>(3).as_str()).unwrap_or_default(),
            genre: serde_json::from_str(row.get::<String, _>(4).as_str()).unwrap_or_default(),
            status: row.get(5),
            description: row.get(6),
            path: row.get(7),
            cover_url: row.get(8),
            date_added: row.get(9),
        })
    }

    async fn get_manga_by_ids(&self, ids: &[i64]) -> Result<Vec<Manga>, MangaRepositoryError> {
        let query_str = format!(
            r#"SELECT * FROM manga WHERE id IN ({})"#,
            vec!["?"; ids.len()].join(",")
        );
        let mut query = sqlx::query(&query_str);
        for id in ids {
            query = query.bind(id);
        }
        let manga = query
            .fetch_all(&self.pool as &SqlitePool)
            .await?
            .iter()
            .map(|row| Manga {
                id: row.get(0),
                source_id: row.get(1),
                title: row.get(2),
                author: serde_json::from_str(row.get::<String, _>(3).as_str()).unwrap_or_default(),
                genre: serde_json::from_str(row.get::<String, _>(4).as_str()).unwrap_or_default(),
                status: row.get(5),
                description: row.get(6),
                path: row.get(7),
                cover_url: row.get(8),
                date_added: row.get(9),
            })
            .collect();

        Ok(manga)
    }

    async fn get_manga_by_source_path(
        &self,
        source_id: i64,
        path: &str,
    ) -> Result<Manga, MangaRepositoryError> {
        let row = sqlx::query(r#"SELECT * FROM manga WHERE source_id = ? AND path = ?"#)
            .bind(source_id)
            .bind(path)
            .fetch_one(&self.pool as &SqlitePool)
            .await?;

        Ok(Manga {
            id: row.get(0),
            source_id: row.get(1),
            title: row.get(2),
            author: serde_json::from_str(row.get::<String, _>(3).as_str()).unwrap_or_default(),
            genre: serde_json::from_str(row.get::<String, _>(4).as_str()).unwrap_or_default(),
            status: row.get(5),
            description: row.get(6),
            path: row.get(7),
            cover_url: row.get(8),
            date_added: row.get(9),
        })
    }

    async fn insert_manga(&self, manga: &mut Manga) -> Result<(), MangaRepositoryError> {
        let row_id = sqlx::query(
            r#"
            INSERT INTO manga(
                source_id,
                title,
                author,
                genre,
                status,
                description,
                path,
                cover_url,
                date_added
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(source_id, path)
            DO UPDATE SET
                title=excluded.title,
                author=excluded.author,
                genre=excluded.genre,
                status=excluded.status,
                description=excluded.description,
                date_added=excluded.date_added,
                cover_url=excluded.cover_url
        "#,
        )
        .bind(manga.source_id)
        .bind(&manga.title)
        .bind(serde_json::to_string(&manga.author).unwrap_or_else(|_| "[]".to_string()))
        .bind(serde_json::to_string(&manga.genre).unwrap_or_else(|_| "[]".to_string()))
        .bind(&manga.status)
        .bind(&manga.description)
        .bind(&manga.path)
        .bind(&manga.cover_url)
        .bind(chrono::NaiveDateTime::from_timestamp(
            chrono::Local::now().timestamp(),
            0,
        ))
        .execute(&self.pool as &SqlitePool)
        .await?
        .last_insert_rowid();

        if manga.id == 0 {
            manga.id = row_id;
        }

        Ok(())
    }
}
