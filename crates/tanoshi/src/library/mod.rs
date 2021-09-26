use crate::catalogue::Manga;
use crate::db::MangaDatabase;
use crate::user;
use async_graphql::connection::{query, Connection, Edge, EmptyFields};
use async_graphql::{Context, Object, Result};
use chrono::{Local, NaiveDateTime};

mod recent;
pub use recent::{RecentChapter, RecentUpdate};
use tanoshi_vm::prelude::ExtensionBus;

#[derive(Default)]
pub struct LibraryRoot;

#[Object]
impl LibraryRoot {
    async fn library(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "refresh data from source", default = false)] refresh: bool,
    ) -> Result<Vec<Manga>> {
        let user = user::get_claims(ctx)?;
        let db = ctx.data::<MangaDatabase>()?;
        let manga = db.get_library(user.sub).await?;

        if refresh {
            let extensions = ctx.data::<ExtensionBus>()?;
            for favorite_manga in manga.iter() {
                let mut m: crate::db::model::Manga = {
                    extensions
                        .get_manga_info(favorite_manga.source_id, favorite_manga.path.clone())
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
        let user = user::get_claims(ctx)?;
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
                connection.append(
                    edges
                        .into_iter()
                        .map(|e| Edge::new(encode_cursor(e.uploaded.timestamp(), e.chapter_id), e)),
                );
                Ok(connection)
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
        let user = user::get_claims(ctx)?;
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
                    .unwrap_or((NaiveDateTime::from_timestamp(0, 0).timestamp(), 0));

                let edges = if let Some(first) = first {
                    db.get_first_read_chapters(
                        user.sub,
                        after_timestamp,
                        after_id,
                        before_timestamp,
                        before_id,
                        first as i32,
                    )
                    .await
                } else if let Some(last) = last {
                    db.get_last_read_chapters(
                        user.sub,
                        after_timestamp,
                        after_id,
                        before_timestamp,
                        before_id,
                        last as i32,
                    )
                    .await
                } else {
                    db.get_read_chapters(after_timestamp, after_id, before_timestamp, before_id)
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
                connection.append(
                    edges
                        .into_iter()
                        .map(|e| Edge::new(encode_cursor(e.read_at.timestamp(), e.manga_id), e)),
                );
                Ok(connection)
            },
        )
        .await
    }
}

fn decode_cursor(cursor: &str) -> std::result::Result<(i64, i64), base64::DecodeError> {
    match base64::decode(cursor) {
        Ok(res) => {
            let cursor = String::from_utf8(res).unwrap();
            let decoded = cursor
                .split('#')
                .map(|s| s.parse::<i64>().unwrap())
                .collect::<Vec<i64>>();
            Ok((decoded[0], decoded[1]))
        }
        Err(err) => Err(err),
    }
}

fn encode_cursor(timestamp: i64, id: i64) -> String {
    base64::encode(format!("{}#{}", timestamp, id))
}

#[derive(Default)]
pub struct LibraryMutationRoot;

#[Object]
impl LibraryMutationRoot {
    async fn add_to_library(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "manga id")] manga_id: i64,
    ) -> Result<u64> {
        let user = user::get_claims(ctx)?;
        match ctx
            .data::<MangaDatabase>()?
            .insert_user_library(user.sub, manga_id)
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
        let user = user::get_claims(ctx)?;
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
    ) -> Result<u64> {
        let user = user::get_claims(ctx)?;
        match ctx
            .data::<MangaDatabase>()?
            .update_page_read_at(user.sub, chapter_id, page)
            .await
        {
            Ok(rows) => Ok(rows),
            Err(err) => Err(format!("error update page read_at: {}", err).into()),
        }
    }

    async fn mark_chapter_as_read(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "chapter ids")] chapter_ids: Vec<i64>,
    ) -> Result<u64> {
        let user = user::get_claims(ctx)?;
        match ctx
            .data::<MangaDatabase>()?
            .update_chapters_read_at(user.sub, &chapter_ids)
            .await
        {
            Ok(rows) => Ok(rows),
            Err(err) => Err(format!("error update chapter read_at: {}", err).into()),
        }
    }

    async fn mark_chapter_as_unread(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "chapter ids")] chapter_ids: Vec<i64>,
    ) -> Result<u64> {
        let user = user::get_claims(ctx)?;
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
