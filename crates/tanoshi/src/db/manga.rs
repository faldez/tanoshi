use super::model::{Chapter, Manga};
use crate::library::{RecentChapter, RecentUpdate};
use anyhow::{anyhow, Result};
use sqlx::sqlite::{SqliteArguments, SqlitePool};
use sqlx::{Arguments, Row};
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
        let stream = sqlx::query(r#"SELECT * FROM manga WHERE id = ?"#)
            .bind(id)
            .fetch_one(&self.pool)
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

    pub async fn get_manga_by_source_path(&self, source_id: i64, path: &str) -> Result<Manga> {
        let stream = sqlx::query(r#"SELECT * FROM manga WHERE source_id = ? AND path = ?"#)
            .bind(source_id)
            .bind(path)
            .fetch_one(&self.pool)
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

    pub async fn get_library(&self, user_id: i64) -> Result<Vec<Manga>> {
        let mut stream = sqlx::query(
            r#"SELECT manga.* FROM manga
                    JOIN user_library ON manga.id = user_library.manga_id AND user_library.user_id = ?
                    ORDER BY title"#,
        )
        .bind(user_id)
        .fetch(&self.pool);

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

    pub async fn get_all_user_library(&self) -> Result<Vec<(Option<i64>, Manga)>> {
        let mut stream = sqlx::query(
            r#"SELECT manga.*, user.telegram_chat_id FROM manga
            JOIN user_library ON user_library.manga_id = manga.id
            JOIN user ON user.id = user_library.user_id"#,
        )
        .fetch(&self.pool);

        let mut mangas = vec![];
        while let Some(row) = stream.try_next().await? {
            mangas.push((
                row.get(10),
                Manga {
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
            ));
        }
        Ok(mangas)
    }

    pub async fn is_user_library(&self, user_id: i64, manga_id: i64) -> Result<bool> {
        let stream =
            sqlx::query(r#"SELECT true FROM user_library WHERE user_id = ? AND manga_id = ?"#)
                .bind(user_id)
                .bind(manga_id)
                .fetch_one(&self.pool)
                .await
                .ok();

        if let Some(row) = stream {
            Ok(row.get(0))
        } else {
            Ok(false)
        }
    }

    pub async fn get_recent_updates(
        &self,
        user_id: i64,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
    ) -> Result<Vec<RecentUpdate>> {
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
        ORDER BY chapter.uploaded DESC, chapter.id DESC"#,
        )
        .bind(user_id)
        .bind(after_timestamp)
        .bind(after_id)
        .bind(before_timestamp)
        .bind(before_id)
        .fetch(&self.pool);

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
        ORDER BY chapter.uploaded DESC, chapter.id DESC
        LIMIT ?"#,
        )
        .bind(user_id)
        .bind(after_timestamp)
        .bind(after_id)
        .bind(before_timestamp)
        .bind(before_id)
        .bind(first)
        .fetch(&self.pool);

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
        let mut stream = sqlx::query(
            r#"
        SELECT * FROM (
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
            ORDER BY chapter.uploaded ASC, chapter.id ASC
            LIMIT ?) c
        ORDER BY c.uploaded DESC, c.id DESC"#,
        )
        .bind(user_id)
        .bind(after_timestamp)
        .bind(after_id)
        .bind(before_timestamp)
        .bind(before_id)
        .bind(last)
        .fetch(&self.pool);

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
        .fetch_one(&self.pool)
        .await
        .ok();

        let mut count = 0;
        if let Some(row) = stream {
            count = row.get(0);
        }
        count > 0
    }

    pub async fn get_chapter_has_before_page(&self, user_id: i64, timestamp: i64, id: i64) -> bool {
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
        .fetch_one(&self.pool)
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
        let stream = sqlx::query(
            r#"
            SELECT COUNT(id) 
            FROM chapter 
            JOIN manga ON manga.id = chapter.manga_id
            WHERE manga.is_favorite = true"#,
        )
        .fetch_one(&self.pool)
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
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
    ) -> Result<Vec<RecentChapter>> {
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
        JOIN chapter ON chapter.id = user_history.chapter_id
        JOIN manga ON manga.id = chapter.manga_id
        WHERE 
            user_history.user_id = ? AND
            manga.id NOT IN (?, ?) AND
            user_history.read_at < datetime(?, 'unixepoch') AND
            user_history.read_at > datetime(?, 'unixepoch')
        GROUP BY manga.id
        ORDER BY user_history.read_at DESC, manga.id DESC"#,
        )
        .bind(after_id)
        .bind(before_id)
        .bind(after_timestamp)
        .bind(before_timestamp)
        .fetch(&self.pool);

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
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        first: i32,
    ) -> Result<Vec<RecentChapter>> {
        log::info!(
            "{} {} {} {}",
            after_timestamp,
            after_id,
            before_timestamp,
            before_id
        );
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
        JOIN chapter ON chapter.id = user_history.chapter_id
        JOIN manga ON manga.id = chapter.manga_id
        WHERE 
            user_history.user_id = ? AND
            manga.id NOT IN (?, ?) AND
            user_history.read_at < datetime(?, 'unixepoch') AND
            user_history.read_at > datetime(?, 'unixepoch')
        GROUP BY manga.id
        ORDER BY user_history.read_at DESC, manga.id DESC
        LIMIT ?"#,
        )
        .bind(user_id)
        .bind(after_id)
        .bind(before_id)
        .bind(after_timestamp)
        .bind(before_timestamp)
        .bind(first)
        .fetch(&self.pool);

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
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        last: i32,
    ) -> Result<Vec<RecentChapter>> {
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
            JOIN chapter ON chapter.id = user_history.chapter_id
            JOIN manga ON manga.id = chapter.manga_id
            WHERE 
                user_history.user_id = ? AND
                manga.id NOT IN (?, ?) AND
                user_history.read_at < datetime(?, 'unixepoch') AND
                user_history.read_at > datetime(?, 'unixepoch')
            GROUP BY manga.id
            ORDER BY user_history.read_at ASC, manga.id ASC
            LIMIT ?) c ORDER BY c.read_at DESC, c.id DESC"#,
        )
        .bind(user_id)
        .bind(after_id)
        .bind(before_id)
        .bind(after_timestamp)
        .bind(before_timestamp)
        .bind(last)
        .fetch(&self.pool);

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
        let stream = sqlx::query(
            r#"
            SELECT COUNT(1) FROM (
				SELECT
                	user_history.last_page,
                	MAX(user_history.read_at) as read_at
            	FROM user_history
            	JOIN chapter ON user_history.chapter_id = chapter.id
            	WHERE 
                user_history.user_id = ? AND
                	chapter.manga_id <> ? AND
                	user_history.read_at < datetime(?, 'unixepoch')
            	GROUP BY chapter.manga_id
            	ORDER BY user_history.read_at DESC, chapter.manga_id DESC
            )"#,
        )
        .bind(user_id)
        .bind(id)
        .bind(timestamp)
        .fetch_one(&self.pool)
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
        let stream = sqlx::query(
            r#"
            SELECT COUNT(1) FROM (
				SELECT
                	user_history.last_page,
                	MAX(user_history.read_at) as read_at
            	FROM user_history
            	JOIN chapter ON user_history.chapter_id = chapter.id
            	WHERE 
                    user_history.user_id = ? AND
                	chapter.manga_id <> ? AND
                	user_history.read_at > datetime(?, 'unixepoch')
            	GROUP BY chapter.manga_id
            	ORDER BY user_history.read_at DESC, chapter.manga_id DESC
            )"#,
        )
        .bind(user_id)
        .bind(id)
        .bind(timestamp)
        .fetch_one(&self.pool)
        .await
        .ok();

        let mut count = 0;
        if let Some(row) = stream {
            count = row.get(0);
        }
        count > 0
    }

    pub async fn insert_manga(&self, manga: &mut Manga) -> Result<()> {
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
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        manga.id = row_id;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn insert_mangas(&self, _manga: Vec<Manga>) -> Result<()> {
        todo!()
    }

    #[allow(dead_code)]
    pub async fn update_manga_info(&self, manga: &Manga) -> Result<u64> {
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
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(rows_affected)
    }

    pub async fn get_chapter_by_id(&self, id: i64) -> Result<Chapter> {
        let stream = sqlx::query(
            r#"
            SELECT *, 
            (SELECT JSON_GROUP_ARRAY(remote_url) FROM page WHERE chapter_id = chapter.id) pages,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number < chapter.number ORDER BY c.number DESC LIMIT 1) prev,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number > chapter.number ORDER BY c.number ASC LIMIT 1) next 
            FROM chapter WHERE id = ?"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
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
            pages: serde_json::from_str(row.get(9)).unwrap_or_default(),
            prev: row.get(10),
            next: row.get(11),
            last_page_read: None,
        })?)
    }

    #[allow(dead_code)]
    pub async fn get_chapter_by_source_path(&self, source_id: i64, path: &str) -> Option<Chapter> {
        let stream = sqlx::query(
            r#"
            SELECT *,
            (SELECT JSON_GROUP_ARRAY(remote_url) FROM page WHERE chapter_id = chapter.id) pages,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number < chapter.number ORDER BY c.number DESC LIMIT 1) prev,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number > chapter.number ORDER BY c.number ASC LIMIT 1) next 
            FROM chapter WHERE source_id = ? AND path = ?"#,
        )
        .bind(source_id)
        .bind(path)
        .fetch_one(&self.pool)
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
            pages: serde_json::from_str(row.get(9)).unwrap_or_default(),
            prev: row.get(10),
            next: row.get(11),
            last_page_read: None,
        })
    }

    pub async fn get_chapters_by_manga_id(&self, manga_id: i64) -> Result<Vec<Chapter>> {
        let mut stream = sqlx::query(
            r#"
            SELECT *,
            (SELECT JSON_GROUP_ARRAY(remote_url) FROM page WHERE chapter_id = chapter.id) pages,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number < chapter.number ORDER BY c.number DESC LIMIT 1) prev,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number > chapter.number ORDER BY c.number ASC LIMIT 1) next 
            FROM chapter WHERE manga_id = ? ORDER BY number DESC"#
        )
        .bind(manga_id)
        .fetch(&self.pool);

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
                pages: serde_json::from_str(row.get(9)).unwrap_or_default(),
                prev: row.get(10),
                next: row.get(11),
                last_page_read: None,
            });
        }
        if chapters.is_empty() {
            Err(anyhow::anyhow!("Chapters not found"))
        } else {
            Ok(chapters)
        }
    }

    pub async fn get_last_uploaded_chapters_by_manga_id(&self, manga_id: i64) -> Option<Chapter> {
        let stream = sqlx::query(
            r#"
            SELECT *,
            (SELECT JSON_GROUP_ARRAY(remote_url) FROM page WHERE chapter_id = chapter.id) pages,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number < chapter.number ORDER BY c.number DESC LIMIT 1) prev,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.number > chapter.number ORDER BY c.number ASC LIMIT 1) next 
            FROM chapter WHERE manga_id = ? ORDER BY uploaded DESC LIMIT 1"#
        )
        .bind(manga_id)
        .fetch_one(&self.pool)
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
            pages: serde_json::from_str(row.get(9)).unwrap_or_default(),
            prev: row.get(10),
            next: row.get(11),
            last_page_read: None,
        })
    }

    #[allow(dead_code)]
    pub async fn insert_chapter(&self, chapter: &Chapter) -> Result<i64> {
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
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
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
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(row_id)
    }

    pub async fn insert_chapters(&self, chapters: &[Chapter]) -> Result<()> {
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

        query.execute(&self.pool).await?;

        Ok(())
    }

    pub async fn insert_pages(&self, chapter_id: i64, pages: &[String]) -> Result<()> {
        if pages.is_empty() {
            return Ok(());
        }

        let mut values = vec![];
        values.resize(pages.len(), "(?, ?, ?)");

        let query_str = format!(
            r#"INSERT INTO page (
                chapter_id, 
                rank, 
                remote_url
            ) VALUES {} ON CONFLICT(chapter_id, rank) DO UPDATE SET
                remote_url=excluded.remote_url
            "#,
            values.join(",")
        );

        let mut query = sqlx::query(&query_str);
        for (index, page) in pages.iter().enumerate() {
            query = query.bind(chapter_id).bind(index as i64).bind(page);
        }

        query.execute(&self.pool).await?;

        Ok(())
    }

    pub async fn insert_user_library(&self, user_id: i64, manga_id: i64) -> Result<u64> {
        sqlx::query("INSERT INTO user_library (user_id, manga_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(manga_id)
            .execute(&self.pool)
            .await
            .map(|res| res.rows_affected())
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn delete_user_library(&self, user_id: i64, manga_id: i64) -> Result<u64> {
        sqlx::query("DELETE FROM user_library WHERE user_id = ? AND manga_id = ?")
            .bind(user_id)
            .bind(manga_id)
            .execute(&self.pool)
            .await
            .map(|res| res.rows_affected())
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn update_page_read_at(
        &self,
        user_id: i64,
        chapter_id: i64,
        page: i64,
    ) -> Result<u64> {
        sqlx::query(
            r#"INSERT INTO 
            user_history(user_id, chapter_id, last_page, read_at) VALUES(?, ?, ?, ?)
            ON CONFLICT(user_id, chapter_id) 
            DO UPDATE SET 
            last_page = excluded.last_page, 
            read_at = excluded.read_at"#,
        )
        .bind(user_id)
        .bind(chapter_id)
        .bind(page)
        .bind(chrono::Local::now())
        .execute(&self.pool)
        .await
        .map(|res| res.rows_affected())
        .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn get_user_history_last_read(
        &self,
        user_id: i64,
        chapter_id: i64,
    ) -> Result<Option<i64>> {
        let stream = sqlx::query(
            r#"SELECT last_page FROM user_history WHERE user_id = ? AND chapter_id = ?"#,
        )
        .bind(user_id)
        .bind(chapter_id)
        .fetch_one(&self.pool)
        .await
        .ok();

        if let Some(row) = stream {
            Ok(Some(row.get::<i64, _>(0)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_user_history_read_at(
        &self,
        user_id: i64,
        chapter_id: i64,
    ) -> Result<Option<chrono::NaiveDateTime>> {
        let stream =
            sqlx::query(r#"SELECT read_at FROM user_history WHERE user_id = ? AND chapter_id = ?"#)
                .bind(user_id)
                .bind(chapter_id)
                .fetch_one(&self.pool)
                .await
                .ok();

        if let Some(row) = stream {
            Ok(Some(row.get::<chrono::NaiveDateTime, _>(0)))
        } else {
            Ok(None)
        }
    }
}
