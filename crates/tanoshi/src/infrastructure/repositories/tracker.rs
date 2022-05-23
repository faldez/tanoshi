use std::collections::HashMap;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use sqlx::{Row, SqlitePool};
use tanoshi_tracker::{anilist, myanimelist, AniList, MyAnimeList, Session, Tracker, TrackerManga};

use crate::{
    domain::{
        entities::tracker::{Token, TrackedManga},
        repositories::tracker::{TrackerRepository, TrackerRepositoryError},
    },
    infrastructure::database::Pool,
};

pub struct TrackerRepositoryImpl {
    pool: Pool,
    clients: HashMap<&'static str, Box<dyn Tracker>>,
}

impl TrackerRepositoryImpl {
    pub fn new(pool: Pool, mal: Option<MyAnimeList>, anilist: Option<AniList>) -> Self {
        let mut clients = HashMap::new();
        if let Some(mal) = mal {
            clients.insert(myanimelist::NAME, Box::new(mal) as Box<dyn Tracker>);
        }
        if let Some(anilist) = anilist {
            clients.insert(anilist::NAME, Box::new(anilist) as Box<dyn Tracker>);
        }

        Self { pool, clients }
    }
}

#[async_trait]
impl TrackerRepository for TrackerRepositoryImpl {
    fn get_authorize_url(&self, tracker: &str) -> Result<Session, TrackerRepositoryError> {
        let session = self
            .clients
            .get(tracker)
            .ok_or_else(|| TrackerRepositoryError::NoTracker)?
            .get_authorize_url()
            .map_err(|e| TrackerRepositoryError::Other(anyhow::anyhow!("{e}")))?;

        Ok(session)
    }

    async fn exchange_code(
        &self,
        tracker: &str,
        code: String,
        state: Option<String>,
        csrf_state: Option<String>,
        pkce_code_verifier: Option<String>,
    ) -> Result<Token, TrackerRepositoryError> {
        let token = self
            .clients
            .get(tracker)
            .ok_or_else(|| TrackerRepositoryError::NoTracker)?
            .exchange_code(code, state, csrf_state, pkce_code_verifier)
            .await
            .map_err(|e| TrackerRepositoryError::Other(anyhow::anyhow!("{e}")))?;

        Ok(Token {
            token_type: token.token_type,
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_in: token.expires_in,
        })
    }

    async fn refresh_token(
        &self,
        tracker: &str,
        refresh_token: &str,
    ) -> Result<Token, TrackerRepositoryError> {
        match self
            .clients
            .get(tracker)
            .ok_or_else(|| TrackerRepositoryError::NoTracker)?
            .refresh_token(refresh_token.to_string())
            .await
        {
            Ok(token) => Ok(Token {
                token_type: token.token_type,
                access_token: token.access_token,
                refresh_token: token.refresh_token,
                expires_in: token.expires_in,
            }),
            Err(tanoshi_tracker::Error::Unauthorized) => Err(TrackerRepositoryError::Unauthorized),
            Err(e) => Err(TrackerRepositoryError::Other(anyhow::anyhow!("{e}"))),
        }
    }

    async fn insert_tracker_credential(
        &self,
        user_id: i64,
        tracker: &str,
        token: Token,
    ) -> Result<(), TrackerRepositoryError> {
        sqlx::query(
            r#"INSERT INTO tracker_credential(
                user_id,
                tracker,
                token_type,
                expires_in,
                access_token,
                refresh_token
            ) VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(user_id, tracker) DO UPDATE SET
            token_type = excluded.token_type,
            expires_in = excluded.expires_in,
            access_token = excluded.access_token,
            refresh_token = excluded.refresh_token"#,
        )
        .bind(user_id)
        .bind(tracker)
        .bind(token.token_type)
        .bind(token.expires_in)
        .bind(token.access_token)
        .bind(token.refresh_token)
        .execute(&self.pool as &SqlitePool)
        .await?;

