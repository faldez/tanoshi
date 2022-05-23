use async_graphql::{Context, InputObject, Object, Result, SimpleObject};
use chrono::NaiveDateTime;

use crate::domain::services::tracker::TrackerService;
use crate::infrastructure::auth::Claims;
use crate::infrastructure::repositories::tracker::TrackerRepositoryImpl;
use tanoshi_tracker::{anilist, myanimelist};

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

        let session = ctx
            .data::<TrackerService<TrackerRepositoryImpl>>()?
            .login_start(myanimelist::NAME)?;

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
        let claim = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<TrackerService<TrackerRepositoryImpl>>()?
            .login_end(
                claim.sub,
                myanimelist::NAME,
                code,
                Some(state),
                Some(csrf_state),
                Some(pkce_code_verifier),
            )
            .await?;

        Ok("Success".to_string())
    }

    async fn anilist_login_start(&self, ctx: &Context<'_>) -> Result<Session> {
        let _ = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let session = ctx
            .data::<TrackerService<TrackerRepositoryImpl>>()?
            .login_start(anilist::NAME)?;

        Ok(Session {
            authorize_url: session.authorize_url,
            csrf_state: session.csrf_state.secret().to_owned(),
            pkce_code_verifier: session
                .pkce_code_verifier
                .map(|val| val.secret().to_owned()),
        })
    }

    async fn anilist_login_end(&self, ctx: &Context<'_>, code: String) -> Result<String> {
        let claim = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<TrackerService<TrackerRepositoryImpl>>()?
            .login_end(claim.sub, anilist::NAME, code, None, None, None)
            .await?;

        Ok("Success".to_string())
    }

    async fn search_tracker_manga(
        &self,
        ctx: &Context<'_>,
        tracker: String,
        title: String,
    ) -> Result<Vec<TrackerManga>> {
        let claim = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let manga = ctx
            .data::<TrackerService<TrackerRepositoryImpl>>()?
            .search_manga(claim.sub, &tracker, &title)
            .await?
            .into_iter()
            .map(|m| m.into())
            .collect();

        Ok(manga)
    }

    async fn manga_tracker_status(
        &self,
        ctx: &Context<'_>,
        manga_id: i64,
    ) -> Result<Vec<TrackerStatus>> {
        let claim = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let status = ctx
            .data::<TrackerService<TrackerRepositoryImpl>>()?
            .fetch_manga_tracking_status(claim.sub, manga_id)
            .await?
            .into_iter()
            .map(|m| m.into())
            .collect();

        Ok(status)
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
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<TrackerService<TrackerRepositoryImpl>>()?
            .track_manga(claims.sub, manga_id, &tracker, &tracker_manga_id)
            .await?;

        Ok(1)
    }

    async fn untrack_manga(
        &self,
        ctx: &Context<'_>,
        tracker: String,
        manga_id: i64,
    ) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<TrackerService<TrackerRepositoryImpl>>()?
            .untrack_manga(claims.sub, manga_id, &tracker)
            .await?;

        Ok(1)
    }

    async fn update_tracker_status(
        &self,
        ctx: &Context<'_>,
        tracker: String,
        tracker_manga_id: String,
        status: TrackerStatusInput,
    ) -> Result<bool> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<TrackerService<TrackerRepositoryImpl>>()?
            .update_manga_tracking_status(
                claims.sub,
                &tracker,
                tracker_manga_id,
                status.status,
                status.score,
                status.num_chapters_read,
                status.start_date,
                status.finish_date,
            )
            .await?;

        Ok(true)
    }

    async fn tracker_logout(&self, ctx: &Context<'_>, tracker: String) -> Result<u64> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        ctx.data::<TrackerService<TrackerRepositoryImpl>>()?
            .logout_tracker(claims.sub, &tracker)
            .await?;

        Ok(1)
    }
}
