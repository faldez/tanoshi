use async_trait::async_trait;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlx::{Row, SqlitePool};

use crate::{
    domain::{
        entities::download::{DownloadQueue, DownloadQueueEntry, DownloadedChapter},
        repositories::download::{DownloadRepository, DownloadRepositoryError},
    },
    infrastructure::database::Pool,
};

pub struct DownloadRepositoryImpl {
    pool: Pool,
}

impl DownloadRepositoryImpl {
    pub fn new<P: Into<Pool>>(pool: P) -> Self {
        Self { pool: pool.into() }
    }
}

#[async_trait]
impl DownloadRepository for DownloadRepositoryImpl {
    async fn get_first_downloaded_chapters(
        &self,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        first: i32,
    ) -> Result<Vec<DownloadedChapter>, DownloadRepositoryError> {
        let chapters = sqlx::query(
            r#"
        SELECT * FROM chapter
        WHERE
            (date_added, id) < (datetime(?, 'unixepoch'), ?) AND
            (date_added, id) > (datetime(?, 'unixepoch'), ?)
        WHERE downloaded_path IS NOT NULL
        ORDER BY date_added DESC, id DESC
        LIMIT ?"#,
        )
        .bind(after_timestamp)
        .bind(after_id)
        .bind(before_timestamp)
        .bind(before_id)
        .bind(first)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
        .map(|row| DownloadedChapter {
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
        })
        .collect();

        Ok(chapters)
    }

    async fn get_last_downloaded_chapters(
        &self,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        last: i32,
    ) -> Result<Vec<DownloadedChapter>, DownloadRepositoryError> {
        let chapters = sqlx::query(
            r#"
            SELECT * FROM (
                SELECT * FROM chapter
                WHERE
                    (date_added, id) < (datetime(?, 'unixepoch'), ?) AND
                    (date_added, id) > (datetime(?, 'unixepoch'), ?)
                WHERE downloaded_path IS NOT NULL
                ORDER BY date_added ASC, id ASC
                LIMIT ?) c
            ORDER BY c.date_added DESC, c.id DESC"#,
        )
        .bind(after_timestamp)
        .bind(after_id)
        .bind(before_timestamp)
        .bind(before_id)
        .bind(last)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
        .map(|row| DownloadedChapter {
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
        })
        .collect();

        Ok(chapters)
    }

    async fn get_downloaded_chapters(
        &self,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
    ) -> Result<Vec<DownloadedChapter>, DownloadRepositoryError> {
        let chapters = sqlx::query(
            r#"
            SELECT * FROM chapter
            WHERE
                (date_added, id) < (datetime(?, 'unixepoch'), ?) AND
                (date_added, id) > (datetime(?, 'unixepoch'), ?)
            WHERE downloaded_path IS NOT NULL
            ORDER BY date_added DESC, id DESC
            LIMIT ?"#,
        )
        .bind(after_timestamp)
        .bind(after_id)
        .bind(before_timestamp)
        .bind(before_id)
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .into_par_iter()
        .map(|row| DownloadedChapter {
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
        })
        .collect();

        Ok(chapters)
    }

    async fn get_chapter_downloaded_path(
        &self,
        chapter_id: i64,
    ) -> Result<String, DownloadRepositoryError> {
        let downloaded_path = sqlx::query(
            r#"
            SELECT downloaded_path FROM chapter WHERE id = ?"#,
        )
        .bind(chapter_id)
        .fetch_one(&self.pool as &SqlitePool)
        .await?
        .try_get(0)?;

        Ok(downloaded_path)
    }

    async fn update_chapter_downloaded_path(
        &self,
        chapter_id: i64,
        path: Option<String>,
    ) -> Result<(), DownloadRepositoryError> {
        sqlx::query(r#"UPDATE chapter SET downloaded_path = ? WHERE id = ?"#)
            .bind(path)
            .bind(chapter_id)
            .execute(&self.pool as &SqlitePool)
            .await?;

        Ok(())
    }

    async fn insert_download_queue(
        &self,
        items: &[DownloadQueue],
    ) -> Result<(), DownloadRepositoryError> {
        if items.is_empty() {
            return Ok(());
        }

        let mut values = vec![];
        values.resize(items.len(), "(?, ?, ?, ?, ?, ?, ?, ?, ?, ?)");

        let query_str = format!(
            r#"INSERT OR IGNORE INTO download_queue(
                source_id,
                source_name,
                manga_id,
                manga_title,
                chapter_id,
                chapter_title,
                rank,
                url,
                priority,
                date_added 
        ) VALUES {}"#,
            values.join(",")
        );

        let mut query = sqlx::query(&query_str);
        for item in items {
            query = query
                .bind(&item.source_id)
                .bind(&item.source_name)
                .bind(&item.manga_id)
                .bind(&item.manga_title)
                .bind(&item.chapter_id)
                .bind(&item.chapter_title)
                .bind(item.rank)
                .bind(&item.url)
                .bind(&item.priority)
                .bind(item.date_added.timestamp())
        }

        query.execute(&self.pool as &SqlitePool).await?;

        Ok(())
    }

