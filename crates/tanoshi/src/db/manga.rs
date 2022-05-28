use std::collections::HashMap;

use super::model::*;
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlx::{
    sqlite::{SqliteArguments, SqlitePool},
    Arguments, Row,
};
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub fn new(pool: SqlitePool) -> Db {
        Db { pool }
    }

    pub async fn get_manga_by_id(&self, id: i64) -> Result<Manga> {
        let mut conn = self.pool.acquire().await?;
        let stream = sqlx::query(r#"SELECT * FROM manga WHERE id = ?"#)
            .bind(id)
            .fetch_one(&mut conn)
            .await;

        Ok(stream.map(|row| Manga {
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
        })?)
    }

    pub async fn get_manga_by_ids(&self, ids: &[i64]) -> Result<Vec<Manga>> {
        let mut conn = self.pool.acquire().await?;
        let query_str = format!(
            r#"SELECT * FROM manga WHERE id IN ({})"#,
            vec!["?"; ids.len()].join(",")
        );
        let mut query = sqlx::query(&query_str);
        for id in ids {
            query = query.bind(id);
        }
        let manga = query
            .fetch_all(&mut conn)
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

    pub async fn get_manga_by_source_path(&self, source_id: i64, path: &str) -> Result<Manga> {
        let mut conn = self.pool.acquire().await?;
        let stream = sqlx::query(r#"SELECT * FROM manga WHERE source_id = ? AND path = ?"#)
            .bind(source_id)
            .bind(path)
            .fetch_one(&mut conn)
            .await;

        Ok(stream.map(|row| Manga {
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
        })?)
    }

    #[allow(dead_code)]
    pub async fn get_library(&self, user_id: i64) -> Result<Vec<Manga>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
            r#"SELECT manga.* FROM manga
                    JOIN user_library ON manga.id = user_library.manga_id AND user_library.user_id = ?
                    ORDER BY title"#,
        )
        .bind(user_id)
        .fetch(&mut conn);

        let mut mangas = vec![];
        while let Some(row) = stream.try_next().await? {
            mangas.push(Manga {
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
            });
        }
        Ok(mangas)
    }

    pub async fn get_library_by_category_id(
        &self,
        user_id: i64,
        category_id: Option<i64>,
    ) -> Result<Vec<Manga>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
            r#"SELECT manga.*, library_category.category_id FROM manga
            INNER JOIN user_library ON user_library.user_id = ? AND manga.id = user_library.manga_id
            LEFT JOIN library_category ON user_library.id = library_category.library_id
            WHERE category_id IS ?
            ORDER BY title"#,
        )
        .bind(user_id)
        .bind(category_id)
        .fetch(&mut conn);

        let mut mangas = vec![];
        while let Some(row) = stream.try_next().await? {
            mangas.push(Manga {
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
            });
        }
        Ok(mangas)
    }

    pub async fn count_library_by_category_id(
        &self,
        user_id: i64,
        category_id: Option<i64>,
    ) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(
            r#"SELECT COUNT(1) FROM manga
            INNER JOIN user_library ON user_library.user_id = ? AND manga.id = user_library.manga_id
            LEFT JOIN library_category ON user_library.id = library_category.library_id
            WHERE category_id IS ?"#,
        )
        .bind(user_id)
        .bind(category_id)
        .fetch_one(&mut conn)
        .await?;

        Ok(row.get::<i64, _>(0))
    }

    pub async fn count_library_by_category(&self, user_id: i64) -> Result<Vec<(Option<i64>, i64)>> {
        let mut conn = self.pool.acquire().await?;
        let rows = sqlx::query(
            r#"SELECT user_category.id, COUNT(1) FROM manga
            INNER JOIN user_library ON user_library.user_id = 1 AND manga.id = user_library.manga_id
            LEFT JOIN library_category ON user_library.id = library_category.library_id
            LEFT JOIN user_category ON library_category.category_id = user_category.id
            GROUP BY user_category.id;"#,
        )
        .bind(user_id)
        .fetch_all(&mut conn)
        .await?
        .into_par_iter()
        .map(|row| (row.get(0), row.get(1)))
        .collect();

        Ok(rows)
    }

    pub async fn get_all_user_library(&self) -> Result<Vec<UserMangaLibrary>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
            r#"SELECT manga.*, JSON_GROUP_ARRAY(user.id) FROM manga
            JOIN user_library ON user_library.manga_id = manga.id
            JOIN user ON user.id = user_library.user_id
            GROUP BY user_library.manga_id"#,
        )
        .fetch(&mut conn);

        let mut mangas = vec![];
        while let Some(row) = stream.try_next().await? {
            mangas.push(UserMangaLibrary {
                user_ids: serde_json::from_str(row.get::<String, _>(10).as_str())
                    .unwrap_or_default(),
                manga: Manga {
                    id: row.get(0),
                    source_id: row.get(1),
                    title: row.get(2),
                    author: serde_json::from_str(row.get::<String, _>(3).as_str())
                        .unwrap_or_default(),
                    genre: serde_json::from_str(row.get::<String, _>(4).as_str())
                        .unwrap_or_default(),
                    status: row.get(5),
                    description: row.get(6),
                    path: row.get(7),
                    cover_url: row.get(8),
                    date_added: row.get(9),
                },
            });
        }
        Ok(mangas)
    }

    #[allow(dead_code)]
    pub async fn is_user_library(&self, user_id: i64, manga_id: i64) -> Result<bool> {
        let mut conn = self.pool.acquire().await?;
        let stream =
            sqlx::query(r#"SELECT true FROM user_library WHERE user_id = ? AND manga_id = ?"#)
                .bind(user_id)
                .bind(manga_id)
                .fetch_one(&mut conn)
                .await
                .ok();

        if let Some(row) = stream {
            Ok(row.get(0))
        } else {
            Ok(false)
        }
    }

    pub async fn is_user_library_by_manga_ids(
        &self,
        user_id: i64,
        manga_ids: &[i64],
    ) -> Result<HashMap<i64, bool>> {
        let mut conn = self.pool.acquire().await?;

        let mut values = vec![];
        values.resize(manga_ids.len(), "?");

        let query_str = format!(
            r#"SELECT manga_id FROM user_library 
            WHERE user_id = ? AND manga_id IN ({})"#,
            values.join(",")
        );

        let mut query = sqlx::query(&query_str).bind(user_id);
        for manga_id in manga_ids {
            query = query.bind(manga_id)
        }

        let data = query
            .fetch_all(&mut conn)
            .await?
            .iter()
            .map(|row| (row.get(0), true))
            .collect();

        Ok(data)
    }

    pub async fn is_user_library_by_manga_paths(
        &self,
        user_id: i64,
        manga_paths: &[String],
    ) -> Result<HashMap<String, bool>> {
        let mut conn = self.pool.acquire().await?;

        let mut values = vec![];
        values.resize(manga_paths.len(), "?");

        let query_str = format!(
            r#"SELECT manga.path FROM user_library
            JOIN manga ON manga.id = user_library.manga_id
            WHERE user_library.user_id = ? AND manga.path IN ({})"#,
            values.join(",")
        );

        let mut query = sqlx::query(&query_str).bind(user_id);
        for manga_path in manga_paths {
            query = query.bind(manga_path)
        }

        let data = query
            .fetch_all(&mut conn)
            .await?
            .iter()
            .map(|row| (row.get(0), true))
            .collect();

        Ok(data)
    }

    pub async fn get_recent_updates(
        &self,
        user_id: i64,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
    ) -> Result<Vec<RecentUpdate>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
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
        .fetch(&mut conn);

        let mut chapters = vec![];
        while let Some(row) = stream.try_next().await? {
            chapters.push(RecentUpdate {
                manga_id: row.get(0),
                chapter_id: row.get(1),
                manga_title: row.get(2),
                cover_url: row.get(3),
                chapter_title: row.get(4),
                uploaded: row.get(5),
            });
        }
        Ok(chapters)
    }

    pub async fn get_first_recent_updates(
        &self,
        user_id: i64,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        first: i32,
    ) -> Result<Vec<RecentUpdate>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
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
        .fetch(&mut conn);

        let mut chapters = vec![];
        while let Some(row) = stream.try_next().await? {
            chapters.push(RecentUpdate {
                manga_id: row.get(0),
                chapter_id: row.get(1),
                manga_title: row.get(2),
                cover_url: row.get(3),
                chapter_title: row.get(4),
                uploaded: row.get(5),
            });
        }
        Ok(chapters)
    }

    pub async fn get_last_recent_updates(
        &self,
        user_id: i64,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        last: i32,
    ) -> Result<Vec<RecentUpdate>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
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
        .fetch(&mut conn);

        let mut chapters = vec![];
        while let Some(row) = stream.try_next().await? {
            chapters.push(RecentUpdate {
                manga_id: row.get(0),
                chapter_id: row.get(1),
                manga_title: row.get(2),
                cover_url: row.get(3),
                chapter_title: row.get(4),
                uploaded: row.get(5),
            });
        }
        Ok(chapters)
    }

    pub async fn get_chapter_has_next_page(&self, user_id: i64, timestamp: i64, id: i64) -> bool {
        let mut conn = if let Ok(conn) = self.pool.acquire().await {
            conn
        } else {
            return false;
        };
        let stream = sqlx::query(
            r#"
            SELECT
                chapter.id as chapter_id,
                chapter.uploaded
            FROM chapter
            JOIN manga ON manga.id = chapter.manga_id
            JOIN user_library ON
                user_library.manga_id = manga.id
                AND user_library.user_id = ?
            WHERE
                (uploaded, chapter.id) < (datetime(?, 'unixepoch'), ?)
            ORDER BY chapter.uploaded DESC, chapter.id DESC"#,
        )
        .bind(user_id)
        .bind(timestamp)
        .bind(id)
        .fetch_one(&mut conn)
        .await
        .ok();

        let mut count = 0;
        if let Some(row) = stream {
            count = row.get(0);
        }
        count > 0
    }

    pub async fn get_chapter_has_before_page(&self, user_id: i64, timestamp: i64, id: i64) -> bool {
        let mut conn = if let Ok(conn) = self.pool.acquire().await {
            conn
        } else {
            return false;
        };
        let stream = sqlx::query(
            r#"
        SELECT
            chapter.id as chapter_id,
            chapter.uploaded
        FROM chapter
        JOIN manga ON manga.id = chapter.manga_id
        JOIN user_library ON
            user_library.manga_id = manga.id
            AND user_library.user_id = ?
        WHERE
            (uploaded, chapter.id) > (datetime(?, 'unixepoch'), ?)
        ORDER BY chapter.uploaded DESC, chapter.id DESC"#,
        )
        .bind(user_id)
        .bind(timestamp)
        .bind(id)
        .fetch_one(&mut conn)
        .await
        .ok();

        let mut count = 0;
        if let Some(row) = stream {
            count = row.get(0);
        }
        count > 0
    }

    #[allow(dead_code)]
    pub async fn get_chapter_len(&self) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let stream = sqlx::query(
            r#"
            SELECT COUNT(id)
            FROM chapter
            JOIN manga ON manga.id = chapter.manga_id
            WHERE manga.is_favorite = true"#,
        )
        .fetch_one(&mut conn)
        .await
        .ok();

        if let Some(row) = stream {
            Ok(row.get(0))
        } else {
            Err(anyhow::anyhow!("error count chapters"))
        }
    }

    pub async fn get_read_chapters(
        &self,
        after_timestamp: i64,
        before_timestamp: i64,
    ) -> Result<Vec<RecentChapter>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
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
        .bind(after_timestamp)
        .bind(before_timestamp)
        .fetch(&mut conn);

        let mut chapters = vec![];
        while let Some(row) = stream.try_next().await? {
            chapters.push(RecentChapter {
                manga_id: row.get(0),
                chapter_id: row.get(1),
                manga_title: row.get(2),
                cover_url: row.get(3),
                chapter_title: row.get(4),
                read_at: row.get(5),
                last_page_read: row.get(6),
            });
        }
        Ok(chapters)
    }

    pub async fn get_first_read_chapters(
        &self,
        user_id: i64,
        after_timestamp: i64,
        before_timestamp: i64,
        first: i32,
    ) -> Result<Vec<RecentChapter>> {
        let mut conn = self.pool.acquire().await?;

        let mut stream = sqlx::query(
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
        .fetch(&mut conn);

        let mut chapters = vec![];
        while let Some(row) = stream.try_next().await? {
            chapters.push(RecentChapter {
                manga_id: row.get(0),
                chapter_id: row.get(1),
                manga_title: row.get(2),
                cover_url: row.get(3),
                chapter_title: row.get(4),
                read_at: row.get(5),
                last_page_read: row.get(6),
            });
        }
        Ok(chapters)
    }

    pub async fn get_last_read_chapters(
        &self,
        user_id: i64,
        after_timestamp: i64,
        before_timestamp: i64,
        last: i32,
    ) -> Result<Vec<RecentChapter>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
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
        .fetch(&mut conn);

        let mut chapters = vec![];
        while let Some(row) = stream.try_next().await? {
            chapters.push(RecentChapter {
                manga_id: row.get(0),
                chapter_id: row.get(1),
                manga_title: row.get(2),
                cover_url: row.get(3),
                chapter_title: row.get(4),
                read_at: row.get(5),
                last_page_read: row.get(6),
            });
        }
        Ok(chapters)
    }

    pub async fn get_read_chapter_has_next_page(
        &self,
        user_id: i64,
        timestamp: i64,
        id: i64,
    ) -> bool {
        let mut conn = if let Ok(conn) = self.pool.acquire().await {
            conn
        } else {
            return false;
        };
        let stream = sqlx::query(
            r#"
            SELECT COUNT(1) FROM (
				SELECT
                	user_history.last_page,
                	MAX(user_history.read_at) as read_at
            	FROM user_history
            	JOIN 
                    chapter ON user_history.chapter_id = chapter.id AND
                    user_history.user_id = ? AND
                	chapter.manga_id <> ?
                GROUP BY chapter.manga_id
                HAVING read_at < datetime(?, 'unixepoch')
            	ORDER BY user_history.read_at DESC, chapter.manga_id DESC
            )"#,
        )
        .bind(user_id)
        .bind(id)
        .bind(timestamp)
        .fetch_one(&mut conn)
        .await
        .ok();

        let mut count = 0;
        if let Some(row) = stream {
            count = row.get(0);
        }
        count > 0
    }

    pub async fn get_read_chapter_has_before_page(
        &self,
        user_id: i64,
        timestamp: i64,
        id: i64,
    ) -> bool {
        let mut conn = if let Ok(conn) = self.pool.acquire().await {
            conn
        } else {
            return false;
        };

        let stream = sqlx::query(
            r#"
            SELECT COUNT(1) FROM (
				SELECT
                	user_history.last_page,
                	MAX(user_history.read_at) as read_at
            	FROM user_history
            	JOIN 
                    chapter ON user_history.chapter_id = chapter.id AND
                    user_history.user_id = ? AND
                	chapter.manga_id <> ?
                GROUP BY chapter.manga_id
                HAVING	user_history.read_at > datetime(?, 'unixepoch')
            	ORDER BY user_history.read_at DESC, chapter.manga_id DESC
            )"#,
        )
        .bind(user_id)
        .bind(id)
        .bind(timestamp)
        .fetch_one(&mut conn)
        .await
        .ok();

        let mut count = 0;
        if let Some(row) = stream {
            count = row.get(0);
        }
        count > 0
    }

    pub async fn insert_manga(&self, manga: &mut Manga) -> Result<()> {
        let mut conn = self.pool.acquire().await?;
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
        .execute(&mut conn)
        .await?
        .last_insert_rowid();

        if manga.id == 0 {
            manga.id = row_id;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn insert_mangas(&self, _manga: Vec<Manga>) -> Result<()> {
        todo!()
    }

    #[allow(dead_code)]
    pub async fn update_manga_info(&self, manga: &Manga) -> Result<u64> {
        let mut conn = self.pool.acquire().await?;
        let mut column_to_update = vec![];
        let mut arguments = SqliteArguments::default();
        if manga.source_id > 0 {
            column_to_update.push("source_id = ?");
            arguments.add(manga.source_id);
        }
        if !manga.title.is_empty() {
            column_to_update.push("title = ?");
            arguments.add(&manga.title);
        }
        if !manga.author.is_empty() {
            column_to_update.push("author = ?");
            arguments
                .add(serde_json::to_string(&manga.author).unwrap_or_else(|_| "[]".to_string()));
        }
        if !manga.genre.is_empty() {
            column_to_update.push("genre = ?");
            arguments.add(serde_json::to_string(&manga.genre).unwrap_or_else(|_| "[]".to_string()));
        }
        if manga.status.is_some() {
            column_to_update.push("status = ?");
            arguments.add(&manga.status);
        }
        if manga.description.is_some() {
            column_to_update.push("description = ?");
            arguments.add(&manga.description);
        }
        if !manga.path.is_empty() {
            column_to_update.push("path = ?");
            arguments.add(&manga.path);
        }
        if !manga.cover_url.is_empty() {
            column_to_update.push("cover_url = ?");
            arguments.add(&manga.cover_url);
        }

        if column_to_update.is_empty() {
            return Err(anyhow!("Nothing to update"));
        }

        let query = format!(
            r#"UPDATE manga SET
                {}
                WHERE id = ?"#,
            column_to_update.join(",")
        );

        let rows_affected = sqlx::query_with(&query, arguments)
            .execute(&mut conn)
            .await?
            .rows_affected();

        Ok(rows_affected)
    }

    pub async fn get_last_read_at_by_user_id_and_manga_ids(
        &self,
        user_id: i64,
        manga_ids: &[i64],
    ) -> Result<HashMap<i64, NaiveDateTime>> {
        let mut conn = self.pool.acquire().await?;

        let mut values = vec![];
        values.resize(manga_ids.len(), "?");

        let query_str = format!(
            r#"SELECT manga_id, read_at FROM (
                SELECT manga_id, MAX(user_history.read_at) as read_at FROM chapter
                JOIN user_history ON user_history.chapter_id = chapter.id AND user_history.user_id = ?
                WHERE chapter.manga_id IN ({})
                GROUP BY chapter.manga_id
            )
            WHERE read_at IS NOT NULL"#,
            values.join(",")
        );

        let mut query = sqlx::query(&query_str).bind(user_id);
        for manga_id in manga_ids {
            query = query.bind(manga_id)
        }
        let data = query
            .fetch_all(&mut conn)
            .await?
            .iter()
            .map(|row| (row.get(0), row.get::<chrono::NaiveDateTime, _>(1)))
            .collect();

        Ok(data)
    }

    pub async fn get_prev_chapter_id_by_ids(
        &self,
        chapter_ids: &[i64],
    ) -> Result<HashMap<i64, i64>> {
        let mut conn = self.pool.acquire().await?;

        let mut values = vec![];
        values.resize(chapter_ids.len(), "?");

        let query_str = format!(
            r#"SELECT id,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number < chapter.number ORDER BY c.number DESC LIMIT 1) prev
            FROM chapter WHERE id IN ({}) AND prev IS NOT NULL"#,
            values.join(",")
        );

        let mut query = sqlx::query(&query_str);
        for chapter_id in chapter_ids {
            query = query.bind(chapter_id)
        }

        let data = query
            .fetch_all(&mut conn)
            .await?
            .iter()
            .map(|row| (row.get(0), row.get(1)))
            .collect();

        Ok(data)
    }

    pub async fn get_next_chapter_id_by_ids(
        &self,
        chapter_ids: &[i64],
    ) -> Result<HashMap<i64, i64>> {
        let mut conn = self.pool.acquire().await?;

        let mut values = vec![];
        values.resize(chapter_ids.len(), "?");

        let query_str = format!(
            r#"SELECT id,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number > chapter.number ORDER BY c.number ASC LIMIT 1) next
            FROM chapter WHERE id IN ({}) AND next IS NOT NULL"#,
            values.join(",")
        );

        let mut query = sqlx::query(&query_str);
        for chapter_id in chapter_ids {
            query = query.bind(chapter_id)
        }

        let data = query
            .fetch_all(&mut conn)
            .await?
            .iter()
            .map(|row| (row.get(0), row.get(1)))
            .collect();

        Ok(data)
    }

    pub async fn get_chapter_by_id(&self, id: i64) -> Result<Chapter> {
        let mut conn = self.pool.acquire().await?;
        let stream = sqlx::query(
            r#"
            SELECT * FROM chapter WHERE id = ?"#,
        )
        .bind(id)
        .fetch_one(&mut conn)
        .await;

        Ok(stream.map(|row| Chapter {
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
        })?)
    }

    pub async fn get_next_chapter_by_manga_id(
        &self,
        user_id: i64,
        manga_id: i64,
    ) -> Result<Option<Chapter>> {
        let mut conn = self.pool.acquire().await?;
        let stream = sqlx::query(
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
                chapter.*
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
        .fetch_optional(&mut conn)
        .await?;

        Ok(stream.map(|row| Chapter {
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
        }))
    }

    pub async fn get_chapter_by_source_path(&self, source_id: i64, path: &str) -> Option<Chapter> {
        let mut conn = self.pool.acquire().await.ok()?;
        let stream = sqlx::query(
            r#"
            SELECT * FROM chapter WHERE source_id = ? AND path = ?"#,
        )
        .bind(source_id)
        .bind(path)
        .fetch_one(&mut conn)
        .await
        .ok();

        stream.map(|row| Chapter {
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
    }
    pub async fn get_chapters_by_manga_id(&self, manga_id: i64) -> Result<Vec<Chapter>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
            r#"
            SELECT * FROM chapter WHERE manga_id = ? ORDER BY number DESC"#,
        )
        .bind(manga_id)
        .fetch(&mut conn);

        let mut chapters = vec![];
        while let Some(row) = stream.try_next().await? {
            chapters.push(Chapter {
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
            });
        }
        if chapters.is_empty() {
            Err(anyhow::anyhow!("Chapters not found"))
        } else {
            Ok(chapters)
        }
    }

    pub async fn get_last_uploaded_chapters_by_manga_id(&self, manga_id: i64) -> Option<Chapter> {
        let mut conn = self.pool.acquire().await.ok()?;
        let stream = sqlx::query(
            r#"
            SELECT * FROM chapter WHERE manga_id = ? ORDER BY uploaded DESC LIMIT 1"#,
        )
        .bind(manga_id)
        .fetch_one(&mut conn)
        .await
        .ok();

        stream.map(|row| Chapter {
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
    }

    pub async fn get_chapters_by_manga_id_after(
        &self,
        manga_id: i64,
        after: NaiveDateTime,
    ) -> Result<Vec<Chapter>> {
        let mut conn = self.pool.acquire().await?;
        let stream = sqlx::query(
            r#"
            SELECT * FROM chapter WHERE manga_id = ? AND uploaded > ? ORDER BY uploaded ASC"#,
        )
        .bind(manga_id)
        .bind(after)
        .fetch_all(&mut conn)
        .await?;

        Ok(stream
            .iter()
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
            })
            .collect())
    }

    pub async fn insert_chapter(&self, chapter: &Chapter) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let row_id = sqlx::query(
            r#"INSERT INTO chapter(
                source_id,
                manga_id,
                title,
                path,
                number,
                scanlator,
                uploaded,
                date_added
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(source_id, path) DO UPDATE SET
            manga_id=excluded.manga_id,
            title=excluded.title,
            number=excluded.number,
            scanlator=excluded.scanlator,
            uploaded=excluded.uploaded,
            date_added=excluded.date_added"#,
        )
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
        ))
        .execute(&mut conn)
        .await?
        .last_insert_rowid();

        Ok(row_id)
    }

    pub async fn insert_chapters(&self, chapters: &[Chapter]) -> Result<()> {
        let mut conn = self.pool.acquire().await?;
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

        query.execute(&mut conn).await?;

        Ok(())
    }

    pub async fn insert_user_library(
        &self,
        user_id: i64,
        manga_id: i64,
        category_ids: Vec<i64>,
    ) -> Result<u64> {
        let mut tx = self.pool.begin().await?;
        let library_id = sqlx::query("INSERT INTO user_library(user_id, manga_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(manga_id)
            .execute(&mut tx)
            .await
            .map(|res| res.last_insert_rowid())
            .map_err(|e| anyhow::anyhow!(e))?;

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

        Ok(library_id as _)
    }

    pub async fn delete_user_library(&self, user_id: i64, manga_id: i64) -> Result<u64> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query("DELETE FROM user_library WHERE user_id = ? AND manga_id = ?")
            .bind(user_id)
            .bind(manga_id)
            .execute(&mut conn)
            .await
            .map(|res| res.rows_affected())
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn update_page_read_at(
        &self,
        user_id: i64,
        chapter_id: i64,
        page: i64,
        is_complete: bool,
    ) -> Result<u64> {
        let mut conn = self.pool.acquire().await?;
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
        .execute(&mut conn)
        .await
        .map(|res| res.rows_affected())
        .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn update_chapters_read_at(&self, user_id: i64, chapter_ids: &[i64]) -> Result<u64> {
        let mut conn = self.pool.acquire().await?;
        if chapter_ids.is_empty() {
            return Ok(0);
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

        query
            .execute(&mut conn)
            .await
            .map(|res| res.rows_affected())
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn delete_chapters_read_at(&self, user_id: i64, chapter_ids: &[i64]) -> Result<u64> {
        let mut conn = self.pool.acquire().await?;
        if chapter_ids.is_empty() {
            return Ok(0);
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

        query
            .execute(&mut conn)
            .await
            .map(|res| res.rows_affected())
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn update_chapter_downloaded_path(
        &self,
        id: i64,
        path: Option<String>,
    ) -> Result<()> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query(r#"UPDATE chapter SET downloaded_path = ? WHERE id = ?"#)
            .bind(path)
            .bind(id)
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_user_history_last_read(
        &self,
        user_id: i64,
        chapter_id: i64,
    ) -> Result<Option<i64>> {
        let mut conn = self.pool.acquire().await?;
        let stream = sqlx::query(
            r#"SELECT last_page FROM user_history WHERE user_id = ? AND chapter_id = ?"#,
        )
        .bind(user_id)
        .bind(chapter_id)
        .fetch_one(&mut conn)
        .await
        .ok();

        if let Some(row) = stream {
            Ok(Some(row.get::<i64, _>(0)))
        } else {
            Ok(None)
        }
    }

    #[allow(dead_code)]
    pub async fn get_user_history_progress(
        &self,
        user_id: i64,
        chapter_id: i64,
    ) -> Result<Option<ReadProgress>> {
        let mut conn = self.pool.acquire().await?;
        let progress= sqlx::query(r#"SELECT read_at, last_page, is_complete FROM user_history WHERE user_id = ? AND chapter_id = ?"#)
                .bind(user_id)
                .bind(chapter_id)
                .fetch_optional(&mut conn)
                .await?
                .map(|row| ReadProgress {
                    at: row.get::<chrono::NaiveDateTime, _>(0),
                    last_page: row.get::<i64, _>(1),
                    is_complete: row.get::<bool, _>(2),
                });

        Ok(progress)
    }

    pub async fn get_user_history_progress_by_chapter_ids(
        &self,
        user_id: i64,
        chapter_ids: &[i64],
    ) -> Result<HashMap<i64, ReadProgress>> {
        let mut conn = self.pool.acquire().await?;

        let mut values = vec![];
        values.resize(chapter_ids.len(), "?");

        let query_str = format!(
            r#"SELECT chapter_id, read_at, last_page, is_complete FROM user_history WHERE user_id = ? AND chapter_id IN ({})"#,
            values.join(",")
        );

        let mut query = sqlx::query(&query_str).bind(user_id);
        for chapter_id in chapter_ids {
            query = query.bind(chapter_id)
        }

        let progress = query
            .fetch_all(&mut conn)
            .await?
            .iter()
            .map(|row| {
                (
                    row.get(0),
                    ReadProgress {
                        at: row.get::<chrono::NaiveDateTime, _>(1),
                        last_page: row.get::<i64, _>(2),
                        is_complete: row.get::<bool, _>(3),
                    },
                )
            })
            .collect();

        Ok(progress)
    }

    pub async fn get_user_library_unread_chapters(
        &self,
        user_id: i64,
        manga_ids: &[i64],
    ) -> Result<HashMap<i64, i64>> {
        let mut conn = self.pool.acquire().await?;

        let mut values = vec![];
        values.resize(manga_ids.len(), "?");

        let query_str = format!(
            r#"SELECT manga_id, COUNT(1) FROM (
                SELECT manga_id, IFNULL(user_history.is_complete, false) AS is_complete FROM chapter c LEFT JOIN user_history ON user_history.user_id = ? AND user_history.chapter_id = c.id WHERE c.manga_id IN ({})
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
            .fetch_all(&mut conn)
            .await?
            .iter()
            .map(|row| (row.get(0), row.get(1)))
            .collect();

        Ok(data)
    }

    pub async fn insert_download_queue(&self, items: &[DownloadQueue]) -> Result<()> {
        let mut conn = self.pool.acquire().await?;
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

        query.execute(&mut conn).await?;

        Ok(())
    }

    pub async fn get_single_download_queue(&self) -> Result<Option<DownloadQueue>> {
        let mut conn = self.pool.acquire().await?;
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
        .fetch_optional(&mut conn)
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

    pub async fn get_single_chapter_download_status(&self, chapter_id: i64) -> Result<bool> {
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(
            r#"SELECT SUM(downloaded) = COUNT(1)
                FROM download_queue
                WHERE chapter_id = ?"#,
        )
        .bind(chapter_id)
        .fetch_one(&mut conn)
        .await?;
        Ok(row.get(0))
    }

    pub async fn mark_single_download_queue_as_completed(&self, id: i64) -> Result<()> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query(r#"UPDATE download_queue SET downloaded = true WHERE id = ?"#)
            .bind(id)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn get_download_queue_last_priority(&self) -> Result<Option<i64>> {
        let mut conn = self.pool.acquire().await?;
        let data = sqlx::query(r#"SELECT MAX(priority) FROM download_queue"#)
            .fetch_optional(&mut conn)
            .await?
            .and_then(|row| row.try_get(0).ok());
        Ok(data)
    }

    pub async fn get_download_queue(&self) -> Result<Vec<DownloadQueueEntry>> {
        let mut conn = self.pool.acquire().await?;
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
        .fetch_all(&mut conn)
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

    pub async fn delete_single_chapter_download_queue(&self, chapter_id: i64) -> Result<()> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query(r#"DELETE FROM download_queue WHERE chapter_id = ?"#)
            .bind(chapter_id)
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn delete_download_queue_by_chapter_id(&self, id: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("UPDATE download_queue SET priority = priority - 1 WHERE priority > (SELECT priority FROM download_queue WHERE chapter_id = ? LIMIT 1)").bind(id).execute(&mut tx).await?;
        sqlx::query("DELETE FROM download_queue WHERE chapter_id = ?")
            .bind(id)
            .execute(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn update_download_queue_priority(
        &self,
        chapter_id: i64,
        priority: i64,
    ) -> Result<()> {
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

    pub async fn get_first_downloaded_chapters(
        &self,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        first: i32,
    ) -> Result<Vec<Chapter>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
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
        .fetch(&mut conn);

        let mut chapters = vec![];
        while let Some(row) = stream.try_next().await? {
            chapters.push(Chapter {
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
            });
        }
        Ok(chapters)
    }

    pub async fn get_last_downloaded_chapters(
        &self,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        last: i32,
    ) -> Result<Vec<Chapter>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
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
        .fetch(&mut conn);

        let mut chapters = vec![];
        while let Some(row) = stream.try_next().await? {
            chapters.push(Chapter {
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
            });
        }
        Ok(chapters)
    }

    pub async fn get_downloaded_chapters(
        &self,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
    ) -> Result<Vec<Chapter>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
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
        .fetch(&mut conn);

        let mut chapters = vec![];
        while let Some(row) = stream.try_next().await? {
            chapters.push(Chapter {
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
            });
        }
        Ok(chapters)
    }

    pub async fn get_downloaded_chapter_has_next_page(&self, timestamp: i64, id: i64) -> bool {
        let mut conn = if let Ok(conn) = self.pool.acquire().await {
            conn
        } else {
            return false;
        };
        let stream = sqlx::query(
            r#"
            SELECT * FROM chapter
            WHERE
                (date_added, id) < (datetime(?, 'unixepoch'), ?)
            GROUP BY id HAVING downloaded_path IS NOT NULL
            ORDER BY date_added DESC, id DESC
            LIMIT 1"#,
        )
        .bind(timestamp)
        .bind(id)
        .fetch_one(&mut conn)
        .await
        .ok();

        stream.is_some()
    }

    pub async fn get_downloaded_chapter_has_before_page(&self, timestamp: i64, id: i64) -> bool {
        let mut conn = if let Ok(conn) = self.pool.acquire().await {
            conn
        } else {
            return false;
        };
        let stream = sqlx::query(
            r#"
            SELECT * FROM chapter
            WHERE
                (date_added, id) < (datetime(?, 'unixepoch'), ?)
            GROUP BY id HAVING downloaded_path IS NOT NULL
            ORDER BY date_added DESC, id DESC
            LIMIT 1"#,
        )
        .bind(timestamp)
        .bind(id)
        .fetch_one(&mut conn)
        .await
        .ok();

        stream.is_some()
    }
    pub async fn insert_user_category(&self, user_id: i64, name: &str) -> Result<Category> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query("INSERT INTO user_category (user_id, name) VALUES (?, ?) RETURNING id, name")
            .bind(user_id)
            .bind(name)
            .fetch_one(&mut conn)
            .await
            .map(|row| Category {
                id: row.get(0),
                name: row.get(1),
            })
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn get_user_categories(&self, user_id: i64) -> Result<Vec<Category>> {
        let mut conn = self.pool.acquire().await?;
        let data = sqlx::query(
            r#"SELECT
                id,
                name
            FROM user_category
            WHERE user_id = ?
            ORDER BY name"#,
        )
        .bind(user_id)
        .fetch_all(&mut conn)
        .await?
        .iter()
        .map(|row| Category {
            id: row.get(0),
            name: row.get(1),
        })
        .collect();

        Ok(data)
    }

    pub async fn get_user_category(&self, id: i64) -> Result<Category> {
        let mut conn = self.pool.acquire().await?;
        let row = sqlx::query(
            r#"SELECT
                    id,
                    name
                FROM user_category
                WHERE id = ?"#,
        )
        .bind(id)
        .fetch_one(&mut conn)
        .await?;

        Ok(Category {
            id: row.get(0),
            name: row.get(1),
        })
    }

    pub async fn update_user_category(&self, id: i64, name: &str) -> Result<Category> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query("UPDATE user_category SET name = ? WHERE id = ? RETURNING id, name")
            .bind(name)
            .bind(id)
            .fetch_one(&mut conn)
            .await
            .map(|row| Category {
                id: row.get(0),
                name: row.get(1),
            })
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn delete_user_category(&self, id: i64) -> Result<u64> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query("DELETE FROM user_category WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await
            .map(|res| res.rows_affected())
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn insert_tracker_manga(
        &self,
        user_id: i64,
        manga_id: i64,
        tracker: &str,
        tracker_manga_id: String,
    ) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let row_id = sqlx::query(
            r#"
            INSERT INTO tracker_manga(
                user_id,
                manga_id,
                tracker,
                tracker_manga_id
            ) VALUES (?, ?, ?, ?)"#,
        )
        .bind(user_id)
        .bind(manga_id)
        .bind(tracker)
        .bind(tracker_manga_id)
        .execute(&mut conn)
        .await?
        .last_insert_rowid();

        Ok(row_id)
    }

    pub async fn delete_tracker_manga(
        &self,
        user_id: i64,
        manga_id: i64,
        tracker: &str,
    ) -> Result<u64> {
        let mut conn = self.pool.acquire().await?;
        let rows = sqlx::query(
            r#"
            DELETE FROM tracker_manga
            WHERE user_id = ? AND manga_id = ? AND tracker = ?
            "#,
        )
        .bind(user_id)
        .bind(manga_id)
        .bind(tracker)
        .execute(&mut conn)
        .await?
        .rows_affected();

        Ok(rows)
    }

    pub async fn get_tracker_manga_ids(
        &self,
        user_id: i64,
        manga_ids: &[i64],
    ) -> Result<Vec<TrackedManga>> {
        let mut conn = self.pool.acquire().await?;

        let query_str = format!(
            r#"
            SELECT m.id as manga_id, tc.tracker, tm.tracker_manga_id FROM tracker_credential tc 
            LEFT JOIN manga m ON m.id IN ({})
            LEFT JOIN tracker_manga tm ON tc.tracker = tm.tracker AND tm.manga_id = m.id
            WHERE tc.user_id = ?;
            "#,
            vec!["?"; manga_ids.len()].join(",")
        );

        let mut query = sqlx::query(&query_str);

        for manga_id in manga_ids {
            query = query.bind(manga_id);
        }

        let rows = query
            .bind(user_id)
            .fetch_all(&mut conn)
            .await?
            .iter()
            .map(|row| TrackedManga {
                manga_id: row.get(0),
                tracker: row.get(1),
                tracker_manga_id: row.get(2),
            })
            .collect();

        Ok(rows)
    }

    pub async fn get_tracker_manga_id(
        &self,
        user_id: i64,
        manga_id: i64,
    ) -> Result<Vec<TrackedManga>> {
        let mut conn = self.pool.acquire().await?;

        let query = sqlx::query(
            r#"
        SELECT m.id as manga_id, tc.tracker, tm.tracker_manga_id FROM tracker_credential tc 
        LEFT JOIN manga m ON m.id = ?
        LEFT JOIN tracker_manga tm ON tc.tracker = tm.tracker AND tm.manga_id = m.id
        WHERE tc.user_id = ?;
        "#,
        );

        let rows = query
            .bind(manga_id)
            .bind(user_id)
            .fetch_all(&mut conn)
            .await?
            .iter()
            .map(|row| TrackedManga {
                manga_id: row.get(0),
                tracker: row.get(1),
                tracker_manga_id: row.get(2),
            })
            .collect();

        Ok(rows)
    }
}
