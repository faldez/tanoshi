use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlx::{Row, SqlitePool};

use crate::{
    domain::{
        entities::history::HistoryChapter,
        repositories::history::{HistoryRepository, HistoryRepositoryError},
    },
    infrastructure::database::Pool,
};

#[derive(Clone)]
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
            user_history.last_page,
            user_history.is_complete
        FROM user_history
        JOIN chapter ON 
            user_history.user_id = ? AND
            chapter.id = user_history.chapter_id
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
            is_complete: row.get(7),
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
                user_history.last_page,
                user_history.is_complete
            FROM user_history
            JOIN chapter ON 
                user_history.user_id = ? AND
                chapter.id = user_history.chapter_id
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
            is_complete: row.get(7),
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
            user_history.last_page,
            user_history.is_complete
        FROM user_history
        JOIN chapter ON 
            user_history.user_id = ? AND
            chapter.id = user_history.chapter_id
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
            is_complete: row.get(7),
        })
        .collect();

        Ok(chapters)
    }

    async fn get_history_chapters_by_manga_ids(
        &self,
        user_id: i64,
        manga_ids: &[i64],
    ) -> Result<Vec<HistoryChapter>, HistoryRepositoryError> {
        let query_str = format!(
            r#"SELECT
                    manga.id,
                    chapter.id,
                    manga.title,
                    manga.cover_url,
                    chapter.title,
                    user_history.read_at,
                    user_history.last_page,
                    user_history.is_complete
                FROM user_history
                JOIN chapter ON 
                    chapter.id = user_history.chapter_id AND
                    chapter.manga_id IN ({})
                JOIN manga ON 
                    manga.id = chapter.manga_id
                WHERE user_history.user_id = ?"#,
            vec!["?"; manga_ids.len()].join(",")
        );

        let mut query = sqlx::query(&query_str);

        for manga_id in manga_ids {
            query = query.bind(manga_id);
        }

        let chapters = query
            .bind(user_id)
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
                is_complete: row.get(7),
            })
            .collect();

        Ok(chapters)
    }

    async fn get_history_chapters_by_chapter_ids(
        &self,
        user_id: i64,
        chapter_ids: &[i64],
    ) -> Result<Vec<HistoryChapter>, HistoryRepositoryError> {
        let query_str = format!(
            r#"SELECT
                    manga.id,
                    chapter.id,
                    manga.title,
                    manga.cover_url,
                    chapter.title,
                    user_history.read_at,
                    user_history.last_page,
                    user_history.is_complete
                FROM user_history
                JOIN chapter ON 
                    chapter.id = user_history.chapter_id
                JOIN manga ON manga.id = chapter.manga_id
                WHERE user_history.user_id = ? AND user_history.chapter_id IN ({})"#,
            vec!["?"; chapter_ids.len()].join(",")
        );

        let mut query = sqlx::query(&query_str).bind(user_id);

        for chapter_id in chapter_ids {
            query = query.bind(chapter_id);
        }

        let chapters = query
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
                is_complete: row.get(7),
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
            INSERT INTO user_history(user_id, chapter_id, last_page, read_at, is_complete)
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
        .bind(Utc::now().naive_utc())
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

        let now = Utc::now().naive_utc();
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
            r#"DELETE FROM user_history WHERE user_id = ? AND chapter_id IN ({})"#,
            values.join(",")
        );

        let mut query = sqlx::query(&query_str).bind(user_id);

        for chapter_id in chapter_ids.iter() {
            query = query.bind(chapter_id);
        }

        query.execute(&self.pool as &SqlitePool).await?;

        Ok(())
    }

    async fn get_unread_chapters_by_manga_ids(
        &self,
        user_id: i64,
        manga_ids: &[i64],
    ) -> Result<HashMap<i64, i64>, HistoryRepositoryError> {
        let mut values = vec![];
        values.resize(manga_ids.len(), "?");

        let query_str = format!(
            r#"SELECT manga_id, COUNT(1) FROM (
                SELECT manga_id, IFNULL(user_history.is_complete, false) AS is_complete 
                FROM chapter c 
                LEFT JOIN user_history ON 
                    user_history.user_id = ? AND 
                    user_history.chapter_id = c.id 
                WHERE c.manga_id IN ({})
            )
            WHERE is_complete = false
            GROUP BY manga_id"#,
            values.join(",")
        );

        let mut query = sqlx::query(&query_str).bind(user_id);
        for manga_id in manga_ids {
            query = query.bind(manga_id)
        }

        let data = query
            .fetch_all(&self.pool as &SqlitePool)
            .await?
            .iter()
            .map(|row| (row.get(0), row.get(1)))
            .collect();

        Ok(data)
    }

    async fn get_next_chapter_by_manga_id(
        &self,
        user_id: i64,
        manga_id: i64,
    ) -> Result<Option<i64>, HistoryRepositoryError> {
        let chapter_id = sqlx::query(
            r#"
            WITH last_reading_session AS (
                SELECT
                    chapter_id,
                    is_complete,
                    number as chapter_number
                FROM
                    user_history
                    INNER JOIN chapter on chapter.id = user_history.chapter_id
                    AND manga_id = ?
                WHERE
                    user_history.user_id = ?
                ORDER BY
                    user_history.read_at DESC
                LIMIT
                    1
            ), first_unread_chapter AS (
                SELECT
                    id
                FROM
                    chapter
                    LEFT JOIN user_history ON user_history.chapter_id = chapter.id
                    AND user_history.user_id = ?
                WHERE
                    manga_id = ?
                    AND user_history.is_complete IS NOT true
                ORDER BY
                    chapter.number ASC
                LIMIT
                    1
            ), resume_chapter AS (
                SELECT
                    COALESCE(
                        CASE
                            WHEN is_complete THEN (
                                SELECT
                                    id
                                FROM
                                    chapter
                                    LEFT JOIN user_history ON user_history.chapter_id = chapter.id
                                    AND user_history.user_id = ?
                                WHERE
                                    chapter.number > chapter_number
                                    AND manga_id = ?
                                    AND user_history.is_complete IS NOT true
                                ORDER BY
                                    number ASC
                                LIMIT
                                    1
                            )
                            ELSE chapter_id
                        END,
                        first_unread_chapter.id
                    ) AS id
                FROM
                    (SELECT null)
                    LEFT JOIN first_unread_chapter
                    LEFT JOIN last_reading_session
            )
            SELECT
                chapter.id
            FROM
                chapter
            WHERE
                chapter.id = (
                    SELECT
                        id
                    FROM
                        resume_chapter
                )"#,
        )
        .bind(manga_id)
        .bind(user_id)
        .bind(user_id)
        .bind(manga_id)
        .bind(user_id)
        .bind(manga_id)
        .fetch_optional(&self.pool as &SqlitePool)
        .await?
        .map(|row| row.get(0));

        Ok(chapter_id)
    }
}
