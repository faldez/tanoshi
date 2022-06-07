use std::any::Any;

use crate::infrastructure::domain::repositories::{
    history::HistoryRepositoryImpl, library::LibraryRepositoryImpl, manga::MangaRepositoryImpl,
    tracker::TrackerRepositoryImpl,
};

use super::{
    catalogue::CatalogueRoot,
    categories::{CategoryMutationRoot, CategoryRoot},
    downloads::{DownloadMutationRoot, DownloadRoot},
    library::{LibraryMutationRoot, LibraryRoot},
    notification::NotificationRoot,
    source::{SourceMutationRoot, SourceRoot},
    status::StatusRoot,
    tracking::{TrackingMutationRoot, TrackingRoot},
    user::{UserMutationRoot, UserRoot},
};

use async_graphql::{dataloader::DataLoader, EmptySubscription, MergedObject, Schema};

pub type TanoshiSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    SourceRoot,
    CatalogueRoot,
    LibraryRoot,
    CategoryRoot,
    UserRoot,
    StatusRoot,
    NotificationRoot,
    DownloadRoot,
    TrackingRoot,
);

#[derive(MergedObject, Default)]
pub struct MutationRoot(
    LibraryMutationRoot,
    CategoryMutationRoot,
    UserMutationRoot,
    SourceMutationRoot,
    DownloadMutationRoot,
    TrackingMutationRoot,
);

pub type DatabaseLoader = crate::presentation::graphql::loader::DatabaseLoader<
    HistoryRepositoryImpl,
    LibraryRepositoryImpl,
    MangaRepositoryImpl,
    TrackerRepositoryImpl,
>;

pub struct SchemaBuilder(async_graphql::SchemaBuilder<QueryRoot, MutationRoot, EmptySubscription>);

impl Default for SchemaBuilder {
    fn default() -> Self {
        let builder = Schema::build(
            QueryRoot::default(),
            MutationRoot::default(),
            EmptySubscription::default(),
        );

        Self(builder)
    }
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn loader(self, loader: DatabaseLoader) -> Self {
        Self(self.0.data(DataLoader::new(loader, tokio::spawn)))
    }

    pub fn data<D>(self, data: D) -> Self
    where
        D: Any + Send + Sync,
    {
        Self(self.0.data(data))
    }

    pub fn build(self) -> TanoshiSchema {
        self.0.finish()
    }
}
