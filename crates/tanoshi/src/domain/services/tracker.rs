use chrono::NaiveDateTime;
use tanoshi_tracker::{Session, TrackerManga, TrackerStatus};
use thiserror::Error;

use crate::domain::repositories::tracker::{TrackerRepository, TrackerRepositoryError};

#[derive(Debug, Error)]
pub enum TrackerError {
    #[error("other: {0}")]
    Other(String),
}

impl From<TrackerRepositoryError> for TrackerError {
    fn from(e: TrackerRepositoryError) -> Self {
        match e {
            TrackerRepositoryError::NoTracker => Self::Other(format!("tracker not available")),
            TrackerRepositoryError::DbError(e) => Self::Other(format!("db error: {e}")),
            TrackerRepositoryError::Other(e) => Self::Other(format!("db error: {e}")),
        }
    }
}

pub struct TrackerService<R>
where
    R: TrackerRepository,
{
    repo: R,
}

impl<R> TrackerService<R>
where
    R: TrackerRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn check_tracker_login(
        &self,
        tracker: &str,
        user_id: i64,
    ) -> Result<(), TrackerError> {
        self.repo.get_user_tracker_token(tracker, user_id).await?;

        Ok(())
    }

    pub async fn logout_tracker(&self, tracker: &str, user_id: i64) -> Result<(), TrackerError> {
        self.repo
            .delete_user_tracker_login(tracker, user_id)
            .await?;

        Ok(())
    }

    pub fn login_start(&self, tracker: &str) -> Result<Session, TrackerError> {
        let session = self.repo.get_authorize_url(tracker)?;

        Ok(session)
    }

    pub async fn login_end(
        &self,
        user_id: i64,
        tracker: &str,
        code: String,
        state: Option<String>,
        csrf_state: Option<String>,
        pkce_code_verifier: Option<String>,
    ) -> Result<(), TrackerError> {
        let token = self
            .repo
            .exchange_code(tracker, code, state, csrf_state, pkce_code_verifier)
            .await?;

        self.repo
            .insert_tracker_credential(user_id, tracker, token)
            .await?;

        Ok(())
    }

    pub async fn search_manga(
        &self,
        user_id: i64,
        tracker: &str,
        title: &str,
    ) -> Result<Vec<TrackerManga>, TrackerError> {
        let token = self.repo.get_user_tracker_token(tracker, user_id).await?;

        let res = self.repo.search_manga(&token, tracker, title).await;

        match res {
            Ok(manga) => Ok(manga),
            Err(e) => {
                todo!()
            }
        }
    }

    pub async fn fetch_manga_tracking_status(
        &self,
        user_id: i64,
        manga_id: i64,
    ) -> Result<Vec<TrackerStatus>, TrackerError> {
        let tracked_manga = self.repo.get_tracked_manga_id(user_id, manga_id).await?;

        let mut data: Vec<TrackerStatus> = vec![];
        for manga in tracked_manga {
            let tracker_token = self
                .repo
                .get_user_tracker_token(&manga.tracker, user_id)
                .await?;

            let mut status: Option<TrackerStatus> = None;
            if let Some(tracker_manga_id) = manga.tracker_manga_id.to_owned() {
                status = match self
                    .repo
                    .fetch_manga_details(
                        &tracker_token.access_token,
                        &manga.tracker,
                        tracker_manga_id
                            .parse()
                            .map_err(|e| TrackerError::Other(format!("{e}")))?,
                    )
                    .await
                {
                    Ok(res) => res.tracker_status,
                    Err(e) => {
                        todo!()
                    }
                }
            }

            data.push(status.unwrap_or_else(|| TrackerStatus {
                tracker: manga.tracker,
                ..Default::default()
            }));
        }

        Ok(data)
    }

    pub async fn update_manga_tracking_status(
        &self,
        user_id: i64,
        tracker: &str,
        tracker_manga_id: String,
        status: Option<String>,
        score: Option<i64>,
        progress: Option<i64>,
        started_at: Option<NaiveDateTime>,
        completed_at: Option<NaiveDateTime>,
    ) -> Result<(), TrackerError> {
        let token = self.repo.get_user_tracker_token(tracker, user_id).await?;

        let tracker_manga_id: i64 = tracker_manga_id
            .parse()
            .map_err(|e| TrackerError::Other(format!("{e}")))?;

        if let Err(e) = self
            .repo
            .update_manga_tracking_status(
                token.access_token,
                tracker_manga_id,
                status,
                score,
                progress,
                started_at,
                completed_at,
            )
            .await
        {
            todo!()
        }

        Ok(())
    }

    pub async fn track_manga(
        &self,
        user_id: i64,
        manga_id: i64,
        tracker: &str,
        tracker_manga_id: &str,
    ) -> Result<(), TrackerError> {
        self.repo
            .update_tracker_manga_id(user_id, manga_id, tracker, tracker_manga_id)
            .await?;

        Ok(())
    }

    pub async fn untrack_manga(
        &self,
        user_id: i64,
        manga_id: i64,
        tracker: &str,
    ) -> Result<(), TrackerError> {
        self.repo
            .delete_tracker_manga_id(user_id, manga_id, tracker)
            .await?;

        Ok(())
    }
}
