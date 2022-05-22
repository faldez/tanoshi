use async_graphql::{Context, InputObject, Object, Result, SimpleObject};
use chrono::NaiveDateTime;

use crate::db::{model, MangaDatabase, UserDatabase};
use crate::infrastructure::auth::Claims;
use tanoshi_tracker::{anilist, myanimelist, AniList, MyAnimeList, Tracker};

#[derive(SimpleObject)]
pub struct Session {
    pub authorize_url: String,
    pub csrf_state: String,
    pub pkce_code_verifier: Option<String>,
}

#[derive(Debug, Default, SimpleObject)]
pub struct TrackerStatus {
    pub tracker: String,
    pub tracker_manga_id: Option<String>,
    pub tracker_manga_title: Option<String>,
    pub status: Option<String>,
    pub score: Option<i64>,
    pub num_chapters_read: Option<i64>,
    pub start_date: Option<NaiveDateTime>,
    pub finish_date: Option<NaiveDateTime>,
}

impl From<tanoshi_tracker::TrackerStatus> for TrackerStatus {
    fn from(other: tanoshi_tracker::TrackerStatus) -> Self {
        Self {
            tracker: other.tracker,
            tracker_manga_id: other.tracker_manga_id,
            tracker_manga_title: other.tracker_manga_title,
            status: other.status,
            score: other.score,
            num_chapters_read: other.num_chapters_read,
            start_date: other.start_date,
            finish_date: other.finish_date,
        }
    }
}

#[derive(Debug, Default, InputObject)]
pub struct TrackerStatusInput {
    pub status: Option<String>,
    pub score: Option<i64>,
    pub num_chapters_read: Option<i64>,
    pub start_date: Option<NaiveDateTime>,
    pub finish_date: Option<NaiveDateTime>,
}

#[derive(Default, SimpleObject)]
pub struct TrackerManga {
    pub tracker: String,
    pub tracker_manga_id: String,
    pub title: String,
    pub synopsis: String,
    pub cover_url: String,
    pub status: String,
}

impl From<tanoshi_tracker::TrackerManga> for TrackerManga {
    fn from(other: tanoshi_tracker::TrackerManga) -> Self {
        Self {
            tracker: other.tracker,
            tracker_manga_id: other.tracker_manga_id,
            title: other.title,
            synopsis: other.synopsis,
            cover_url: other.cover_url,
            status: other.status,
        }
    }
}

#[derive(Default)]
pub struct TrackingRoot;

#[Object]
impl TrackingRoot {
    async fn myanimelist_login_start(&self, ctx: &Context<'_>) -> Result<Session> {
        let _ = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        let session = ctx.data::<MyAnimeList>()?.get_authorize_url().unwrap();
        Ok(Session {
            authorize_url: session.authorize_url,
            csrf_state: session.csrf_state.secret().to_owned(),
            pkce_code_verifier: session
                .pkce_code_verifier
                .map(|val| val.secret().to_owned()),
        })
    }

