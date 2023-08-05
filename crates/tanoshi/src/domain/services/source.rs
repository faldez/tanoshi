use std::{collections::HashMap, str::FromStr};

use crate::domain::{
    entities::source::Source,
    repositories::source::{SourceRepository, SourceRepositoryError},
};

use tanoshi_lib::prelude::Version;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("extension return error: {0}")]
    ExtensionError(#[from] SourceRepositoryError),
    #[error("invalid version error: {0}")]
    LibError(#[from] tanoshi_lib::error::Error),
    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct SourceService<R>
where
    R: SourceRepository,
{
    repo: R,
}

impl<R> SourceService<R>
where
    R: SourceRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn get_installed_sources(
        &self,
        repo_url: &str,
        check_update: bool,
    ) -> Result<Vec<Source>, SourceError> {
        let mut sources = self.repo.installed_sources().await?;

        if check_update {
            let available_sources: HashMap<i64, Source> = self
                .repo
                .available_sources(repo_url, false)
                .await?
                .into_iter()
                .map(|s| (s.id, s))
                .collect();

            for source in sources.iter_mut() {
                if let Some(available_source) = available_sources.get(&source.id) {
                    let available_version = Version::from_str(&available_source.version)?;
                    let installed_version = Version::from_str(&source.version)?;

                    source.has_update = available_version > installed_version;
                    debug!(
                        "source {} {available_version} > {installed_version}: {}",
                        source.id, source.has_update
                    );
                }
            }
        }

        Ok(sources)
    }

    pub async fn get_available_sources(&self, repo_url: &str) -> Result<Vec<Source>, SourceError> {
        let sources = self.repo.available_sources(repo_url, true).await?;

        Ok(sources)
    }

    pub async fn get_source_by_id(&self, id: i64) -> Result<Source, SourceError> {
        let source = self.repo.get_source_by_id(id).await?;

        Ok(source)
    }

    pub async fn install_source(&self, repo_url: &str, id: i64) -> Result<(), SourceError> {
        self.repo.install_source(repo_url, id).await?;

        Ok(())
    }

    pub async fn update_source(&self, repo_url: &str, id: i64) -> Result<(), SourceError> {
        self.repo.update_source(repo_url, id).await?;

        Ok(())
    }

    pub async fn uninstall_source(&self, id: i64) -> Result<(), SourceError> {
        self.repo.uninstall_source(id).await?;

        Ok(())
    }
}