        Ok(())
    }

    async fn get_user_tracker_token(
        &self,
        tracker: &str,
        user_id: i64,
    ) -> Result<Token, TrackerRepositoryError> {
        let row = sqlx::query(
            r#"SELECT token_type, access_token, refresh_token, expires_in FROM tracker_credential WHERE user_id = ? AND tracker = ?"#,
        )
        .bind(user_id)
        .bind(tracker)
        .fetch_one(&self.pool as &SqlitePool)
        .await?;

        Ok(Token {
            token_type: row.get(0),
            access_token: row.get(1),
            refresh_token: row.get(2),
            expires_in: row.get(3),
        })
    }

    async fn delete_user_tracker_login(
        &self,
        tracker: &str,
        user_id: i64,
    ) -> Result<u64, TrackerRepositoryError> {
        Ok(
            sqlx::query("DELETE FROM tracker_credential WHERE user_id = ? AND tracker = ?")
                .bind(user_id)
                .bind(tracker)
                .execute(&self.pool as &SqlitePool)
                .await?
                .rows_affected(),
        )
    }

    async fn search_manga(
        &self,
        token: &Token,
        tracker: &str,
        title: &str,
    ) -> Result<Vec<TrackerManga>, TrackerRepositoryError> {
        match self
            .clients
            .get(tracker)
            .ok_or_else(|| TrackerRepositoryError::NoTracker)?
            .search_manga(token.access_token.clone(), title.to_string())
            .await
        {
            Ok(manga) => Ok(manga),
            Err(tanoshi_tracker::Error::Unauthorized) => Err(TrackerRepositoryError::Unauthorized),
            Err(e) => Err(TrackerRepositoryError::Other(anyhow::anyhow!("{e}"))),
        }
    }

    async fn get_tracked_manga_id(
        &self,
        user_id: i64,
        manga_id: i64,
    ) -> Result<Vec<TrackedManga>, TrackerRepositoryError> {
        let query = sqlx::query(
            r#"
        SELECT m.id as manga_id, tc.tracker, tm.tracker_manga_id FROM tracker_credential tc 
        LEFT JOIN manga m ON m.id = ?
        LEFT JOIN tracker_manga tm ON tc.tracker = tm.tracker AND tm.manga_id = m.id
        WHERE tc.user_id = ?;
        "#,
        );

        let rows = query
            .bind(manga_id)
            .bind(user_id)
            .fetch_all(&self.pool as &SqlitePool)
            .await?
            .iter()
            .map(|row| TrackedManga {
                manga_id: row.get(0),
                tracker: row.get(1),
                tracker_manga_id: row.get(2),
            })
            .collect();

        Ok(rows)
    }

    async fn fetch_manga_details(
        &self,
        token: &str,
        tracker: &str,
        tracker_manga_id: i64,
    ) -> Result<TrackerManga, TrackerRepositoryError> {
        match self
            .clients
            .get(tracker)
            .ok_or_else(|| TrackerRepositoryError::NoTracker)?
            .get_manga_details(token.to_string(), tracker_manga_id)
            .await
        {
            Ok(manga) => Ok(manga),
            Err(tanoshi_tracker::Error::Unauthorized) => Err(TrackerRepositoryError::Unauthorized),
            Err(e) => Err(TrackerRepositoryError::Other(anyhow::anyhow!("{e}"))),
        }
    }

    async fn update_manga_tracking_status(
        &self,
        token: &str,
        tracker: &str,
        tracker_manga_id: i64,
        status: Option<String>,
        score: Option<i64>,
        progress: Option<i64>,
        started_at: Option<NaiveDateTime>,
        completed_at: Option<NaiveDateTime>,
    ) -> Result<(), TrackerRepositoryError> {
        match self
            .clients
            .get(tracker)
            .ok_or_else(|| TrackerRepositoryError::NoTracker)?
            .update_tracker_status(
                token.to_string(),
                tracker_manga_id,
                status,
                score,
                progress,
                started_at,
                completed_at,
            )
            .await
        {
            Ok(manga) => Ok(manga),
            Err(tanoshi_tracker::Error::Unauthorized) => Err(TrackerRepositoryError::Unauthorized),
            Err(e) => Err(TrackerRepositoryError::Other(anyhow::anyhow!("{e}"))),
        }
    }

    async fn update_tracker_manga_id(
        &self,
        user_id: i64,
        manga_id: i64,
        tracker: &str,
        tracker_manga_id: &str,
    ) -> Result<(), TrackerRepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO tracker_manga(
                user_id,
                manga_id,
                tracker,
                tracker_manga_id
            ) VALUES (?, ?, ?, ?)"#,
        )
        .bind(user_id)
        .bind(manga_id)
        .bind(tracker)
        .bind(tracker_manga_id)
        .execute(&self.pool as &SqlitePool)
        .await?;

        Ok(())
    }

    async fn delete_tracker_manga_id(
        &self,
        user_id: i64,
        manga_id: i64,
        tracker: &str,
    ) -> Result<(), TrackerRepositoryError> {
        sqlx::query(
            r#"
            DELETE FROM tracker_manga
            WHERE user_id = ? AND manga_id = ? AND tracker = ?
            "#,
        )
        .bind(user_id)
        .bind(manga_id)
        .bind(tracker)
        .execute(&self.pool as &SqlitePool)
        .await?;

        Ok(())
    }
}
