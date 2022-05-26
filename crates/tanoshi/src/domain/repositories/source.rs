use async_trait::async_trait;

use thiserror::Error;

use crate::domain::entities::source::Source;

#[derive(Debug, Error)]
pub enum SourceRepositoryError {
    #[error("extension return error: {0}")]
    ExtensionError(#[from] anyhow::Error),
    #[error("version return error: {0}")]
    VersionError(#[from] tanoshi_lib::error::Error),
    #[error("request return error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("source not found")]
    NotFound,
    #[error("other error: {0}")]
    Other(String),
}

#[async_trait]
pub trait SourceRepository {
    async fn installed_sources(&self) -> Result<Vec<Source>, SourceRepositoryError>;

    async fn available_sources(&self, repo_url: &str)
        -> Result<Vec<Source>, SourceRepositoryError>;
    async fn get_source_by_id(&self, id: i64) -> Result<Source, SourceRepositoryError>;

    async fn install_source(&self, repo_url: &str, id: i64) -> Result<(), SourceRepositoryError>;

    async fn update_source(&self, repo_url: &str, id: i64) -> Result<(), SourceRepositoryError>;

    async fn uninstall_source(&self, id: i64) -> Result<(), SourceRepositoryError>;
}
