use super::{catalogue::Chapter, guard::AdminGuard};
use crate::{
    domain::services::download::DownloadService,
    infrastructure::{
        config::GLOBAL_CONFIG,
        domain::repositories::download::DownloadRepositoryImpl,
        utils::{decode_cursor, encode_cursor},
    },
};
use async_graphql::{
    connection::{query, Connection, Edge, EmptyFields},
    Context, Error, Object, Result, SimpleObject,
};
use chrono::Local;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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

impl From<crate::domain::entities::download::DownloadQueueEntry> for DownloadQueueEntry {
    fn from(queue: crate::domain::entities::download::DownloadQueueEntry) -> Self {
        Self {
            source_id: queue.source_id,
            source_name: queue.source_name,
            manga_id: queue.manga_id,
            manga_title: queue.manga_title,
            chapter_id: queue.chapter_id,
            chapter_title: queue.chapter_title,
            downloaded: queue.downloaded,
            total: queue.total,
            priority: queue.priority,
        }
    }
}

#[derive(Default)]
pub struct DownloadRoot;

#[Object]
impl DownloadRoot {
    async fn download_status(&self, ctx: &Context<'_>) -> Result<bool> {
        let download_path = GLOBAL_CONFIG
            .get()
            .map(|cfg| &cfg.download_path)
            .ok_or("config not initialized")?;

        let status = ctx
            .data::<DownloadService<DownloadRepositoryImpl>>()?
            .get_download_status(download_path);

        Ok(status)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn download_queue(&self, ctx: &Context<'_>) -> Result<Vec<DownloadQueueEntry>> {
        let queue = ctx
            .data::<DownloadService<DownloadRepositoryImpl>>()?
            .get_download_queue()
            .await?
            .into_par_iter()
            .map(|q| q.into())
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
        let download_svc = ctx.data::<DownloadService<DownloadRepositoryImpl>>()?;
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

                let edges = download_svc
                    .get_downloaded_chapters(
                        after_timestamp,
                        after_id,
                        before_timestamp,
                        before_id,
                        first,
                        last,
                    )
                    .await
                    .unwrap_or_default();

                let mut has_previous_page = false;
                if let Some(e) = edges.first() {
                    has_previous_page = download_svc
                        .get_downloaded_chapters(
                            Local::now().naive_local().timestamp(),
                            1,
                            e.date_added.timestamp(),
                            e.id,
                            None,
                            Some(1),
                        )
                        .await?
                        .len()
                        > 0;
                }

                let mut has_next_page = false;
                if let Some(e) = edges.last() {
                    has_next_page = download_svc
                        .get_downloaded_chapters(
                            e.date_added.timestamp(),
                            e.id,
                            0,
                            0,
                            Some(1),
                            None,
                        )
                        .await?
                        .len()
                        > 0;
                }

                let mut connection = Connection::new(has_previous_page, has_next_page);
                connection.append(edges.into_iter().map(|e| {
                    Edge::new(
                        encode_cursor(e.uploaded.timestamp(), e.id),
                        Chapter {
                            id: e.id,
                            source_id: e.source_id,
                            manga_id: e.manga_id,
                            title: e.title,
                            path: e.path,
                            number: e.number,
                            scanlator: e.scanlator,
                            uploaded: e.uploaded,
                            date_added: e.date_added,
                            read_progress: None,
                            downloaded_path: e.downloaded_path,
                        },
                    )
                }));

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
    async fn pause_download(&self, ctx: &Context<'_>) -> Result<bool> {
        let download_path = GLOBAL_CONFIG
            .get()
            .map(|cfg| &cfg.download_path)
            .ok_or("config not initialized")?;

        ctx.data::<DownloadService<DownloadRepositoryImpl>>()?
            .change_download_status(download_path, false)
            .await?;

        Ok(true)
    }

    async fn resume_download(&self, ctx: &Context<'_>) -> Result<bool> {
        let download_path = GLOBAL_CONFIG
            .get()
            .map(|cfg| &cfg.download_path)
            .ok_or("config not initialized")?;

        ctx.data::<DownloadService<DownloadRepositoryImpl>>()?
            .change_download_status(download_path, true)
            .await?;

        Ok(true)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn download_chapters(&self, ctx: &Context<'_>, ids: Vec<i64>) -> Result<i64> {
        let len = ids.len() as i64;
        ctx.data::<DownloadService<DownloadRepositoryImpl>>()?
            .download_chapters(ids)
            .await?;

        Ok(len)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn remove_chapters_from_queue(&self, ctx: &Context<'_>, ids: Vec<i64>) -> Result<i64> {
        let len = ids.len() as i64;
        ctx.data::<DownloadService<DownloadRepositoryImpl>>()?
            .remove_chapters_from_queue(ids)
            .await?;

        Ok(len)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn remove_downloaded_chapters(&self, ctx: &Context<'_>, ids: Vec<i64>) -> Result<i64> {
        let len = ids.len() as i64;
        ctx.data::<DownloadService<DownloadRepositoryImpl>>()?
            .remove_downloaded_chapters(ids)
            .await?;

        Ok(len)
    }

    #[graphql(guard = "AdminGuard::new()")]
    async fn update_chapter_priority(
        &self,
        ctx: &Context<'_>,
        id: i64,
        priority: i64,
    ) -> Result<bool> {
        ctx.data::<DownloadService<DownloadRepositoryImpl>>()?
            .update_chapter_priority(id, priority)
            .await?;

        Ok(true)
    }
}