    async fn myanimelist_login_end(
        &self,
        ctx: &Context<'_>,
        code: String,
        state: String,
        csrf_state: String,
        pkce_code_verifier: String,
    ) -> Result<String> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        let client = ctx.data::<MyAnimeList>()?;
        let token = client
            .exchange_code(
                code,
                Some(state),
                Some(csrf_state),
                Some(pkce_code_verifier),
            )
            .await
            .map(|token| model::Token {
                token_type: token.token_type,
                access_token: token.access_token,
                refresh_token: token.refresh_token,
                expires_in: token.expires_in,
            })?;
        ctx.data::<UserDatabase>()?
            .insert_tracker_credential(user.sub, myanimelist::NAME, token)
            .await?;
        Ok("Success".to_string())
    }

    async fn anilist_login_start(&self, ctx: &Context<'_>) -> Result<Session> {
        let _ = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        let session = ctx.data::<AniList>()?.get_authorize_url().unwrap();
        Ok(Session {
            authorize_url: session.authorize_url,
            csrf_state: session.csrf_state.secret().to_owned(),
            pkce_code_verifier: session
                .pkce_code_verifier
                .map(|val| val.secret().to_owned()),
        })
    }

    async fn anilist_login_end(&self, ctx: &Context<'_>, code: String) -> Result<String> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        let client = ctx.data::<AniList>()?;
        let token = client
            .exchange_code(code, None, None, None)
            .await
            .map(|token| model::Token {
                token_type: token.token_type,
                access_token: token.access_token,
                refresh_token: token.refresh_token,
                expires_in: token.expires_in,
            })?;
        ctx.data::<UserDatabase>()?
            .insert_tracker_credential(user.sub, anilist::NAME, token)
            .await?;
        Ok("Success".to_string())
    }

    async fn search_tracker_manga(
        &self,
        ctx: &Context<'_>,
        tracker: String,
        title: String,
    ) -> Result<Vec<TrackerManga>> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let tracker_token = ctx
            .data::<UserDatabase>()?
            .get_user_tracker_token(&tracker, user.sub)
            .await?;

        let client: &dyn Tracker = match tracker.as_str() {
            myanimelist::NAME => ctx.data::<MyAnimeList>()?,
            anilist::NAME => ctx.data::<AniList>()?,
            _ => return Err("tracker not available".into()),
        };

        match client.search_manga(tracker_token.access_token, title).await {
            Ok(manga_list) => Ok(manga_list.into_iter().map(|m| m.into()).collect()),
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
                        .insert_tracker_credential(user.sub, &tracker, token)
                        .await?;
                }
                Err(e.into())
            }
        }
    }

    async fn manga_tracker_status(
        &self,
        ctx: &Context<'_>,
        manga_id: i64,
    ) -> Result<Vec<TrackerStatus>> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        let trackers = ctx
            .data::<MangaDatabase>()?
            .get_tracker_manga_id(user.sub, manga_id)
            .await?;

        let mut data: Vec<TrackerStatus> = vec![];
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

            let mut status: Option<TrackerStatus> = None;
            if let Some(tracker_manga_id) = tracker.tracker_manga_id.to_owned() {
                status = match client
                    .get_manga_details(tracker_token.access_token, tracker_manga_id.parse()?)
                    .await
                {
                    Ok(res) => res.tracker_status.map(|status| status.into()),
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
                }
            }

            data.push(status.unwrap_or_else(|| TrackerStatus {
                tracker: tracker.tracker,
                ..Default::default()
            }));
        }

        Ok(data)
    }
}

#[derive(Default)]
pub struct TrackingMutationRoot;

#[Object]
impl TrackingMutationRoot {
    async fn track_manga(
        &self,
        ctx: &Context<'_>,
        tracker: String,
        manga_id: i64,
        tracker_manga_id: String,
    ) -> Result<i64> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        if !matches!(tracker.as_str(), myanimelist::NAME | anilist::NAME) {
            return Err("tracker not available".into());
        }

        Ok(ctx
            .data::<MangaDatabase>()?
            .insert_tracker_manga(user.sub, manga_id, &tracker, tracker_manga_id)
            .await?)
    }

    async fn untrack_manga(
        &self,
        ctx: &Context<'_>,
        tracker: String,
        manga_id: i64,
    ) -> Result<u64> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        Ok(ctx
            .data::<MangaDatabase>()?
            .delete_tracker_manga(user.sub, manga_id, &tracker)
            .await?)
    }

    async fn update_tracker_status(
        &self,
        ctx: &Context<'_>,
        tracker: String,
        tracker_manga_id: String,
        status: TrackerStatusInput,
    ) -> Result<bool> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let tracker_token = ctx
            .data::<UserDatabase>()?
            .get_user_tracker_token(&tracker, user.sub)
            .await?;

        let tracker_manga_id: i64 = tracker_manga_id.parse()?;

        let client: &dyn Tracker = match tracker.as_str() {
            myanimelist::NAME => ctx.data::<MyAnimeList>()?,
            anilist::NAME => ctx.data::<AniList>()?,
            _ => return Err("tracker not available".into()),
        };

        match client
            .update_tracker_status(
                tracker_token.access_token,
                tracker_manga_id,
                status.status,
                status.score,
                status.num_chapters_read,
                status.start_date,
                status.finish_date,
            )
            .await
        {
            Ok(_) => Ok(true),
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
                        .insert_tracker_credential(user.sub, &tracker, token)
                        .await?;
                }

                Err(e.into())
            }
        }
    }
}
