use super::{
    catalogue::{CatalogueRoot, SourceMutationRoot, SourceRoot},
    downloads::{DownloadMutationRoot, DownloadRoot},
    library::{CategoryMutationRoot, CategoryRoot, LibraryMutationRoot, LibraryRoot},
    loader::DatabaseLoader,
    notification::NotificationRoot,
    status::StatusRoot,
    tracking::{TrackingMutationRoot, TrackingRoot},
    user::{UserMutationRoot, UserRoot},
};
use crate::{
    db::{MangaDatabase, UserDatabase},
    notifier::Notifier,
    worker::downloads::DownloadSender,
};
use tanoshi_tracker::{AniList, MyAnimeList};
use tanoshi_vm::extension::SourceBus;

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

pub fn build(
    userdb: UserDatabase,
    mangadb: MangaDatabase,
    ext_manager: SourceBus,
    download_tx: DownloadSender,
    notifier: Notifier,
    mal_client: Option<MyAnimeList>,
    al_client: Option<AniList>,
) -> TanoshiSchema {
    let mut builder = Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription::default(),
    )
    // .extension(ApolloTracing)
    .data(DataLoader::new(
        DatabaseLoader {
            mangadb: mangadb.clone(),
        },
        tokio::spawn,
    ))
    .data(userdb)
    .data(mangadb)
    .data(ext_manager)
    .data(notifier)
    .data(download_tx);

    if let Some(mal_client) = mal_client {
        builder = builder.data(mal_client);
    }

    if let Some(al_client) = al_client {
        builder = builder.data(al_client);
    }

    builder.finish()
}
