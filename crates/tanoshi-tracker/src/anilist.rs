use std::collections::HashMap;

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, RefreshToken, TokenUrl,
};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::{Error, Tracker, TrackerManga, TrackerStatus};

use super::{Session, Token};

pub const NAME: &str = "anilist";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MediaListStatus {
    Current,
    Planning,
    Completed,
    Dropped,
    Paused,
    Repeating,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct CoverImage {
    pub large: Option<String>,
    pub medium: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct MediaTitle {
    pub romaji: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaListEntry {
    pub id: i64,
    pub status: Option<MediaListStatus>,
    pub progress: Option<i64>,
    pub score: Option<i64>,
    pub started_at: Option<NaiveDate>,
    pub completed_at: Option<NaiveDate>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub id: i64,
    pub title: Option<MediaTitle>,
    pub description: Option<String>,
    pub cover_image: Option<CoverImage>,
    pub status: Option<String>,
    pub media_list_entry: Option<MediaListEntry>,
}

impl From<Media> for TrackerManga {
    fn from(other: Media) -> Self {
        let title = other
            .title
            .and_then(|t| t.romaji)
            .unwrap_or_else(|| "".to_string());
        Self {
            tracker: NAME.to_string(),
            tracker_manga_id: other.id.to_string(),
            title: title.clone(),
            synopsis: other.description.unwrap_or_else(|| "".to_string()),
            cover_url: other
                .cover_image
                .and_then(|c| c.medium)
                .unwrap_or_else(|| "".to_string()),
            status: other.status.unwrap_or_else(|| "".to_string()),
            tracker_status: if let Some(status) = other.media_list_entry {
                Some(TrackerStatus {
                    tracker: NAME.to_string(),
                    tracker_manga_id: Some(other.id.to_string()),
                    tracker_manga_title: Some(title),
                    status: status.status.and_then(|s| match s {
                        MediaListStatus::Current => Some("reading".to_string()),
                        MediaListStatus::Planning => Some("plan_to_read".to_string()),
                        MediaListStatus::Completed => Some("completed".to_string()),
                        MediaListStatus::Dropped => Some("dropped".to_string()),
                        MediaListStatus::Paused => Some("on_hold".to_string()),
                        _ => None,
                    }),
                    num_chapters_read: status.progress,
                    score: status.score,
                    start_date: status.started_at.map(|at| {
                        NaiveDateTime::new(at, NaiveTime::from_hms_opt(0, 0, 0).unwrap_or_default())
                    }),
                    finish_date: status.completed_at.map(|at| {
                        NaiveDateTime::new(at, NaiveTime::from_hms_opt(0, 0, 0).unwrap_or_default())
                    }),
                })
            } else {
                Some(TrackerStatus {
                    tracker: NAME.to_string(),
                    tracker_manga_id: Some(other.id.to_string()),
                    tracker_manga_title: Some(title),
                    ..Default::default()
                })
            },
        }
    }
}

pub struct AniList {
    pub oauth_client: BasicClient,
}

#[async_trait]
impl Tracker for AniList {
    fn get_authorize_url(&self) -> Result<Session, Error> {
        let (authorize_url, csrf_state) =
            self.oauth_client.authorize_url(CsrfToken::new_random).url();

        Ok(Session {
            authorize_url: authorize_url.to_string(),
            csrf_state,
            pkce_code_verifier: None,
        })
    }

    async fn exchange_code(
        &self,
        code: String,
        _state: Option<String>,
        _csrf_state: Option<String>,
        _pkce_code_verifier: Option<String>,
    ) -> Result<Token, Error> {
        let code = AuthorizationCode::new(code);

        let token = self
            .oauth_client
            .exchange_code(code)
            .request_async(async_http_client)
            .await
            .map_err(|e| anyhow!("{e}"))?;

        let token_str = serde_json::to_string(&token).map_err(|e| anyhow!("{e}"))?;
        Ok(serde_json::from_str(&token_str).map_err(|e| anyhow!("{e}"))?)
    }

    async fn refresh_token(&self, refresh_token: String) -> Result<Token, Error> {
        let token = self
            .oauth_client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await
            .map_err(|e| anyhow!("{e}"))?;
        let token_str = serde_json::to_string(&token).map_err(|e| anyhow!("{e}"))?;
        Ok(serde_json::from_str(&token_str).map_err(|e| anyhow!("{e}"))?)
    }

    async fn search_manga(
        &self,
        token: String,
        search: String,
    ) -> Result<Vec<TrackerManga>, Error> {
        const QUERY: &str = "
        query SearchManga($search: String!) {
            Media(search: $search, format_in: [MANGA, ONE_SHOT]) {
              id
              title {
                romaji
              }
              description(asHtml: false)
              coverImage {
                large
                medium
              }
              status
            }
          }
        ";

        let res = self
            .post_graphql(
                token,
                &json!({
                    "query": QUERY,
                    "variables": {
                        "search": search
                    }
                }),
            )
            .await?;

        debug!("res: {res:?}");

        let res = res
            .get("data")
            .and_then(|data| data.get("Media"))
            .map(|media| media.to_owned())
            .ok_or_else(|| anyhow!("no data"))?;

        let media: Media = serde_json::from_value(res).map_err(|e| anyhow!("{e}"))?;
        Ok(vec![media.into()])
    }

    async fn get_manga_details(
        &self,
        token: String,
        tracker_manga_id: i64,
    ) -> Result<TrackerManga, Error> {
        let media = self.get_media_details(token, tracker_manga_id).await?;
        Ok(media.into())
    }

    async fn update_tracker_status(
        &self,
        token: String,
        tracker_manga_id: i64,
        status: Option<String>,
        score: Option<i64>,
        progress: Option<i64>,
        started_at: Option<NaiveDateTime>,
        completed_at: Option<NaiveDateTime>,
    ) -> Result<(), Error> {
        let entry_status = status.and_then(|s| match s.as_str() {
            "reading" => Some("CURRENT".to_string()),
            "completed" => Some("COMPLETED".to_string()),
            "on_hold" => Some("PAUSED".to_string()),
            "dropped" => Some("DROPPED".to_string()),
            "plan_to_read" => Some("PLANNING".to_string()),
            _ => None,
        });
        let score = score.map(|s| s * 10);
        let started_at =
            started_at.map(|at| (at.year() as i64, at.month() as i64, at.day() as i64));
        let completed_at =
            completed_at.map(|at| (at.year() as i64, at.month() as i64, at.day() as i64));

        let id = self
            .get_media_details(token.clone(), tracker_manga_id)
            .await
            .map(|res| res.media_list_entry.map(|entry| entry.id))?;

        self.save_entry(
            token,
            id,
            tracker_manga_id,
            entry_status,
            score,
            progress,
            started_at,
            completed_at,
        )
        .await?;
        Ok(())
    }
}

impl AniList {
    pub fn new(base_url: &str, client_id: String, client_secret: String) -> Result<Self, Error> {
        let client_id = ClientId::new(client_id);
        let client_secret = ClientSecret::new(client_secret);
        let authorization_url =
            AuthUrl::new("https://anilist.co/api/v2/oauth/authorize".to_string())
                .map_err(|e| anyhow!("{e}"))?;
        let token_url = TokenUrl::new("https://anilist.co/api/v2/oauth/token".to_string())
            .map_err(|e| anyhow!("{e}"))?;

        let redirect_url = RedirectUrl::new(format!("{base_url}/tracker/{NAME}/redirect"))
            .map_err(|e| anyhow!("{e}"))?;
        let client = BasicClient::new(
            client_id,
            Some(client_secret),
            authorization_url,
            Some(token_url),
        )
        .set_redirect_uri(redirect_url);

        Ok(Self {
            oauth_client: client,
        })
    }

    async fn get_media_details(&self, token: String, media_id: i64) -> Result<Media, Error> {
        const QUERY: &str = "
        query GetManga($id: Int!) {
            Media(id: $id) {
              id
              title {
                romaji
              }
              description(asHtml: false)
              coverImage {
                large
                medium
              }
              status
              mediaListEntry {
                id
                status
                progress
                score(format: POINT_10)
                # startedAt {
                #   year
                #   month
                #   day
                # }
                # completedAt {
                #   year
                #   month
                #   day
                # }
              }
            }
          }
        ";

        let res = self
            .post_graphql(
                token,
                &json!({
                    "query": QUERY,
                    "variables": {
                        "id": media_id
                    }
                }),
            )
            .await?;

        debug!("res: {res:?}");

        let res = res
            .get("data")
            .and_then(|data| data.get("Media"))
            .map(|media| media.to_owned())
            .ok_or_else(|| anyhow!("no data"))?;

        let media: Media = serde_json::from_value(res).map_err(|e| anyhow!("{e}"))?;
        Ok(media)
    }

    #[allow(clippy::too_many_arguments)]
    async fn save_entry(
        &self,
        token: String,
        id: Option<i64>,
        tracker_manga_id: i64,
        status: Option<String>,
        score: Option<i64>,
        progress: Option<i64>,
        started_at: Option<(i64, i64, i64)>,
        completed_at: Option<(i64, i64, i64)>,
    ) -> Result<(), Error> {
        const QUERY: &str = "
        mutation SaveEntry(
            $id: Int, 
            $mediaId: Int!, 
            $status: MediaListStatus, 
            $score: Int, 
            $progress: Int, 
            $startedAt: FuzzyDateInput, 
            $completedAt: FuzzyDateInput) {
            SaveMediaListEntry(
                id: $id, 
                mediaId: $mediaId, 
                status: $status, 
                scoreRaw: $score, 
                progress: $progress, 
                startedAt: $startedAt, 
                completedAt: $completedAt
            ) {
                id
            }
        }
        ";

        let mut variables = HashMap::new();
        variables.insert("mediaId", json!(tracker_manga_id));
        if let Some(id) = id {
            variables.insert("id", json!(id));
        }
        if let Some(status) = status {
            variables.insert("status", json!(status));
        }
        if let Some(score) = score {
            variables.insert("score", json!(score));
        }
        if let Some(progress) = progress {
            variables.insert("progress", json!(progress));
        }
        if let Some((year, month, day)) = started_at {
            variables.insert(
                "startedAt",
                json!({
                    "year": year,
                    "month": month,
                    "day": day
                }),
            );
        }
        if let Some((year, month, day)) = completed_at {
            variables.insert(
                "completedAt",
                json!({
                    "year": year,
                    "month": month,
                    "day": day
                }),
            );
        }

        let res = self
            .post_graphql(
                token,
                &json!({
                    "query": QUERY,
                    "variables": variables
                }),
            )
            .await?;

        debug!("res: {res:?}");

        Ok(())
    }

    async fn post_graphql(
        &self,
        token: String,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Error> {
        Ok(reqwest::Client::new()
            .post("https://graphql.anilist.co/")
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| {
                if e.status() == Some(StatusCode::UNAUTHORIZED) {
                    Error::Unauthorized
                } else {
                    Error::Other(anyhow!("{e}"))
                }
            })?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| anyhow!("{e}"))?)
    }
}
