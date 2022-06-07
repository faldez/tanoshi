use async_trait::async_trait;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlx::{Row, SqlitePool};

use crate::{
    domain::{
        entities::chapter::Chapter,
        repositories::chapter::{ChapterRepository, ChapterRepositoryError},
    },
    infrastructure::database::Pool,
};

#[derive(Clone)]
pub struct ChapterRepositoryImpl {
    pool: Pool,
}

impl ChapterRepositoryImpl {
    pub fn new<P: Into<Pool>>(pool: P) -> Self {
        Self { pool: pool.into() }
    }
}

#[async_trait]
impl ChapterRepository for ChapterRepositoryImpl {
    async fn insert_chapters(&self, chapters: &[Chapter]) -> Result<(), ChapterRepositoryError> {
        if chapters.is_empty() {
            return Ok(());
        }

        let mut values = vec![];
        values.resize(chapters.len(), "(?, ?, ?, ?, ?, ?, ?, ?)");

        let query_str = format!(
            r#"INSERT INTO chapter(
            source_id,
            manga_id,
            title,
            path,
            number,
            scanlator,
            uploaded,
            date_added
        ) VALUES {} ON CONFLICT(source_id, path) DO UPDATE SET
            manga_id=excluded.manga_id,
            title=excluded.title,
            number=excluded.number,
            scanlator=excluded.scanlator,
            uploaded=excluded.uploaded,
            date_added=excluded.date_added
        "#,
            values.join(",")
        );

        let mut query = sqlx::query(&query_str);
        for chapter in chapters {
            query = query
                .bind(chapter.source_id)
                .bind(chapter.manga_id)
                .bind(&chapter.title)
                .bind(&chapter.path)
                .bind(chapter.number)
                .bind(&chapter.scanlator)
                .bind(chapter.uploaded)
                .bind(chrono::NaiveDateTime::from_timestamp(
                    chrono::Local::now().timestamp(),
                    0,
                ));
        }

        query.execute(&self.pool as &SqlitePool).await?;

        Ok(())
    }

    async fn get_chapter_by_id(&self, id: i64) -> Result<Chapter, ChapterRepositoryError> {
        let row = sqlx::query(
            r#"SELECT 
                        chapter.*,
                        (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number > chapter.number ORDER BY c.number ASC LIMIT 1) next,
                        (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number < chapter.number ORDER BY c.number DESC LIMIT 1) prev
                    FROM chapter WHERE id = ?"#,
        )
        .bind(id)
        .fetch_one(&self.pool as &SqlitePool)
        .await?;

        Ok(Chapter {
            id: row.get(0),
            source_id: row.get(1),
            manga_id: row.get(2),
            title: row.get(3),
            path: row.get(4),
            number: row.get(5),
            scanlator: row.get(6),
            uploaded: row.get(7),
            date_added: row.get(8),
            downloaded_path: row.get(9),
            next: row.get(10),
            prev: row.get(11),
        })
    }

    async fn get_chapter_by_source_id_path(
        &self,
        source_id: i64,
        path: &str,
    ) -> Result<Chapter, ChapterRepositoryError> {
        let row = sqlx::query(
            r#"SELECT 
                        chapter.*,
                        (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number > chapter.number ORDER BY c.number ASC LIMIT 1) next,
                        (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number < chapter.number ORDER BY c.number DESC LIMIT 1) prev
                    FROM chapter WHERE source_id = ? AND path = ?"#,
        )
        .bind(source_id)
        .bind(path)
        .fetch_one(&self.pool as &SqlitePool)
        .await?;

        Ok(Chapter {
            id: row.get(0),
            source_id: row.get(1),
            manga_id: row.get(2),
            title: row.get(3),
            path: row.get(4),
            number: row.get(5),
            scanlator: row.get(6),
            uploaded: row.get(7),
            date_added: row.get(8),
            downloaded_path: row.get(9),
            next: row.get(10),
            prev: row.get(11),
        })
    }

    async fn get_chapters_by_manga_id(
        &self,
        manga_id: i64,
        limit: Option<i64>,
        order_by: Option<&'static str>,
        asc: bool,
    ) -> Result<Vec<Chapter>, ChapterRepositoryError> {
        let limit = limit
            .map(|limit| format!("LIMIT {limit}"))
            .unwrap_or_else(|| "".to_string());
        let order_by = order_by.unwrap_or("number");
        let order = if asc { "ASC" } else { "DESC" };

        let query_str = format!(
            r#"SELECT
                        chapter.*,
                        (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number > chapter.number ORDER BY c.number ASC LIMIT 1) next,
                        (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number < chapter.number ORDER BY c.number DESC LIMIT 1) prev
                    FROM chapter WHERE manga_id = ? ORDER BY {order_by} {order} {limit}"#,
        );
        let chapters = sqlx::query(&query_str)
            .bind(manga_id)
            .fetch_all(&self.pool as &SqlitePool)
            .await?
            .into_par_iter()
            .map(|row| Chapter {
                id: row.get(0),
                source_id: row.get(1),
                manga_id: row.get(2),
                title: row.get(3),
                path: row.get(4),
                number: row.get(5),
                scanlator: row.get(6),
                uploaded: row.get(7),
                date_added: row.get(8),
                downloaded_path: row.get(9),
                next: row.get(10),
                prev: row.get(11),
            })
            .collect();

        Ok(chapters)
    }

    async fn delete_chapter_by_id(&self, chapter_id: i64) -> Result<(), ChapterRepositoryError> {
        sqlx::query("DELETE FROM chapter WHERE id = ?")
            .bind(chapter_id)
            .execute(&self.pool as &SqlitePool)
            .await?;

        Ok(())
    }

    async fn delete_chapter_by_ids(
        &self,
        chapter_ids: &[i64],
    ) -> Result<(), ChapterRepositoryError> {
        if chapter_ids.is_empty() {
            return Err(ChapterRepositoryError::BadArgsError(
                "chapter_ids should at least be 1".to_string(),
            ));
        }

        let query_str = format!(
            "DELETE FROM chapter WHERE id IN ({})",
            vec!["?"; chapter_ids.len()].join(",")
        );

        let mut query = sqlx::query(&query_str);

        for chapter_id in chapter_ids {
            query = query.bind(chapter_id);
        }

        query.execute(&self.pool as &SqlitePool).await?;

        Ok(())
    }

    async fn get_chapters_not_in_source(
        &self,
        source_id: i64,
        manga_id: i64,
        paths: &[String],
    ) -> Result<Vec<Chapter>, ChapterRepositoryError> {
        if paths.is_empty() {
            return Err(ChapterRepositoryError::BadArgsError(
                "paths should at least be 1".to_string(),
            ));
        }

        let query_str = format!(
            "SELECT * FROM chapter WHERE source_id = ? AND manga_id = ? AND path NOT IN ({})",
            vec!["?"; paths.len()].join(",")
        );

        let mut query = sqlx::query(&query_str).bind(source_id).bind(manga_id);

        for path in paths.iter() {
            query = query.bind(path);
        }

        let chapters = query
            .fetch_all(&self.pool as &SqlitePool)
            .await?
            .into_par_iter()
            .map(|row| Chapter {
                id: row.get(0),
                source_id: row.get(1),
                manga_id: row.get(2),
                title: row.get(3),
                path: row.get(4),
                number: row.get(5),
                scanlator: row.get(6),
                uploaded: row.get(7),
                date_added: row.get(8),
                downloaded_path: row.get(9),
                next: None,
                prev: None,
            })
            .collect();

        Ok(chapters)
    }
}
