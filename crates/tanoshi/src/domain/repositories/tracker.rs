use async_trait::async_trait;
use chrono::NaiveDateTime;
use tanoshi_tracker::{Session, TrackerManga, TrackerStatus};
use thiserror::Error;

use crate::domain::entities::tracker::{Token, TrackedManga};

#[derive(Debug, Error)]
pub enum TrackerRepositoryError {
    #[error("tracker not available")]
    NoTracker,
    #[error("database return error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait TrackerRepository {
    fn get_authorize_url(&self, tracker: &str) -> Result<Session, TrackerRepositoryError>;

    async fn exchange_code(
        &self,
        tracker: &str,
        code: String,
        state: Option<String>,
        csrf_state: Option<String>,
        pkce_code_verifier: Option<String>,
    ) -> Result<Token, TrackerRepositoryError>;

    async fn insert_tracker_credential(
        &self,
        user_id: i64,
        tracker: &str,
        token: Token,
    ) -> Result<(), TrackerRepositoryError>;

    async fn get_user_tracker_token(
        &self,
        tracker: &str,
        user_id: i64,
    ) -> Result<Token, TrackerRepositoryError>;

    async fn delete_user_tracker_login(
        &self,
        tracker: &str,
        user_id: i64,
    ) -> Result<u64, TrackerRepositoryError>;

    async fn search_manga(
        &self,
        token: &Token,
        tracker: &str,
        title: &str,
    ) -> Result<Vec<TrackerManga>, TrackerRepositoryError>;

    async fn get_tracked_manga_id(
        &self,
        user_id: i64,
        manga_id: i64,
    ) -> Result<Vec<TrackedManga>, TrackerRepositoryError>;

    async fn fetch_manga_details(
        &self,
        token: &str,
        tracker: &str,
        tracker_manga_id: i64,
    ) -> Result<TrackerManga, TrackerRepositoryError>;

    async fn fetch_manga_tracking_status(
        &self,
        manga_id: i64,
    ) -> Result<Vec<TrackerStatus>, TrackerRepositoryError>;

    #[allow(clippy::too_many_arguments)]
    async fn update_manga_tracking_status(
        &self,
        token: String,
        tracker_manga_id: i64,
        status: Option<String>,
        score: Option<i64>,
        progress: Option<i64>,
        started_at: Option<NaiveDateTime>,
        completed_at: Option<NaiveDateTime>,
    ) -> Result<(), TrackerRepositoryError>;

    async fn update_tracker_manga_id(
        &self,
        user_id: i64,
        manga_id: i64,
        tracker: &str,
        tracker_manga_id: &str,
    ) -> Result<(), TrackerRepositoryError>;

    async fn delete_tracker_manga_id(
        &self,
        user_id: i64,
        manga_id: i64,
        tracker: &str,
    ) -> Result<(), TrackerRepositoryError>;
}
