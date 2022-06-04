use std::collections::HashMap;

use async_trait::async_trait;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlx::{Row, SqlitePool};

use crate::{
    domain::{
        entities::{
            library::{Category, LibraryUpdate},
            manga::Manga,
        },
        repositories::library::{LibraryRepository, LibraryRepositoryError},
    },
    infrastructure::database::Pool,
};

#[derive(Clone)]
pub struct LibraryRepositoryImpl {
    pool: Pool,
}

impl LibraryRepositoryImpl {
    pub fn new<P: Into<Pool>>(pool: P) -> Self {
        Self { pool: pool.into() }
    }
}

#[async_trait]
impl LibraryRepository for LibraryRepositoryImpl {
    async fn get_categories_by_user_id(
        &self,
        user_id: i64,
    ) -> Result<Vec<Category>, LibraryRepositoryError> {
        let categories = sqlx::query(
            r#"SELECT
                id,
                name
            FROM user_category
            WHERE user_id = ?
            ORDER BY name"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
        .map(|row| Category {
            id: row.get(0),
            name: row.get(1),
        })
        .collect();

        Ok(categories)
    }

    async fn get_category_by_id(&self, id: i64) -> Result<Category, LibraryRepositoryError> {
        let row = sqlx::query(
            r#"SELECT
                    id,
                    name
                FROM user_category
                WHERE id = ?"#,
        )
        .bind(id)
        .fetch_one(&self.pool as &SqlitePool)
        .await?;

        Ok(Category {
            id: row.get(0),
            name: row.get(1),
        })
    }

    async fn create_category(
        &self,
        user_id: i64,
        name: &str,
    ) -> Result<Category, LibraryRepositoryError> {
        let row = sqlx::query(
            "INSERT INTO user_category (user_id, name) VALUES (?, ?) RETURNING id, name",
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(&self.pool as &SqlitePool)
        .await?;

        Ok(Category {
            id: row.get(0),
            name: row.get(1),
        })
    }

    async fn rename_category(
        &self,
        id: i64,
        name: &str,
    ) -> Result<Category, LibraryRepositoryError> {
        let row = sqlx::query("UPDATE user_category SET name = ? WHERE id = ? RETURNING id, name")
            .bind(name)
            .bind(id)
            .fetch_one(&self.pool as &SqlitePool)
            .await?;

        Ok(Category {
            id: row.get(0),
            name: row.get(1),
        })
    }

    async fn delete_category(&self, id: i64) -> Result<(), LibraryRepositoryError> {
        sqlx::query("DELETE FROM user_category WHERE id = ?")
            .bind(id)
            .execute(&self.pool as &SqlitePool)
            .await?;

        Ok(())
    }

    async fn get_category_count(
        &self,
        user_id: i64,
    ) -> Result<HashMap<Option<i64>, i64>, LibraryRepositoryError> {
        let data = sqlx::query(
            "SELECT user_category.id, COUNT(1) FROM manga
        INNER JOIN user_library ON user_library.user_id = ? AND manga.id = user_library.manga_id
        LEFT JOIN library_category ON user_library.id = library_category.library_id
        LEFT JOIN user_category ON library_category.category_id = user_category.id
        GROUP BY user_category.id",
        )
        .bind(user_id)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
        .map(|row| (row.get(0), row.get(1)))
        .collect();

        Ok(data)
    }

    async fn get_manga_from_library(
        &self,
        user_id: i64,
    ) -> Result<Vec<Manga>, LibraryRepositoryError> {
        let manga = sqlx::query(
            r#"SELECT manga.*, library_category.category_id FROM manga
            INNER JOIN user_library ON user_library.user_id = ? AND manga.id = user_library.manga_id
            LEFT JOIN library_category ON user_library.id = library_category.library_id
            ORDER BY title"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
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

    async fn get_manga_from_library_by_category_id(
        &self,
        user_id: i64,
        category_id: Option<i64>,
    ) -> Result<Vec<Manga>, LibraryRepositoryError> {
        let manga = sqlx::query(
            r#"SELECT manga.*, library_category.category_id FROM manga
            INNER JOIN user_library ON user_library.user_id = ? AND manga.id = user_library.manga_id
            LEFT JOIN library_category ON user_library.id = library_category.library_id
            WHERE category_id IS ?
            ORDER BY title"#,
        )
        .bind(user_id)
        .bind(category_id)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
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

    async fn insert_manga_to_library(
        &self,
        user_id: i64,
        manga_id: i64,
        category_ids: &[i64],
    ) -> Result<(), LibraryRepositoryError> {
        let mut tx = self.pool.begin().await?;

        let library_id = sqlx::query("INSERT INTO user_library(user_id, manga_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(manga_id)
            .execute(&mut tx)
            .await
            .map(|res| res.last_insert_rowid())?;

        if !category_ids.is_empty() {
            let query_str = format!(
                "INSERT INTO library_category(library_id, category_id) VALUES {}",
                vec!["(?,?)".to_string(); category_ids.len()].join(",")
            );

            let mut query = sqlx::query(&query_str);
            for category_id in category_ids {
                query = query.bind(library_id).bind(category_id);
            }
            query.execute(&mut tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_manga_from_library(
        &self,
        user_id: i64,
        manga_id: i64,
    ) -> Result<(), LibraryRepositoryError> {
        sqlx::query("DELETE FROM user_library WHERE user_id = ? AND manga_id = ?")
            .bind(user_id)
            .bind(manga_id)
            .execute(&self.pool as &SqlitePool)
            .await?;

        Ok(())
    }

    async fn get_first_library_updates(
        &self,
        user_id: i64,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        first: i32,
    ) -> Result<Vec<LibraryUpdate>, LibraryRepositoryError> {
        let chapters = sqlx::query(
            r#"
        SELECT
            manga.id,
            chapter.id,
            manga.title,
            manga.cover_url,
            chapter.title,
            chapter.uploaded
        FROM chapter
        JOIN manga ON manga.id = chapter.manga_id
        JOIN user_library ON
            user_library.manga_id = manga.id
            AND user_library.user_id = ?
        WHERE
            (uploaded, chapter.id) < (datetime(?, 'unixepoch'), ?) AND
            (uploaded, chapter.id) > (datetime(?, 'unixepoch'), ?)
        ORDER BY chapter.uploaded DESC, chapter.number DESC
        LIMIT ?"#,
        )
        .bind(user_id)
        .bind(after_timestamp)
        .bind(after_id)
        .bind(before_timestamp)
        .bind(before_id)
        .bind(first)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
        .map(|row| LibraryUpdate {
            manga_id: row.get(0),
            chapter_id: row.get(1),
            manga_title: row.get(2),
            cover_url: row.get(3),
            chapter_title: row.get(4),
            uploaded: row.get(5),
        })
        .collect();

        Ok(chapters)
    }

    async fn get_last_library_updates(
        &self,
        user_id: i64,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        last: i32,
    ) -> Result<Vec<LibraryUpdate>, LibraryRepositoryError> {
        let chapters = sqlx::query(
            r#"
        SELECT * FROM (
            SELECT
                manga.id,
                chapter.id,
                manga.title,
                manga.cover_url,
                chapter.title,
                chapter.uploaded,
                chapter.number
            FROM chapter
            JOIN manga ON manga.id = chapter.manga_id
            JOIN user_library ON
                user_library.manga_id = manga.id
                AND user_library.user_id = ?
            WHERE
                (uploaded, chapter.id) < (datetime(?, 'unixepoch'), ?) AND
                (uploaded, chapter.id) > (datetime(?, 'unixepoch'), ?)
            ORDER BY chapter.uploaded ASC, chapter.number DESC
            LIMIT ?) c
        ORDER BY c.uploaded DESC, c.number DESC"#,
        )
        .bind(user_id)
        .bind(after_timestamp)
        .bind(after_id)
        .bind(before_timestamp)
        .bind(before_id)
        .bind(last)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
        .map(|row| LibraryUpdate {
            manga_id: row.get(0),
            chapter_id: row.get(1),
            manga_title: row.get(2),
            cover_url: row.get(3),
            chapter_title: row.get(4),
            uploaded: row.get(5),
        })
        .collect();

        Ok(chapters)
    }

    async fn get_library_updates(
        &self,
        user_id: i64,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
    ) -> Result<Vec<LibraryUpdate>, LibraryRepositoryError> {
        let chapters = sqlx::query(
            r#"
        SELECT
            manga.id,
            chapter.id,
            manga.title,
            manga.cover_url,
            chapter.title,
            chapter.uploaded
        FROM chapter
        JOIN manga ON manga.id = chapter.manga_id
        JOIN user_library ON
            user_library.manga_id = manga.id
            AND user_library.user_id = ?
        WHERE
            (uploaded, chapter.id) < (datetime(?, 'unixepoch'), ?) AND
            (uploaded, chapter.id) > (datetime(?, 'unixepoch'), ?)
        ORDER BY chapter.uploaded DESC, chapter.number DESC"#,
        )
        .bind(user_id)
        .bind(after_timestamp)
        .bind(after_id)
        .bind(before_timestamp)
        .bind(before_id)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
        .map(|row| LibraryUpdate {
            manga_id: row.get(0),
            chapter_id: row.get(1),
            manga_title: row.get(2),
            cover_url: row.get(3),
            chapter_title: row.get(4),
            uploaded: row.get(5),
        })
        .collect();

        Ok(chapters)
    }
}
