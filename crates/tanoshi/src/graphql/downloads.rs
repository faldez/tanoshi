use std::path::PathBuf;

use super::{catalogue::Chapter, guard::AdminGuard};
use crate::{
    config::GLOBAL_CONFIG,
    db::{model, MangaDatabase},
    utils::{decode_cursor, encode_cursor},
    worker::downloads::{Command as DownloadCommand, DownloadSender},
};
use async_graphql::{
    connection::{query, Connection, Edge, EmptyFields},
    Context, Error, Object, Result, SimpleObject,
};
use chrono::Local;

#[derive(Debug, SimpleObject)]
pub struct DownloadQueueEntry {
    pub source_id: i64,
    pub source_name: String,
    pub manga_id: i64,
    pub manga_title: String,
    pub chapter_id: i64,
    pub chapter_title: String,
    pub downloaded: i64,
    pub total: i64,
    pub priority: i64,
}

#[derive(Default)]
pub struct DownloadRoot;

#[Object]
impl DownloadRoot {
    async fn download_status(&self) -> Result<bool> {
        let pause_path = GLOBAL_CONFIG
            .get()
            .map(|cfg| PathBuf::new().join(&cfg.download_path).join(".pause"))
            .ok_or("config not initialized")?;

        Ok(!pause_path.exists())
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn download_queue(&self, ctx: &Context<'_>) -> Result<Vec<DownloadQueueEntry>> {
        let db = ctx.data::<MangaDatabase>()?;
        let queue: Vec<model::DownloadQueueEntry> = db.get_download_queue().await?;
        let queue = queue
            .iter()
            .map(|queue| DownloadQueueEntry {
                source_id: queue.source_id,
                source_name: queue.source_name.clone(),
                manga_id: queue.manga_id,
                manga_title: queue.manga_title.clone(),
                chapter_id: queue.chapter_id,
                chapter_title: queue.chapter_title.clone(),
                downloaded: queue.downloaded,
                total: queue.total,
                priority: queue.priority,
            })
            .collect();

        Ok(queue)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn get_downloaded_chapters(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<String, Chapter, EmptyFields, EmptyFields>> {
        let db = ctx.data::<MangaDatabase>()?;
        query(
            after,
            before,
            first,
            last,
            |after, before, first, last| async move {
                let (after_timestamp, after_id) = after
                    .and_then(|cursor: String| decode_cursor(&cursor).ok())
                    .unwrap_or((Local::now().naive_local().timestamp(), 1));
                let (before_timestamp, before_id) = before
                    .and_then(|cursor: String| decode_cursor(&cursor).ok())
                    .unwrap_or((0, 0));

                let edges = if let Some(first) = first {
                    db.get_first_downloaded_chapters(
                        after_timestamp,
                        after_id,
                        before_timestamp,
                        before_id,
                        first as i32,
                    )
                    .await
                } else if let Some(last) = last {
                    db.get_last_downloaded_chapters(
                        after_timestamp,
                        after_id,
                        before_timestamp,
                        before_id,
                        last as i32,
                    )
                    .await
                } else {
                    db.get_downloaded_chapters(
                        after_timestamp,
                        after_id,
                        before_timestamp,
                        before_id,
                    )
                    .await
                };
                let edges: Vec<Chapter> = edges
                    .unwrap_or_default()
                    .into_iter()
                    .map(|ch| ch.into())
                    .collect();

                let mut has_previous_page = false;
                let mut has_next_page = false;
                if !edges.is_empty() {
                    if let Some(e) = edges.first() {
                        has_previous_page = db
                            .get_downloaded_chapter_has_before_page(e.date_added.timestamp(), e.id)
                            .await;
                    }
                    if let Some(e) = edges.last() {
                        has_next_page = db
                            .get_downloaded_chapter_has_next_page(e.date_added.timestamp(), e.id)
                            .await;
                    }
                }

                let mut connection = Connection::new(has_previous_page, has_next_page);
                connection.append(
                    edges
                        .into_iter()
                        .map(|e| Edge::new(encode_cursor(e.uploaded.timestamp(), e.id), e)),
                );

                Ok::<_, Error>(connection)
            },
        )
        .await
    }
}
#[derive(Default)]
pub struct DownloadMutationRoot;

#[Object]
impl DownloadMutationRoot {
    async fn pause_download(&self) -> Result<bool> {
        let pause_path = GLOBAL_CONFIG
            .get()
            .map(|cfg| PathBuf::new().join(&cfg.download_path).join(".pause"))
            .ok_or("config not initialized")?;
        tokio::fs::write(pause_path, b"").await?;

        Ok(true)
    }

    async fn resume_download(&self, ctx: &Context<'_>) -> Result<bool> {
        let pause_path = GLOBAL_CONFIG
            .get()
            .map(|cfg| PathBuf::new().join(&cfg.download_path).join(".pause"))
            .ok_or("config not initialized")?;
        tokio::fs::remove_file(pause_path).await?;

        ctx.data::<DownloadSender>()?
            .send(DownloadCommand::Download)?;

        Ok(true)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn download_chapters(&self, ctx: &Context<'_>, ids: Vec<i64>) -> Result<i64> {
        let mut len = 0_usize;
        for id in ids {
            ctx.data::<DownloadSender>()?
                .send(DownloadCommand::InsertIntoQueue(id))?;

            len += 1;
        }

        ctx.data::<DownloadSender>()?
            .send(DownloadCommand::Download)?;

        Ok(len as _)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn remove_chapters_from_queue(&self, ctx: &Context<'_>, ids: Vec<i64>) -> Result<i64> {
        let db = ctx.data::<MangaDatabase>()?;

        let mut len = 0;
        for id in ids {
            db.delete_download_queue_by_chapter_id(id).await?;

            len += 1;
        }

        Ok(len)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn remove_downloaded_chapters(&self, ctx: &Context<'_>, ids: Vec<i64>) -> Result<i64> {
        let db = ctx.data::<MangaDatabase>()?;

        let mut len = 0_usize;
        for id in ids {
            let chapter = db.get_chapter_by_id(id).await?;
            if let Some(downloaded_path) = chapter.downloaded_path {
                if let Err(e) = tokio::fs::remove_file(&downloaded_path).await {
                    error!("error removing file: {e}");
                }
            }
            db.update_chapter_downloaded_path(id, None).await?;
            len += 1;
        }

        Ok(len as _)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn update_chapter_priority(
        &self,
        ctx: &Context<'_>,
        id: i64,
        priority: i64,
    ) -> Result<bool> {
        ctx.data::<MangaDatabase>()?
            .update_download_queue_priority(id, priority)
            .await?;

        Ok(true)
    }
}
