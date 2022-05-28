use crate::domain::{
    entities::{
        library::{Category, LibraryUpdate},
        manga::Manga,
    },
    repositories::library::{LibraryRepository, LibraryRepositoryError},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("repository error: {0}")]
    RepositoryError(#[from] LibraryRepositoryError),
}

pub struct LibraryService<R>
where
    R: LibraryRepository,
{
    repo: R,
}

impl<R> LibraryService<R>
where
    R: LibraryRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn get_categories_by_user_id(
        &self,
        user_id: i64,
    ) -> Result<Vec<Category>, LibraryError> {
        let mut categories = vec![Category::default()];

        let others = self.repo.get_categories_by_user_id(user_id).await?;

        categories.extend_from_slice(&others);

        Ok(categories)
    }

    pub async fn get_category_by_id(&self, id: Option<i64>) -> Result<Category, LibraryError> {
        let category = if let Some(id) = id {
            self.repo.get_category_by_id(id).await?
        } else {
            Category {
                id: None,
                name: "Default".to_string(),
            }
        };

        Ok(category)
    }

    pub async fn create_category(
        &self,
        user_id: i64,
        name: &str,
    ) -> Result<Category, LibraryError> {
        let category = self.repo.create_category(user_id, name).await?;

        Ok(category)
    }

    pub async fn rename_category(&self, id: i64, name: &str) -> Result<Category, LibraryError> {
        let category = self.repo.rename_category(id, name).await?;

        Ok(category)
    }

    pub async fn delete_category(&self, id: i64) -> Result<(), LibraryError> {
        self.repo.delete_category(id).await?;

        Ok(())
    }

    pub async fn get_manga_from_library_by_category_id(
        &self,
        user_id: i64,
        category_id: Option<i64>,
    ) -> Result<Vec<Manga>, LibraryError> {
        let manga = self
            .repo
            .get_manga_from_library_by_category_id(user_id, category_id)
            .await?;

        Ok(manga)
    }

    pub async fn insert_manga_to_library(
        &self,
        user_id: i64,
        manga_id: i64,
        category_ids: Vec<i64>,
    ) -> Result<(), LibraryError> {
        self.repo
            .insert_manga_to_library(user_id, manga_id, &category_ids)
            .await?;

        Ok(())
    }

    pub async fn delete_manga_from_library(
        &self,
        user_id: i64,
        manga_id: i64,
    ) -> Result<(), LibraryError> {
        self.repo
            .delete_manga_from_library(user_id, manga_id)
            .await?;

        Ok(())
    }

    pub async fn get_library_recent_updates(
        &self,
        user_id: i64,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        first: Option<usize>,
        last: Option<usize>,
    ) -> Result<Vec<LibraryUpdate>, LibraryError> {
        let updates = if let Some(first) = first {
            self.repo
                .get_first_library_updates(
                    user_id,
                    after_timestamp,
                    after_id,
                    before_timestamp,
                    before_id,
                    first as i32,
                )
                .await?
        } else if let Some(last) = last {
            self.repo
                .get_last_library_updates(
                    user_id,
                    after_timestamp,
                    after_id,
                    before_timestamp,
                    before_id,
                    last as i32,
                )
                .await?
        } else {
            self.repo
                .get_library_updates(
                    user_id,
                    after_timestamp,
                    after_id,
                    before_timestamp,
                    before_id,
                )
                .await?
        };

        Ok(updates)
    }
}
