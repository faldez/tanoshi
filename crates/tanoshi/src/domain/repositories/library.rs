use std::collections::HashMap;

use async_trait::async_trait;

use thiserror::Error;

use crate::domain::entities::{
    library::{Category, LibraryUpdate},
    manga::Manga,
};

#[derive(Debug, Error)]
pub enum LibraryRepositoryError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
}

#[async_trait]
pub trait LibraryRepository: Send + Sync {
    async fn get_categories_by_user_id(
        &self,
        user_id: i64,
    ) -> Result<Vec<Category>, LibraryRepositoryError>;

    async fn get_category_by_id(&self, id: i64) -> Result<Category, LibraryRepositoryError>;

    async fn create_category(
        &self,
        user_id: i64,
        name: &str,
    ) -> Result<Category, LibraryRepositoryError>;

    async fn rename_category(
        &self,
        id: i64,
        name: &str,
    ) -> Result<Category, LibraryRepositoryError>;

    async fn delete_category(&self, id: i64) -> Result<(), LibraryRepositoryError>;

    async fn get_category_count(
        &self,
        user_id: i64,
    ) -> Result<HashMap<Option<i64>, i64>, LibraryRepositoryError>;

    async fn get_manga_from_library_by_category_id(
        &self,
        user_id: i64,
        category_id: Option<i64>,
    ) -> Result<Vec<Manga>, LibraryRepositoryError>;

    async fn get_manga_from_library(
        &self,
        user_id: i64,
    ) -> Result<Vec<Manga>, LibraryRepositoryError>;

    async fn insert_manga_to_library(
        &self,
        user_id: i64,
        manga_id: i64,
        category_ids: &[i64],
    ) -> Result<(), LibraryRepositoryError>;

    async fn delete_manga_from_library(
        &self,
        user_id: i64,
        manga_id: i64,
    ) -> Result<(), LibraryRepositoryError>;

    async fn get_first_library_updates(
        &self,
        user_id: i64,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        first: i32,
    ) -> Result<Vec<LibraryUpdate>, LibraryRepositoryError>;

    async fn get_last_library_updates(
        &self,
        user_id: i64,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
        last: i32,
    ) -> Result<Vec<LibraryUpdate>, LibraryRepositoryError>;

    async fn get_library_updates(
        &self,
        user_id: i64,
        after_timestamp: i64,
        after_id: i64,
        before_timestamp: i64,
        before_id: i64,
    ) -> Result<Vec<LibraryUpdate>, LibraryRepositoryError>;
}
