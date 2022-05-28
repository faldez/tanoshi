use async_trait::async_trait;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlx::{Row, SqlitePool};

use crate::{
    domain::{
        entities::history::HistoryChapter,
        repositories::history::{HistoryRepository, HistoryRepositoryError},
    },
    infrastructure::database::Pool,
};

pub struct HistoryRepositoryImpl {
    pool: Pool,
}

impl HistoryRepositoryImpl {
    pub fn new<P: Into<Pool>>(pool: P) -> Self {
        Self { pool: pool.into() }
    }
}

#[async_trait]
impl HistoryRepository for HistoryRepositoryImpl {
    async fn get_first_history_chapters(
        &self,
        user_id: i64,
        after_timestamp: i64,
        before_timestamp: i64,
        first: i32,
    ) -> Result<Vec<HistoryChapter>, HistoryRepositoryError> {
        let chapters = sqlx::query(
            r#"
        SELECT
            manga.id,
            chapter.id,
            manga.title,
            manga.cover_url,
            chapter.title,
            MAX(user_history.read_at) AS read_at,
            user_history.last_page
        FROM user_history
        JOIN 
            chapter ON chapter.id = user_history.chapter_id AND
            user_history.user_id = ?
        JOIN manga ON manga.id = chapter.manga_id
        GROUP BY manga.id
        HAVING
            read_at < datetime(?, 'unixepoch') AND
            read_at > datetime(?, 'unixepoch')
        ORDER BY user_history.read_at DESC, manga.id DESC
        LIMIT ?"#,
        )
        .bind(user_id)
        .bind(after_timestamp)
        .bind(before_timestamp)
        .bind(first)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
        .map(|row| HistoryChapter {
            manga_id: row.get(0),
            chapter_id: row.get(1),
            manga_title: row.get(2),
            cover_url: row.get(3),
            chapter_title: row.get(4),
            read_at: row.get(5),
            last_page_read: row.get(6),
        })
        .collect();

        Ok(chapters)
    }

    async fn get_last_history_chapters(
        &self,
        user_id: i64,
        after_timestamp: i64,
        before_timestamp: i64,
        last: i32,
    ) -> Result<Vec<HistoryChapter>, HistoryRepositoryError> {
        let chapters = sqlx::query(
            r#"
        SELECT * FROM (
            SELECT
                manga.id,
                chapter.id,
                manga.title,
                manga.cover_url,
                chapter.title,
                MAX(user_history.read_at) AS read_at,
                user_history.last_page
            FROM user_history
            JOIN 
                chapter ON chapter.id = user_history.chapter_id AND
                user_history.user_id = ?
            JOIN manga ON manga.id = chapter.manga_id
            GROUP BY manga.id
            HAVING
                read_at < datetime(?, 'unixepoch') AND
                read_at > datetime(?, 'unixepoch')
            ORDER BY user_history.read_at ASC, manga.id ASC
            LIMIT ?) c ORDER BY c.read_at DESC, c.id DESC"#,
        )
        .bind(user_id)
        .bind(after_timestamp)
        .bind(before_timestamp)
        .bind(last)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
        .map(|row| HistoryChapter {
            manga_id: row.get(0),
            chapter_id: row.get(1),
            manga_title: row.get(2),
            cover_url: row.get(3),
            chapter_title: row.get(4),
            read_at: row.get(5),
            last_page_read: row.get(6),
        })
        .collect();

        Ok(chapters)
    }

    async fn get_history_chapters(
        &self,
        user_id: i64,
        after_timestamp: i64,
        before_timestamp: i64,
    ) -> Result<Vec<HistoryChapter>, HistoryRepositoryError> {
        let chapters = sqlx::query(
            r#"
        SELECT
            manga.id,
            chapter.id,
            manga.title,
            manga.cover_url,
            chapter.title,
            MAX(user_history.read_at) AS read_at,
            user_history.last_page
        FROM user_history
        JOIN 
            chapter ON chapter.id = user_history.chapter_id
            user_history.user_id = ?
        JOIN manga ON manga.id = chapter.manga_id
        GROUP BY manga.id
        HAVING
            read_at < datetime(?, 'unixepoch') AND
            read_at > datetime(?, 'unixepoch')
        ORDER BY user_history.read_at DESC, manga.id DESC"#,
        )
        .bind(user_id)
        .bind(after_timestamp)
        .bind(before_timestamp)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
        .map(|row| HistoryChapter {
            manga_id: row.get(0),
            chapter_id: row.get(1),
            manga_title: row.get(2),
            cover_url: row.get(3),
            chapter_title: row.get(4),
            read_at: row.get(5),
            last_page_read: row.get(6),
        })
        .collect();

        Ok(chapters)
    }

    async fn insert_history_chapter(
        &self,
        user_id: i64,
        chapter_id: i64,
        page: i64,
        is_complete: bool,
    ) -> Result<(), HistoryRepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO
            user_history(user_id, chapter_id, last_page, read_at, is_complete)
            VALUES(?, ?, ?, ?, ?)
            ON CONFLICT(user_id, chapter_id)
            DO UPDATE SET
            last_page = excluded.last_page,
            read_at = excluded.read_at,
            is_complete = CASE is_complete WHEN 0 THEN excluded.is_complete ELSE is_complete END"#,
        )
        .bind(user_id)
        .bind(chapter_id)
        .bind(page)
        .bind(chrono::Local::now())
        .bind(is_complete)
        .execute(&self.pool as &SqlitePool)
        .await?;

        Ok(())
    }

    async fn insert_history_chapters_as_completed(
        &self,
        user_id: i64,
        chapter_ids: &[i64],
    ) -> Result<(), HistoryRepositoryError> {
        if chapter_ids.is_empty() {
            return Ok(());
        }

        let now = chrono::Local::now();
        let query_str = format!(
            r#"
            INSERT INTO user_history(user_id, chapter_id, last_page, read_at, is_complete)
            VALUES {}
            ON CONFLICT(user_id, chapter_id)
            DO UPDATE SET
                last_page = excluded.last_page,
                read_at = excluded.read_at,
                is_complete = excluded.is_complete"#,
            vec!["(?, ?, 0, ?, true)"; chapter_ids.len()].join(",")
        );

        let mut query = sqlx::query(&query_str);

        for chapter_id in chapter_ids.iter() {
            query = query.bind(user_id).bind(chapter_id).bind(now);
        }

        query.execute(&self.pool as &SqlitePool).await?;

        Ok(())
    }

    async fn delete_chapters_from_history(
        &self,
        user_id: i64,
        chapter_ids: &[i64],
    ) -> Result<(), HistoryRepositoryError> {
        if chapter_ids.is_empty() {
            return Ok(());
        }

        let mut values = vec![];
        values.resize(chapter_ids.len(), "?");

        let query_str = format!(
            r#"DELETE FROM user_history
            WHERE user_id = ? AND chapter_id IN ({})"#,
            values.join(",")
        );

        let mut query = sqlx::query(&query_str).bind(user_id);

        for chapter_id in chapter_ids.iter() {
            query = query.bind(chapter_id);
        }

        query.execute(&self.pool as &SqlitePool).await?;

        Ok(())
    }
}
