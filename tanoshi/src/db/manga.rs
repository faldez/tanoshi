use crate::catalogue::{Chapter, Manga, Page};
use crate::library::{RecentChapter, RecentUpdate};
use anyhow::{anyhow, Result};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub fn new(pool: SqlitePool) -> Db {
        Db { pool }
    }

    pub async fn get_manga_by_id(&self, id: i64) -> Option<Manga> {
        let stream = sqlx::query(r#"SELECT * FROM manga WHERE id = ?"#)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .ok();

        if let Some(row) = stream {
            Some(Manga {
                id: row.get(0),
                source_id: row.get(1),
                title: row.get(2),
                author: serde_json::from_str(row.get::<String, _>(3).as_str()).unwrap_or(vec![]),
                genre: serde_json::from_str(row.get::<String, _>(4).as_str()).unwrap_or(vec![]),
                status: row.get(5),
                description: row.get(6),
                path: row.get(7),
                cover_url: row.get(8),
                is_favorite: row.get(9),
                last_read_chapter: row.get(10),
                date_added: row.get(11),
            })
        } else {
            None
        }
    }

    pub async fn get_manga_by_source_path(&self, source_id: i64, path: &String) -> Option<Manga> {
        let stream = sqlx::query(r#"SELECT * FROM manga WHERE source_id = ? AND path = ?"#)
            .bind(source_id)
            .bind(path)
            .fetch_one(&self.pool)
            .await
            .ok();

        if let Some(row) = stream {
            Some(Manga {
                id: row.get(0),
                source_id: row.get(1),
                title: row.get(2),
                author: serde_json::from_str(row.get::<String, _>(3).as_str()).unwrap_or(vec![]),
                genre: serde_json::from_str(row.get::<String, _>(4).as_str()).unwrap_or(vec![]),
                status: row.get(5),
                description: row.get(6),
                path: row.get(7),
                cover_url: row.get(8),
                is_favorite: row.get(9),
                last_read_chapter: row.get(10),
                date_added: row.get(11),
            })
        } else {
            None
        }
    }

    pub async fn get_library(&self) -> Result<Vec<Manga>> {
        let mut stream =
            sqlx::query(r#"SELECT * FROM manga WHERE is_favorite = true"#).fetch(&self.pool);

        let mut mangas = vec![];
        while let Some(row) = stream.try_next().await? {
            mangas.push(Manga {
                id: row.get(0),
                source_id: row.get(1),
                title: row.get(2),
                author: serde_json::from_str(row.get::<String, _>(3).as_str()).unwrap_or(vec![]),
                genre: serde_json::from_str(row.get::<String, _>(4).as_str()).unwrap_or(vec![]),
                status: row.get(5),
                description: row.get(6),
                path: row.get(7),
                cover_url: row.get(8),
                last_read_chapter: row.get(9),
                is_favorite: row.get(10),
                date_added: row.get(11),
            });
        }
        Ok(mangas)
    }

    pub async fn get_recent_updates(
        &self,
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
        WHERE 
        manga.is_favorite = true AND
        (uploaded, chapter.id) < (datetime(?, 'unixepoch'), ?) AND
        (uploaded, chapter.id) > (datetime(?, 'unixepoch'), ?)
        ORDER BY chapter.uploaded DESC, chapter.id DESC"#,
        )
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
        WHERE 
        manga.is_favorite = true AND
        (uploaded, chapter.id) < (datetime(?, 'unixepoch'), ?) AND
        (uploaded, chapter.id) > (datetime(?, 'unixepoch'), ?)
        ORDER BY chapter.uploaded DESC, chapter.id DESC
        LIMIT ?"#,
        )
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
            WHERE 
                manga.is_favorite = true AND
                (uploaded, chapter.id) < (datetime(?, 'unixepoch'), ?) AND
                (uploaded, chapter.id) > (datetime(?, 'unixepoch'), ?)
            ORDER BY chapter.uploaded ASC, chapter.id ASC
            LIMIT ?) c
        ORDER BY c.uploaded DESC, c.id DESC"#,
        )
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

    pub async fn get_chapter_has_next_page(&self, timestamp: i64, id: i64) -> bool {
        let stream = sqlx::query(
            r#"
            SELECT
                chapter.id as chapter_id,
                chapter.uploaded
            FROM chapter
            JOIN manga ON manga.id = chapter.manga_id
            WHERE 
                manga.is_favorite = true AND
                (uploaded, chapter.id) < (datetime(?, 'unixepoch'), ?)
            ORDER BY chapter.uploaded DESC, chapter.id DESC"#,
        )
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

    pub async fn get_chapter_has_before_page(&self, timestamp: i64, id: i64) -> bool {
        let stream = sqlx::query(
            r#"
        SELECT
            chapter.id as chapter_id,
            chapter.uploaded
        FROM chapter
        JOIN manga ON manga.id = chapter.manga_id
        WHERE 
            manga.is_favorite = true AND
            (uploaded, chapter.id) > (datetime(?, 'unixepoch'), ?)
        ORDER BY chapter.uploaded DESC, chapter.id DESC"#,
        )
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
            MAX(page.read_at) as read_at,
            page.id
        FROM page
        JOIN chapter ON chapter.id = page.chapter_id
        JOIN manga ON manga.id = page.manga_id
        WHERE
            page.read_at IS NOT NULL AND
            page.manga_id NOT IN (?, ?) AND
            page.read_at < datetime(?, 'unixepoch') AND
            page.read_at > datetime(?, 'unixepoch')
        GROUP BY page.manga_id
        ORDER BY page.read_at DESC, manga.id DESC"#,
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
            MAX(page.read_at) as read_at,
            page.id
        FROM page
        JOIN chapter ON chapter.id = page.chapter_id
        JOIN manga ON manga.id = page.manga_id
        WHERE 
            page.read_at IS NOT NULL AND
            page.manga_id NOT IN (?, ?) AND
            page.read_at < datetime(?, 'unixepoch') AND
            page.read_at > datetime(?, 'unixepoch')
        GROUP BY page.manga_id
        ORDER BY page.read_at DESC, manga.id DESC
        LIMIT ?"#,
        )
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
                MAX(page.read_at) as read_at,
                page.id
            FROM page
            JOIN chapter ON chapter.id = page.chapter_id
            JOIN manga ON manga.id = page.manga_id
            WHERE 
                page.read_at IS NOT NULL AND
                page.manga_id NOT IN (?, ?) AND
                page.read_at < datetime(?, 'unixepoch') AND
                page.read_at > datetime(?, 'unixepoch')
            GROUP BY page.manga_id
            ORDER BY page.read_at ASC, manga.id ASC
            LIMIT ?) c
        ORDER BY c.read_at DESC, c.id DESC"#,
        )
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

    pub async fn get_read_chapter_has_next_page(&self, timestamp: i64, id: i64) -> bool {
        let stream = sqlx::query(
            r#"
            SELECT COUNT(1) FROM (
				SELECT
                	page.id,
                	MAX(page.read_at) as read_at
            	FROM page
            	WHERE 
                	page.read_at IS NOT NULL AND
                	page.manga_id <> ? AND
                	page.read_at < datetime(?, 'unixepoch')
            	GROUP BY page.manga_id
            	ORDER BY page.read_at DESC, manga.id DESC
            )"#,
        )
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

    pub async fn get_read_chapter_has_before_page(&self, timestamp: i64, id: i64) -> bool {
        let stream = sqlx::query(
            r#"
            SELECT COUNT(1) FROM (
				SELECT
                	page.id,
                	MAX(page.read_at) as read_at
            	FROM page
            	WHERE 
                	page.read_at IS NOT NULL AND
                	page.manga_id <> ? AND
                	page.read_at > datetime(?, 'unixepoch')
            	GROUP BY page.manga_id
            	ORDER BY page.read_at DESC, manga.id DESC
            );"#,
        )
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

    pub async fn insert_manga(&self, manga: &Manga) -> Result<i64> {
        let row_id = sqlx::query(
            r#"INSERT INTO manga(
                source_id, 
                title, 
                author, 
                genre, 
                status, 
                description, 
                path, 
                cover_url, 
                date_added
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(manga.source_id)
        .bind(&manga.title)
        .bind(serde_json::to_string(&manga.author).unwrap_or("[]".to_string()))
        .bind(serde_json::to_string(&manga.genre).unwrap_or("[]".to_string()))
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

        Ok(row_id)
    }

    pub async fn insert_mangas(&self, manga: Vec<Manga>) -> Result<()> {
        todo!()
    }

    pub async fn update_manga_info(&self, manga: &Manga) -> Result<u64> {
        let mut column_to_update = vec![];
        if manga.source_id > 0 {
            column_to_update.push("source_id = ?");
        }
        if manga.title != "" {
            column_to_update.push("title = ?");
        }
        if manga.author.len() > 0 {
            column_to_update.push("author = ?");
        }
        if manga.genre.len() > 0 {
            column_to_update.push("genre = ?");
        }
        if manga.status.is_some() {
            column_to_update.push("status = ?");
        }
        if manga.description.is_some() {
            column_to_update.push("description = ?");
        }
        if manga.path != "" {
            column_to_update.push("path = ?");
        }
        if manga.cover_url != "" {
            column_to_update.push("cover_url = ?");
        }

        if column_to_update.len() == 0 {
            return Err(anyhow!("Nothing to update"));
        }

        let query = format!(
            r#"UPDATE manga SET
                {}
                WHERE id = ?"#,
            column_to_update.join(",")
        );

        let rows_affected = sqlx::query(&query)
            .bind(manga.source_id)
            .bind(&manga.title)
            .bind(serde_json::to_string(&manga.author).unwrap_or("[]".to_string()))
            .bind(serde_json::to_string(&manga.genre).unwrap_or("[]".to_string()))
            .bind(&manga.status)
            .bind(&manga.description)
            .bind(&manga.path)
            .bind(&manga.cover_url)
            .bind(manga.id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(rows_affected)
    }

    pub async fn get_chapter_by_id(&self, id: i64) -> Option<Chapter> {
        let stream = sqlx::query(
            r#"
            SELECT *, 
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.rank = chapter.rank - 1) prev,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.rank = chapter.rank + 1) next 
            FROM chapter WHERE id = ?"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .ok();

        if let Some(row) = stream {
            Some(Chapter {
                id: row.get(0),
                source_id: row.get(1),
                manga_id: row.get(2),
                title: row.get(3),
                path: row.get(4),
                rank: row.get(5),
                read_at: row.get(6),
                uploaded: row.get(7),
                date_added: row.get(8),
                prev: row.get(9),
                next: row.get(10),
            })
        } else {
            None
        }
    }

    pub async fn get_chapter_by_source_path(
        &self,
        source_id: i64,
        path: &String,
    ) -> Option<Chapter> {
        let stream = sqlx::query(
            r#"
            SELECT *,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.rank = chapter.rank - 1) prev,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.rank = chapter.rank + 1) next 
            FROM chapter WHERE source_id = ? AND path = ?"#,
        )
        .bind(source_id)
        .bind(path)
        .fetch_one(&self.pool)
        .await
        .ok();

        if let Some(row) = stream {
            Some(Chapter {
                id: row.get(0),
                source_id: row.get(1),
                manga_id: row.get(2),
                title: row.get(3),
                path: row.get(4),
                rank: row.get(5),
                read_at: row.get(6),
                uploaded: row.get(7),
                date_added: row.get(8),
                prev: row.get(9),
                next: row.get(10),
            })
        } else {
            None
        }
    }

    pub async fn get_chapters_by_manga_id(&self, manga_id: i64) -> Result<Vec<Chapter>> {
        let mut stream = sqlx::query(
            r#"
            SELECT *,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.rank = chapter.rank - 1) prev,
            (SELECT c.id FROM chapter c WHERE c.manga_id = chapter.manga_id AND c.rank = chapter.rank + 1) next 
            FROM chapter WHERE manga_id = ? ORDER BY rank DESC"#
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
                rank: row.get(5),
                read_at: row.get(6),
                uploaded: row.get(7),
                date_added: row.get(8),
                prev: row.get(9),
                next: row.get(10),
            });
        }
        if chapters.len() == 0 {
            Err(anyhow::anyhow!("Chapters not found"))
        } else {
            Ok(chapters)
        }
    }

    pub async fn insert_chapter(&self, chapter: &Chapter) -> Result<i64> {
        let row_id = sqlx::query(
            r#"INSERT INTO chapter(
                source_id,
                manga_id,
                title, 
                path, 
                rank, 
                uploaded, 
                date_added
            ) VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(chapter.source_id)
        .bind(chapter.manga_id)
        .bind(&chapter.title)
        .bind(&chapter.path)
        .bind(chapter.rank)
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

    pub async fn get_page_by_source_url(&self, source_id: i64, url: &String) -> Option<Page> {
        let stream = sqlx::query(r#"SELECT * FROM page WHERE source_id = ? AND url = ?"#)
            .bind(source_id)
            .bind(url)
            .fetch_one(&self.pool)
            .await
            .ok();

        if let Some(row) = stream {
            Some(Page {
                id: row.get(0),
                source_id: row.get(1),
                manga_id: row.get(2),
                chapter_id: row.get(3),
                rank: row.get(4),
                url: row.get(5),
                read_at: row.get(6),
                date_added: row.get(7),
            })
        } else {
            None
        }
    }

    pub async fn insert_page(&self, page: &Page) -> Result<i64> {
        let row_id = sqlx::query(
            r#"INSERT INTO page(
                source_id,
                manga_id,
                chapter_id,
                rank, 
                url,
                date_added
            ) VALUES (?, ?, ?, ?, ?, ?)"#,
        )
        .bind(page.source_id)
        .bind(page.manga_id)
        .bind(page.chapter_id)
        .bind(page.rank)
        .bind(&page.url)
        .bind(chrono::NaiveDateTime::from_timestamp(
            chrono::Local::now().timestamp(),
            0,
        ))
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(row_id)
    }

    pub async fn get_pages_by_chapter_id(&self, chapter_id: i64) -> Result<Vec<Page>> {
        let mut stream = sqlx::query(r#"SELECT * FROM page WHERE chapter_id = ?"#)
            .bind(chapter_id)
            .fetch(&self.pool);

        let mut pages = vec![];
        while let Some(row) = stream.try_next().await? {
            pages.push(Page {
                id: row.get(0),
                source_id: row.get(1),
                manga_id: row.get(2),
                chapter_id: row.get(3),
                rank: row.get(4),
                url: row.get(5),
                read_at: row.get(6),
                date_added: row.get(7),
            });
        }
        if pages.len() == 0 {
            Err(anyhow::anyhow!("Pages not found"))
        } else {
            Ok(pages)
        }
    }

    pub async fn favorite_manga(&self, manga_id: i64, is_favorite: bool) -> Result<u64> {
        sqlx::query("UPDATE manga SET is_favorite = ? WHERE id = ?")
            .bind(is_favorite)
            .bind(manga_id)
            .execute(&self.pool)
            .await
            .map(|res| res.rows_affected())
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn update_page_read_at(&self, page_id: i64) -> Result<u64> {
        let now = chrono::Local::now();
        let mut tx = self.pool.begin().await.map_err(|e| anyhow::anyhow!(e))?;
        sqlx::query("UPDATE page SET read_at = ? WHERE id = ?")
            .bind(now)
            .bind(page_id)
            .execute(&mut tx)
            .await
            .map(|res| res.rows_affected())
            .map_err(|e| anyhow::anyhow!(e))?;

        sqlx::query(
            "UPDATE chapter SET read_at = ? WHERE id = (SELECT chapter_id FROM page WHERE id = ?)",
        )
        .bind(now)
        .bind(page_id)
        .execute(&mut tx)
        .await
        .map(|res| res.rows_affected())
        .map_err(|e| anyhow::anyhow!(e))?;

        tx.commit().await.map(|_| 1).map_err(|e| anyhow::anyhow!(e))
    }
}