use async_graphql::{Context, InputObject, Object, Result, SimpleObject};
use chrono::{Datelike, NaiveDateTime, NaiveTime};

use crate::{
    db::{model, MangaDatabase, UserDatabase},
    user::Claims,
};
use tracker::{anilist, myanimelist, AniList, MyAnimeList};

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
            .exchange_code(code, state, csrf_state, pkce_code_verifier)
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
        let token = client.exchange_code(code).await.map(|token| model::Token {
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

        match tracker.as_str() {
            myanimelist::NAME => {
                let tracker_token = ctx
                    .data::<UserDatabase>()?
                    .get_user_tracker_token(myanimelist::NAME, user.sub)
                    .await?;

                let manga_list = ctx
                    .data::<MyAnimeList>()?
                    .get_manga_list(
                        tracker_token.access_token,
                        title,
                        6,
                        0,
                        "id,title,main_picture,synopsis,status".to_string(),
                    )
                    .await?;

                Ok(manga_list
                    .into_iter()
                    .map(|m| TrackerManga {
                        tracker: myanimelist::NAME.to_string(),
                        tracker_manga_id: m.id.to_string(),
                        title: m.title,
                        synopsis: m.synopsis,
                        cover_url: m.main_picture.medium,
                        status: m.status,
                    })
                    .collect())
            }
            anilist::NAME => {
                let tracker_token = ctx
                    .data::<UserDatabase>()?
                    .get_user_tracker_token(anilist::NAME, user.sub)
                    .await?;

                let m = ctx
                    .data::<AniList>()?
                    .search_manga(tracker_token.access_token, title)
                    .await?;

                Ok(vec![TrackerManga {
                    tracker: anilist::NAME.to_string(),
                    tracker_manga_id: m.id.to_string(),
                    title: m
                        .title
                        .and_then(|t| t.romaji)
                        .unwrap_or_else(|| "".to_string()),
                    synopsis: m.description.unwrap_or_else(|| "".to_string()),
                    cover_url: m
                        .cover_image
                        .and_then(|c| c.medium)
                        .unwrap_or_else(|| "".to_string()),
                    status: m.status.unwrap_or_else(|| "".to_string()),
                }])
            }
            _ => Err("tracker not available".into()),
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
            let status = match (
                tracker.tracker.as_str(),
                tracker.tracker_manga_id.to_owned(),
            ) {
                (myanimelist::NAME, Some(tracker_manga_id)) => {
                    let tracker_token = ctx
                        .data::<UserDatabase>()?
                        .get_user_tracker_token(myanimelist::NAME, user.sub)
                        .await?;

                    let tracker_data = ctx
                        .data::<MyAnimeList>()?
                        .get_manga_details(
                            tracker_token.access_token,
                            tracker_manga_id.to_owned(),
                            "title,my_list_status".to_string(),
                        )
                        .await?;

                    let tracker_manga_title = tracker_data.title;
                    if let Some(status) = tracker_data.my_list_status {
                        Some(TrackerStatus {
                            tracker: tracker.tracker.to_owned(),
                            tracker_manga_id: Some(tracker_manga_id),
                            tracker_manga_title: Some(tracker_manga_title),
                            status: status.status,
                            num_chapters_read: Some(status.num_chapters_read),
                            score: Some(status.score),
                            start_date: status.start_date,
                            finish_date: status.finish_date,
                        })
                    } else {
                        Some(TrackerStatus {
                            tracker: tracker.tracker.to_owned(),
                            tracker_manga_id: Some(tracker_manga_id),
                            tracker_manga_title: Some(tracker_manga_title),
                            ..Default::default()
                        })
                    }
                }
                (anilist::NAME, Some(tracker_manga_id)) => {
                    let tracker_token = ctx
                        .data::<UserDatabase>()?
                        .get_user_tracker_token(anilist::NAME, user.sub)
                        .await?;

                    let tracker_data = ctx
                        .data::<AniList>()?
                        .get_manga_details(tracker_token.access_token, tracker_manga_id.parse()?)
                        .await?;

                    let tracker_manga_title = tracker_data
                        .title
                        .and_then(|t| t.romaji)
                        .unwrap_or_else(|| "".to_string());
                    if let Some(status) = tracker_data.media_list_entry {
                        Some(TrackerStatus {
                            tracker: tracker.tracker.to_owned(),
                            tracker_manga_id: Some(tracker_manga_id),
                            tracker_manga_title: Some(tracker_manga_title),
                            status: status.status.and_then(|s| match s {
                                tracker::anilist::MediaListStatus::Current => {
                                    Some("reading".to_string())
                                }
                                tracker::anilist::MediaListStatus::Planning => {
                                    Some("plan_to_read".to_string())
                                }
                                tracker::anilist::MediaListStatus::Completed => {
                                    Some("completed".to_string())
                                }
                                tracker::anilist::MediaListStatus::Dropped => {
                                    Some("dropped".to_string())
                                }
                                tracker::anilist::MediaListStatus::Paused => {
                                    Some("on_hold".to_string())
                                }
                                _ => None,
                            }),
                            num_chapters_read: status.progress,
                            score: status.score,
                            start_date: status
                                .started_at
                                .map(|at| NaiveDateTime::new(at, NaiveTime::from_hms(0, 0, 0))),
                            finish_date: status
                                .completed_at
                                .map(|at| NaiveDateTime::new(at, NaiveTime::from_hms(0, 0, 0))),
                        })
                    } else {
                        Some(TrackerStatus {
                            tracker: tracker.tracker.to_owned(),
                            tracker_manga_id: Some(tracker_manga_id),
                            tracker_manga_title: Some(tracker_manga_title),
                            ..Default::default()
                        })
                    }
                }
                (_, _) => None,
            };

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

        match tracker.as_str() {
            myanimelist::NAME => {
                let mut params = vec![];
                if let Some(status) = status.status.as_ref() {
                    params.push(("status", status.to_owned()));
                }
                if let Some(score) = status.score {
                    params.push(("score", format!("{score}")));
                }
                if let Some(num_chapters_read) = status.num_chapters_read {
                    params.push(("num_chapters_read", format!("{num_chapters_read}")));
                }
                if let Some(start_date) = status.start_date.as_ref() {
                    params.push(("start_date", format!("{start_date}")));
                }
                if let Some(finish_date) = status.finish_date.as_ref() {
                    params.push(("finish_date", format!("{finish_date}")));
                }

                ctx.data::<MyAnimeList>()?
                    .update_my_list_status(tracker_token.access_token, tracker_manga_id, &params)
                    .await?;
            }
            anilist::NAME => {
                let anilist = ctx.data::<AniList>()?;

                let tracker_manga_id: i64 = tracker_manga_id.parse()?;
                let entry_status = status.status.and_then(|s| match s.as_str() {
                    "reading" => Some("CURRENT".to_string()),
                    "completed" => Some("COMPLETED".to_string()),
                    "on_hold" => Some("PAUSED".to_string()),
                    "dropped" => Some("DROPPED".to_string()),
                    "plan_to_read" => Some("PLANNING".to_string()),
                    _ => None,
                });
                let score = status.score.map(|s| s * 10);
                let started_at = status
                    .start_date
                    .map(|at| (at.year() as i64, at.month() as i64, at.day() as i64));
                let completed_at = status
                    .finish_date
                    .map(|at| (at.year() as i64, at.month() as i64, at.day() as i64));

                let id = anilist
                    .get_manga_details(tracker_token.access_token.clone(), tracker_manga_id)
                    .await
                    .map(|res| res.media_list_entry.map(|entry| entry.id))?;
                anilist
                    .save_entry(
                        tracker_token.access_token,
                        id,
                        tracker_manga_id,
                        entry_status,
                        score,
                        status.num_chapters_read,
                        started_at,
                        completed_at,
                    )
                    .await?;
            }
            _ => {}
        }

        Ok(true)
    }
}