    async fn get_single_download_queue(
        &self,
    ) -> Result<Option<DownloadQueue>, DownloadRepositoryError> {
        let data = sqlx::query(
            r#"SELECT 
                    id,
                    source_id,
                    source_name,
                    manga_id,
                    manga_title,
                    chapter_id,
                    chapter_title,
                    rank,
                    url,
                    priority,
                    date_added 
                FROM download_queue
                WHERE downloaded IS NOT true
                ORDER BY priority ASC, date_added ASC, chapter_id ASC, rank ASC
                LIMIT 1"#,
        )
        .fetch_optional(&self.pool as &SqlitePool)
        .await?
        .map(|row| DownloadQueue {
            id: row.get(0),
            source_id: row.get(1),
            source_name: row.get(2),
            manga_id: row.get(3),
            manga_title: row.get(4),
            chapter_id: row.get(5),
            chapter_title: row.get(6),
            rank: row.get(7),
            url: row.get(8),
            priority: row.get(9),
            date_added: row.get(10),
        });

        Ok(data)
    }

    async fn get_single_chapter_download_status(
        &self,
        chapter_id: i64,
    ) -> Result<bool, DownloadRepositoryError> {
        let row = sqlx::query(
            r#"SELECT SUM(downloaded) = COUNT(1)
                FROM download_queue
                WHERE chapter_id = ?"#,
        )
        .bind(chapter_id)
        .fetch_one(&self.pool as &SqlitePool)
        .await?;

        Ok(row.get(0))
    }

    async fn mark_single_download_queue_as_completed(
        &self,
        id: i64,
    ) -> Result<(), DownloadRepositoryError> {
        sqlx::query(r#"UPDATE download_queue SET downloaded = true WHERE id = ?"#)
            .bind(id)
            .execute(&self.pool as &SqlitePool)
            .await?;

        Ok(())
    }

    async fn get_download_queue_last_priority(
        &self,
    ) -> Result<Option<i64>, DownloadRepositoryError> {
        let data = sqlx::query(r#"SELECT MAX(priority) FROM download_queue"#)
            .fetch_optional(&self.pool as &SqlitePool)
            .await?
            .and_then(|row| row.try_get(0).ok());

        Ok(data)
    }

    async fn get_download_queue(&self) -> Result<Vec<DownloadQueueEntry>, DownloadRepositoryError> {
        let data = sqlx::query(
            r#"
            SELECT
                dq.source_id,
                dq.source_name,
                dq.manga_id,
                dq.manga_title, 
                dq.chapter_id,
                dq.chapter_title, 
                SUM(dq.downloaded),
                COUNT(1),
                dq.priority
            FROM download_queue dq
            GROUP BY dq.chapter_id
            ORDER BY dq.priority ASC, dq.date_added ASC, dq.chapter_id ASC"#,
        )
        .fetch_all(&self.pool as &SqlitePool)
        .await?
        .iter()
        .map(|row| DownloadQueueEntry {
            source_id: row.get(0),
            source_name: row.get(1),
            manga_id: row.get(2),
            manga_title: row.get(3),
            chapter_id: row.get(4),
            chapter_title: row.get(5),
            downloaded: row.get(6),
            total: row.get(7),
            priority: row.get(8),
        })
        .collect();

        Ok(data)
    }

    async fn delete_single_chapter_download_queue(
        &self,
        chapter_id: i64,
    ) -> Result<(), DownloadRepositoryError> {
        sqlx::query(r#"DELETE FROM download_queue WHERE chapter_id = ?"#)
            .bind(chapter_id)
            .execute(&self.pool as &SqlitePool)
            .await?;

        Ok(())
    }

    async fn delete_download_queue_by_chapter_id(
        &self,
        id: i64,
    ) -> Result<(), DownloadRepositoryError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("UPDATE download_queue SET priority = priority - 1 WHERE priority > (SELECT priority FROM download_queue WHERE chapter_id = ? LIMIT 1)").bind(id).execute(&mut tx).await?;

        sqlx::query("DELETE FROM download_queue WHERE chapter_id = ?")
            .bind(id)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn update_download_queue_priority(
        &self,
        chapter_id: i64,
        priority: i64,
    ) -> Result<(), DownloadRepositoryError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(r#"UPDATE download_queue SET priority = priority - 1 WHERE priority > (SELECT priority FROM download_queue WHERE chapter_id = ?)"#)
            .bind(chapter_id)
            .execute(&mut tx)
            .await?;

        sqlx::query(r#"UPDATE download_queue SET priority = priority + 1 WHERE priority >= ?"#)
            .bind(priority)
            .execute(&mut tx)
            .await?;

        sqlx::query(r#"UPDATE download_queue SET priority = ? WHERE chapter_id = ?"#)
            .bind(priority)
            .bind(chapter_id)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
