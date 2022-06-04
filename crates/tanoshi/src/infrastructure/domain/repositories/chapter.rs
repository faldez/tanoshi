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

    async fn get_chapters_by_manga_id(
        &self,
        manga_id: i64,
    ) -> Result<Vec<Chapter>, ChapterRepositoryError> {
        let chapters = sqlx::query(
            r#"SELECT 
                        chapter.*,
                        (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number > chapter.number ORDER BY c.number ASC LIMIT 1) next,
                        (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number < chapter.number ORDER BY c.number DESC LIMIT 1) prev
                    FROM chapter WHERE manga_id = ? ORDER BY number DESC"#,
        )
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
}
