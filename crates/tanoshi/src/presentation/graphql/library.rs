use super::{
    common::Cursor,
    manga::Manga,
    recent::{RecentChapter, RecentUpdate},
};
use crate::{
    application::worker::updates::{
        ChapterUpdateCommand, ChapterUpdateCommandSender, ChapterUpdateReceiver,
    },
    domain::services::{
        chapter::ChapterService, history::HistoryService, library::LibraryService,
        tracker::TrackerService,
    },
    infrastructure::{
        auth::Claims,
        domain::repositories::{
            chapter::ChapterRepositoryImpl, history::HistoryRepositoryImpl,
            library::LibraryRepositoryImpl, tracker::TrackerRepositoryImpl,
        },
    },
};
use async_graphql::{
    connection::{query, Connection, Edge, EmptyFields},
    Error, Subscription,
};
use async_graphql::{Context, Object, Result};
use chrono::Utc;

use flume::TrySendError;
use futures::{Stream, StreamExt};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Default)]
pub struct LibraryRoot;

#[Object(cache_control(max_age = 30, private))]
impl LibraryRoot {
    async fn library(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "category id")] category_id: Option<i64>,
    ) -> Result<Vec<Manga>> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let manga = ctx
            .data::<LibraryService<LibraryRepositoryImpl>>()?
            .get_manga_from_library_by_category_id(claims.sub, category_id)
            .await?
            .into_par_iter()
            .map(|m| m.into())
            .collect();

        Ok(manga)
    }

    async fn recent_updates(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<Cursor, RecentUpdate, EmptyFields, EmptyFields>> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let library_svc = ctx.data::<LibraryService<LibraryRepositoryImpl>>()?;

        query(
            after,
            before,
            first,
            last,
            |after: Option<Cursor>, before: Option<Cursor>, first, last| async move {
                let after_cursor = after.unwrap_or_else(|| Cursor(Utc::now().timestamp(), 1));
                let before_cursor = before.unwrap_or(Cursor(0, 0));

                let edges = library_svc
                    .get_library_recent_updates(
                        claims.sub,
                        after_cursor.0,
                        after_cursor.1,
                        before_cursor.0,
                        before_cursor.1,
                        first,
                        last,
                    )
                    .await?;

                let mut has_previous_page = false;
                if let Some(e) = edges.first() {
                    has_previous_page = !library_svc
                        .get_library_recent_updates(
                            claims.sub,
                            Utc::now().timestamp(),
                            1,
                            e.uploaded.timestamp(),
                            e.chapter_id,
                            None,
                            Some(1),
                        )
                        .await?
                        .is_empty();
                }

                let mut has_next_page = false;
                if let Some(e) = edges.last() {
                    has_next_page = !library_svc
                        .get_library_recent_updates(
                            claims.sub,
                            e.uploaded.timestamp(),
                            e.chapter_id,
                            0,
                            0,
                            Some(1),
                            None,
                        )
                        .await?
                        .is_empty();
                }

                let mut connection = Connection::new(has_previous_page, has_next_page);
                connection.edges.extend(
                    edges
                        .into_iter()
                        .map(|e| Edge::new(Cursor(e.uploaded.timestamp(), e.chapter_id), e.into())),
                );

                Ok::<_, Error>(connection)
            },
        )
        .await
    }

    async fn recent_chapters(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<Cursor, RecentChapter, EmptyFields, EmptyFields>> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let history_svc =
            ctx.data::<HistoryService<ChapterRepositoryImpl, HistoryRepositoryImpl>>()?;

        query(
            after,
            before,
            first,
            last,
            |after: Option<Cursor>, before: Option<Cursor>, first, last| async move {
                let after_cursor = after.unwrap_or_else(|| Cursor(Utc::now().timestamp(), 1));
                let before_cursor = before.unwrap_or(Cursor(0, 0));

                let edges = history_svc
                    .get_history_chapters(claims.sub, after_cursor.0, before_cursor.0, first, last)
                    .await?;

                let mut has_previous_page = false;
                if let Some(e) = edges.first() {
                    has_previous_page = !history_svc
                        .get_history_chapters(
                            claims.sub,
                            Utc::now().timestamp(),
                            e.read_at.timestamp(),
                            None,
                            Some(1),
                        )
                        .await?
                        .len()
                        > 0;
                }

                let mut has_next_page = false;
                if let Some(e) = edges.last() {
                    has_next_page = !history_svc
                        .get_history_chapters(claims.sub, e.read_at.timestamp(), 0, Some(1), None)
                        .await?
                        .is_empty();
                }

                let mut connection = Connection::new(has_previous_page, has_next_page);
                connection.edges.extend(
                    edges
                        .into_iter()
                        .map(|e| Edge::new(Cursor(e.read_at.timestamp(), e.manga_id), e.into())),
                );

                Ok::<_, Error>(connection)
            },
        )
        .await
    }
}

