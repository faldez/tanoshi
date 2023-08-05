use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken,
    TokenUrl,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{Error, Tracker, TrackerManga, TrackerStatus};

use super::{Session, Token};

pub const NAME: &str = "myanimelist";

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct MyListStatus {
    pub status: Option<String>,
    pub score: i64,
    pub num_chapters_read: i64,
    pub start_date: Option<NaiveDateTime>,
    pub finish_date: Option<NaiveDateTime>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct MainPicture {
    pub medium: String,
    pub large: String,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Manga {
    pub id: i64,
    pub title: String,
    pub synopsis: String,
    pub main_picture: MainPicture,
    pub status: String,
    pub my_list_status: Option<MyListStatus>,
}

impl From<Manga> for TrackerManga {
    fn from(other: Manga) -> Self {
        Self {
            tracker: NAME.to_string(),
            tracker_manga_id: other.id.to_string(),
            title: other.title.clone(),
            synopsis: other.synopsis,
            cover_url: other.main_picture.medium,
            status: other.status,
            tracker_status: if let Some(status) = other.my_list_status {
                Some(TrackerStatus {
                    tracker: NAME.to_string(),
                    tracker_manga_id: Some(other.id.to_string()),
                    tracker_manga_title: Some(other.title.clone()),
                    status: status.status,
                    num_chapters_read: Some(status.num_chapters_read),
                    score: Some(status.score),
                    start_date: status.start_date,
                    finish_date: status.finish_date,
                })
            } else {
                Some(TrackerStatus {
                    tracker: NAME.to_string(),
                    tracker_manga_id: Some(other.id.to_string()),
                    tracker_manga_title: Some(other.title.clone()),
                    ..Default::default()
                })
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Node<T> {
    pub node: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMangaListResponse {
    pub data: Vec<Node<Manga>>,
}

#[derive(Debug, Clone)]
pub struct MyAnimeList {
    pub oauth_client: BasicClient,
    api_client: reqwest::Client,
}

#[async_trait]
impl Tracker for MyAnimeList {
    fn get_authorize_url(&self) -> Result<Session, Error> {
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_plain();
        let (authorize_url, csrf_state) = self
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_code_challenge)
            .url();
        Ok(Session {
            authorize_url: authorize_url.to_string(),
            csrf_state,
            pkce_code_verifier: Some(pkce_code_verifier),
        })
    }

    async fn exchange_code(
        &self,
        code: String,
        state: Option<String>,
        csrf_state: Option<String>,
        pkce_code_verifier: Option<String>,
    ) -> Result<Token, Error> {
        let code = AuthorizationCode::new(code);

        if let Some((state, csrf_state)) = state.zip(csrf_state) {
            let _state = CsrfToken::new(state);
            let _csrf_state = CsrfToken::new(csrf_state);
        }

        let pkce_code_verifier = PkceCodeVerifier::new(
            pkce_code_verifier.ok_or_else(|| anyhow!("no pkce code verifier"))?,
        );

        let token = self
            .oauth_client
            .exchange_code(code)
            .set_pkce_verifier(pkce_code_verifier)
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
        let manga_list = self
            .get_manga_list(
                token,
                search,
                6,
                0,
                "id,title,main_picture,synopsis,status".to_string(),
            )
            .await?;

        Ok(manga_list.into_iter().map(|m| m.into()).collect())
    }

    async fn get_manga_details(
        &self,
        token: String,
        tracker_manga_id: i64,
    ) -> Result<TrackerManga, Error> {
        let manga = self
            .get_manga_details(
                token,
                tracker_manga_id.to_string(),
                "title,my_list_status".to_string(),
            )
            .await?;
        Ok(manga.into())
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
        let mut params = vec![];
        if let Some(status) = status.as_ref() {
            params.push(("status", status.to_owned()));
        }
        if let Some(score) = score {
            params.push(("score", format!("{score}")));
        }
        if let Some(num_chapters_read) = progress {
            params.push(("num_chapters_read", format!("{num_chapters_read}")));
        }
        if let Some(start_date) = started_at.as_ref() {
            params.push(("start_date", format!("{start_date}")));
        }
        if let Some(finish_date) = completed_at.as_ref() {
            params.push(("finish_date", format!("{finish_date}")));
        }

        self.update_my_list_status(token, tracker_manga_id.to_string(), &params)
            .await?;
        Ok(())
    }
}

impl MyAnimeList {
    pub fn new(base_url: &str, client_id: String, client_secret: String) -> Result<Self, Error> {
        let client_id = ClientId::new(client_id);
        let client_secret = ClientSecret::new(client_secret);
        let authorization_url =
            AuthUrl::new("https://myanimelist.net/v1/oauth2/authorize".to_string())
                .map_err(|e| anyhow!("{e}"))?;
        let token_url = TokenUrl::new("https://myanimelist.net/v1/oauth2/token".to_string())
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
            api_client: reqwest::Client::new(),
        })
    }

    async fn get_manga_list(
        &self,
        token: String,
        q: String,
        limit: i64,
        offset: i64,
        fields: String,
    ) -> Result<Vec<Manga>, Error> {
        let res: GetMangaListResponse = self
            .api_client
            .get("https://api.myanimelist.net/v2/manga")
            .bearer_auth(token)
            .query(&[
                ("q", q),
                ("fields", fields),
                ("nsfw", "true".to_string()),
                ("limit", format!("{limit}")),
                ("offset", format!("{offset}")),
            ])
            .send()
            .await
            .map_err(|e| {
                if e.status() == Some(StatusCode::UNAUTHORIZED) {
                    Error::Unauthorized
                } else {
                    Error::Other(anyhow!("{e}"))
                }
            })?
            .json()
            .await
            .map_err(|e| anyhow!("{e}"))?;
        Ok(res.data.into_iter().map(|node| node.node).collect())
    }

    async fn get_manga_details(
        &self,
        token: String,
        tracker_manga_id: String,
        fields: String,
    ) -> Result<Manga, Error> {
        let res: Manga = self
            .api_client
            .get(format!(
                "https://api.myanimelist.net/v2/manga/{tracker_manga_id}"
            ))
            .bearer_auth(token)
            .query(&[("fields", &fields)])
            .send()
            .await
            .map_err(|e| {
                if e.status() == Some(StatusCode::UNAUTHORIZED) {
                    Error::Unauthorized
                } else {
                    Error::Other(anyhow!("{e}"))
                }
            })?
            .json()
            .await
            .map_err(|e| anyhow!("{e}"))?;

        Ok(res)
    }

    async fn update_my_list_status<T: Serialize + ?Sized>(
        &self,
        token: String,
        tracker_manga_id: String,
        params: &T,
    ) -> Result<(), Error> {
        self.api_client
            .patch(format!(
                "https://api.myanimelist.net/v2/manga/{tracker_manga_id}/my_list_status"
            ))
            .bearer_auth(token)
            .form(params)
            .send()
            .await
            .map_err(|e| {
                if e.status() == Some(StatusCode::UNAUTHORIZED) {
                    Error::Unauthorized
                } else {
                    Error::Other(anyhow!("{e}"))
                }
            })?;

        Ok(())
    }
}
