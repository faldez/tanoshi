use chrono::NaiveDateTime;
use tanoshi_tracker::{Session, TrackerManga, TrackerStatus};
use thiserror::Error;

use crate::domain::{
    entities::tracker::TrackedManga,
    repositories::tracker::{TrackerRepository, TrackerRepositoryError},
};

#[derive(Debug, Error)]
pub enum TrackerError {
    #[error("repository error: {0}")]
    RepositoryError(#[from] TrackerRepositoryError),
    #[error("other error: {0}")]
    Other(String),
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

    pub async fn logout_tracker(&self, user_id: i64, tracker: &str) -> Result<(), TrackerError> {
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

    pub async fn get_tracked_manga_id(
        &self,
        user_id: i64,
        manga_id: i64,
    ) -> Result<Vec<TrackedManga>, TrackerRepositoryError> {
        let tracked_manga = self.repo.get_tracked_manga_id(user_id, manga_id).await?;

        Ok(tracked_manga)
    }

    pub async fn search_manga(
        &self,
        user_id: i64,
        tracker: &str,
        title: &str,
    ) -> Result<Vec<TrackerManga>, TrackerError> {
        let mut tracker_token = self.repo.get_user_tracker_token(tracker, user_id).await?;

        for _ in 0..2 {
            match self.repo.search_manga(&tracker_token, tracker, title).await {
                Ok(manga) => {
                    return Ok(manga);
                }
                Err(TrackerRepositoryError::Unauthorized) => {
                    let token = self
                        .repo
                        .refresh_token(tracker, &tracker_token.refresh_token)
                        .await?;

                    self.repo
                        .insert_tracker_credential(user_id, tracker, token)
                        .await?;

                    tracker_token = self.repo.get_user_tracker_token(tracker, user_id).await?;
                }
                Err(e) => {
                    error!("error search manga, retry");
                    return Err(e.into());
                }
            }
        }

        Err(TrackerError::Other(format!("failed to search manga")))
    }

    pub async fn fetch_manga_tracking_status(
        &self,
        user_id: i64,
        manga_id: i64,
    ) -> Result<Vec<TrackerStatus>, TrackerError> {
        let tracked_manga = self.repo.get_tracked_manga_id(user_id, manga_id).await?;

        let mut data: Vec<TrackerStatus> = vec![];
        for manga in tracked_manga {
            let mut tracker_token = self
                .repo
                .get_user_tracker_token(&manga.tracker, user_id)
                .await?;

            let mut status: Option<TrackerStatus> = None;
            if let Some(tracker_manga_id) = manga
                .tracker_manga_id
                .to_owned()
                .and_then(|id| id.parse::<i64>().ok())
            {
                for _ in 0..2 {
                    match self
                        .repo
                        .fetch_manga_details(
                            &tracker_token.access_token,
                            &manga.tracker,
                            tracker_manga_id,
                        )
                        .await
                    {
                        Ok(res) => status = res.tracker_status,
                        Err(TrackerRepositoryError::Unauthorized) => {
                            let token = self
                                .repo
                                .refresh_token(&manga.tracker, &tracker_token.refresh_token)
                                .await?;

                            self.repo
                                .insert_tracker_credential(user_id, &manga.tracker, token)
                                .await?;

                            tracker_token = self
                                .repo
                                .get_user_tracker_token(&manga.tracker, user_id)
                                .await?;
                        }
                        Err(e) => {
                            error!("error search manga: {e}, retry");
                            return Err(e.into());
                        }
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
        let mut tracker_token = self.repo.get_user_tracker_token(tracker, user_id).await?;

        let tracker_manga_id: i64 = tracker_manga_id
            .parse()
            .map_err(|e| TrackerError::Other(format!("{e}")))?;

        for _ in 0..2 {
            match self
                .repo
                .update_manga_tracking_status(
                    &tracker_token.access_token,
                    tracker,
                    tracker_manga_id,
                    status.clone(),
                    score,
                    progress,
                    started_at,
                    completed_at,
                )
                .await
            {
                Ok(()) => {
                    return Ok(());
                }
                Err(TrackerRepositoryError::Unauthorized) => {
                    let token = self
                        .repo
                        .refresh_token(tracker, &tracker_token.refresh_token)
                        .await?;

                    self.repo
                        .insert_tracker_credential(user_id, tracker, token)
                        .await?;

                    tracker_token = self.repo.get_user_tracker_token(tracker, user_id).await?;
                }
                Err(e) => {
                    error!("error search manga, retry");
                    return Err(e.into());
                }
            }
        }

        Err(TrackerError::Other(
            "failed to update manga tracking".to_string(),
        ))
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