#[derive(Default)]
pub struct LibraryMutationRoot;

#[Object]
impl LibraryMutationRoot {
    async fn add_to_library(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "manga id")] manga_id: i64,
        #[graphql(desc = "category ids")] category_ids: Vec<i64>,
    ) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<LibraryService<LibraryRepositoryImpl>>()?
            .insert_manga_to_library(claims.sub, manga_id, category_ids)
            .await?;

        Ok(1)
    }

    async fn delete_from_library(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "manga id")] manga_id: i64,
    ) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<LibraryService<LibraryRepositoryImpl>>()?
            .delete_manga_from_library(claims.sub, manga_id)
            .await?;

        Ok(1)
    }

    async fn update_page_read_at(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "chapter id")] chapter_id: i64,
        #[graphql(desc = "page")] page: i64,
        #[graphql(desc = "is_complete")] is_complete: bool,
    ) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<HistoryService<ChapterRepositoryImpl, HistoryRepositoryImpl>>()?
            .insert_chapter_to_history(claims.sub, chapter_id, page, is_complete)
            .await?;

        let chapter = ctx
            .data::<ChapterService<ChapterRepositoryImpl>>()?
            .fetch_chapter_by_id(chapter_id)
            .await?;

        if is_complete {
            let tracker_svc = ctx.data::<TrackerService<TrackerRepositoryImpl>>()?;

            let tracked_manga = tracker_svc
                .get_tracked_manga_id(claims.sub, chapter.manga_id)
                .await?;

            for manga in tracked_manga {
                if let Some(tracker_manga_id) = manga.tracker_manga_id {
                    // TODO: Only update if chapter > then read
                    tracker_svc
                        .update_manga_tracking_status(
                            claims.sub,
                            &manga.tracker,
                            tracker_manga_id,
                            None,
                            None,
                            Some(chapter.number as i64),
                            None,
                            None,
                        )
                        .await?;
                }
            }
        }

        Ok(1)
    }

    async fn mark_chapter_as_read(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "chapter ids")] chapter_ids: Vec<i64>,
    ) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<HistoryService<ChapterRepositoryImpl, HistoryRepositoryImpl>>()?
            .insert_chapters_to_history_as_completed(claims.sub, chapter_ids)
            .await?;

        Ok(1)
    }

    async fn mark_chapter_as_unread(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "chapter ids")] chapter_ids: Vec<i64>,
    ) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<HistoryService<ChapterRepositoryImpl, HistoryRepositoryImpl>>()?
            .delete_chapters_from_history(claims.sub, chapter_ids)
            .await?;

        Ok(1)
    }

    async fn refresh_chapters(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "manga id")] manga_id: Option<i64>,
        #[graphql(desc = "wait for updates", default = false)] wait: bool,
    ) -> Result<bool> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let (tx, rx) = tokio::sync::oneshot::channel();
        let command = if let Some(manga_id) = manga_id {
            ChapterUpdateCommand::Manga(manga_id, tx)
        } else {
            ChapterUpdateCommand::Library(claims.sub, tx)
        };

        if let Err(e) = ctx.data::<ChapterUpdateCommandSender>()?.try_send(command) {
            match e {
                TrySendError::Full(_) => {
                    return Err("chapter updates is ongoing, try again later".into());
                }
                TrySendError::Disconnected(_) => {
                    return Err("chapter updates thread is closed".into());
                }
            }
        }

        if wait {
            rx.await??;
        }

        Ok(true)
    }
}

#[derive(Default)]
pub struct LibrarySubscriptionRoot;

#[Subscription]
impl LibrarySubscriptionRoot {
    async fn recent_updates_subscription(
        &self,
        ctx: &Context<'_>,
    ) -> Result<impl Stream<Item = RecentUpdate>> {
        let user_id = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?
            .sub;

        let receiver = ctx.data::<ChapterUpdateReceiver>()?.resubscribe();

        let stream = tokio_stream::wrappers::BroadcastStream::new(receiver);

        let stream = stream.filter_map(move |res| async move {
            debug!("update: {res:?}");
            if let Ok(update) = res {
                if update.users.get(&user_id).is_some() {
                    Some(RecentUpdate {
                        manga_id: update.chapter.manga_id,
                        chapter_id: update.chapter.id,
                        manga_title: update.manga.title,
                        cover_url: update.manga.cover_url,
                        chapter_title: update.chapter.title,
                        uploaded: update.chapter.uploaded,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        });

        Ok(stream)
    }
}
