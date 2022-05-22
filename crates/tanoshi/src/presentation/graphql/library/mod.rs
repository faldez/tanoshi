use super::catalogue::Manga;
use crate::{
    db::{model, MangaDatabase, UserDatabase},
    infrastructure::{
        auth::Claims,
        utils::{decode_cursor, encode_cursor},
    },
};
use async_graphql::{
    connection::{query, Connection, Edge, EmptyFields},
    Error,
};
use async_graphql::{Context, Object, Result};
use chrono::{Local, NaiveDateTime};
use tanoshi_tracker::{anilist, myanimelist, AniList, MyAnimeList, Tracker};

mod categories;
pub use categories::{Category, CategoryMutationRoot, CategoryRoot};

mod recent;
pub use recent::{RecentChapter, RecentUpdate};

use tanoshi_vm::extension::SourceBus;

#[derive(Default)]
pub struct LibraryRoot;

#[Object]
impl LibraryRoot {
    async fn library(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "refresh data from source", default = false)] refresh: bool,
        #[graphql(desc = "category id")] category_id: Option<i64>,
    ) -> Result<Vec<Manga>> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        let db = ctx.data::<MangaDatabase>()?;
        let manga = db.get_library_by_category_id(user.sub, category_id).await?;

        if refresh {
            let extensions = ctx.data::<SourceBus>()?;
            for favorite_manga in manga.iter() {
                let mut m: model::Manga = {
                    extensions
                        .get_manga_detail(favorite_manga.source_id, favorite_manga.path.clone())
                        .await?
                        .into()
                };

                m.id = favorite_manga.id;
                db.insert_manga(&mut m).await?;
            }
        }

        Ok(manga.into_iter().map(|m| m.into()).collect())
    }

    async fn recent_updates(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<String, RecentUpdate, EmptyFields, EmptyFields>> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        let db = ctx.data::<MangaDatabase>()?;
        query(
            after,
            before,
            first,
            last,
            |after, before, first, last| async move {
                let (after_timestamp, after_id) = after
                    .and_then(|cursor: String| decode_cursor(&cursor).ok())
                    .unwrap_or((Local::now().timestamp(), 1));
                let (before_timestamp, before_id) = before
                    .and_then(|cursor: String| decode_cursor(&cursor).ok())
                    .unwrap_or((0, 0));

                let edges = if let Some(first) = first {
                    db.get_first_recent_updates(
                        user.sub,
                        after_timestamp,
                        after_id,
                        before_timestamp,
                        before_id,
                        first as i32,
                    )
                    .await
                } else if let Some(last) = last {
                    db.get_last_recent_updates(
                        user.sub,
                        after_timestamp,
                        after_id,
                        before_timestamp,
                        before_id,
                        last as i32,
                    )
                    .await
                } else {
                    db.get_recent_updates(
                        user.sub,
                        after_timestamp,
                        after_id,
                        before_timestamp,
                        before_id,
                    )
                    .await
                };
                let edges = edges.unwrap_or_default();

                let mut has_previous_page = false;
                let mut has_next_page = false;
                if !edges.is_empty() {
                    if let Some(e) = edges.first() {
                        has_previous_page = db
                            .get_chapter_has_before_page(
                                user.sub,
                                e.uploaded.timestamp(),
                                e.chapter_id,
                            )
                            .await;
                    }
                    if let Some(e) = edges.last() {
                        has_next_page = db
                            .get_chapter_has_next_page(
                                user.sub,
                                e.uploaded.timestamp(),
                                e.chapter_id,
                            )
                            .await;
                    }
                }

                let mut connection = Connection::new(has_previous_page, has_next_page);
                connection.append(edges.into_iter().map(|e| {
                    Edge::new(
                        encode_cursor(e.uploaded.timestamp(), e.chapter_id),
                        e.into(),
                    )
                }));

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
    ) -> Result<Connection<String, RecentChapter, EmptyFields, EmptyFields>> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        let db = ctx.data::<MangaDatabase>()?;
        query(
            after,
            before,
            first,
            last,
            |after, before, first, last| async move {
                let (after_timestamp, _after_id) = after
                    .and_then(|cursor: String| decode_cursor(&cursor).ok())
                    .unwrap_or((Local::now().timestamp(), 1));
                let (before_timestamp, _before_id) = before
                    .and_then(|cursor: String| decode_cursor(&cursor).ok())
                    .unwrap_or((NaiveDateTime::from_timestamp(0, 0).timestamp(), 0));

                let edges = if let Some(first) = first {
                    db.get_first_read_chapters(
                        user.sub,
                        after_timestamp,
                        before_timestamp,
                        first as i32,
                    )
                    .await
                } else if let Some(last) = last {
                    db.get_last_read_chapters(
                        user.sub,
                        after_timestamp,
                        before_timestamp,
                        last as i32,
                    )
                    .await
                } else {
                    db.get_read_chapters(after_timestamp, before_timestamp)
                        .await
                };
                let edges = edges.unwrap_or_default();

                let mut has_previous_page = false;
                let mut has_next_page = false;
                if !edges.is_empty() {
                    if let Some(e) = edges.first() {
                        has_previous_page = db
                            .get_read_chapter_has_before_page(
                                user.sub,
                                e.read_at.timestamp(),
                                e.manga_id,
                            )
                            .await;
                    }
                    if let Some(e) = edges.last() {
                        has_next_page = db
                            .get_read_chapter_has_next_page(
                                user.sub,
                                e.read_at.timestamp(),
                                e.manga_id,
                            )
                            .await;
                    }
                }

                let mut connection = Connection::new(has_previous_page, has_next_page);
                connection.append(edges.into_iter().map(|e| {
                    Edge::new(encode_cursor(e.read_at.timestamp(), e.manga_id), e.into())
                }));

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
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        match ctx
            .data::<MangaDatabase>()?
            .insert_user_library(user.sub, manga_id, category_ids)
            .await
        {
            Ok(rows) => Ok(rows),
            Err(err) => Err(format!("error add manga to library: {}", err).into()),
        }
    }

    async fn delete_from_library(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "manga id")] manga_id: i64,
    ) -> Result<u64> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        match ctx
            .data::<MangaDatabase>()?
            .delete_user_library(user.sub, manga_id)
            .await
        {
            Ok(rows) => Ok(rows),
            Err(err) => Err(format!("error delete manga from library: {}", err).into()),
        }
    }

    async fn update_page_read_at(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "chapter id")] chapter_id: i64,
        #[graphql(desc = "page")] page: i64,
        #[graphql(desc = "is_complete")] is_complete: bool,
    ) -> Result<u64> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let mangadb = ctx.data::<MangaDatabase>()?;
        let rows = mangadb
            .update_page_read_at(user.sub, chapter_id, page, is_complete)
            .await?;

        let chapter = mangadb.get_chapter_by_id(chapter_id).await?;
        // TODO: nepnep source have weird number, don't update tracker status for them for now
        if !is_complete || (chapter.source_id == 3 || chapter.source_id == 4) {
            return Ok(rows);
        }

        let trackers = mangadb
            .get_tracker_manga_id(user.sub, chapter.manga_id)
            .await?;
        for tracker in trackers {
            let tracker_token = ctx
                .data::<UserDatabase>()?
                .get_user_tracker_token(&tracker.tracker, user.sub)
                .await?;

            let client: &dyn Tracker = match tracker.tracker.as_str() {
                myanimelist::NAME => ctx.data::<MyAnimeList>()?,
                anilist::NAME => ctx.data::<AniList>()?,
                _ => return Err("tracker not available".into()),
            };

            if let Some(tracker_manga_id) = tracker.tracker_manga_id.to_owned() {
                let tracker_manga_id = tracker_manga_id.parse()?;
                let tracker_data = match client
                    .get_manga_details(tracker_token.access_token.clone(), tracker_manga_id)
                    .await
                {
                    Ok(res) => res,
                    Err(e) => {
                        if matches!(e, tanoshi_tracker::Error::Unauthorized) {
                            let token = client
                                .refresh_token(tracker_token.refresh_token)
                                .await
                                .map(|token| model::Token {
                                    token_type: token.token_type,
                                    access_token: token.access_token,
                                    refresh_token: token.refresh_token,
                                    expires_in: token.expires_in,
                                })?;

                            ctx.data::<UserDatabase>()?
                                .insert_tracker_credential(user.sub, &tracker.tracker, token)
                                .await?;
                        }

                        return Err(e.into());
                    }
                };

                if let Some(num_chapters_read) = tracker_data
                    .tracker_status
                    .and_then(|status| status.num_chapters_read)
                {
                    if chapter.number <= num_chapters_read as f64 {
                        continue;
                    }
                }

                client
                    .update_tracker_status(
                        tracker_token.access_token,
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

        Ok(rows)
    }

    async fn mark_chapter_as_read(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "chapter ids")] chapter_ids: Vec<i64>,
    ) -> Result<u64> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        match ctx
            .data::<MangaDatabase>()?
            .update_chapters_read_at(user.sub, &chapter_ids)
            .await
        {
            Ok(rows) => Ok(rows),
            Err(err) => Err(format!("error mark_chapter_as_read: {}", err).into()),
        }
    }

    async fn mark_chapter_as_unread(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "chapter ids")] chapter_ids: Vec<i64>,
    ) -> Result<u64> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        match ctx
            .data::<MangaDatabase>()?
            .delete_chapters_read_at(user.sub, &chapter_ids)
            .await
        {
            Ok(rows) => Ok(rows),
            Err(err) => Err(format!("error delete chapter read_at: {}", err).into()),
        }
    }
}
