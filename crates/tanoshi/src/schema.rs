use crate::catalogue::{CatalogueRoot, SourceMutationRoot, SourceRoot};
use crate::library::{LibraryMutationRoot, LibraryRoot};
use crate::notifier::NotificationRoot;
use crate::status::StatusRoot;
use crate::user::{UserMutationRoot, UserRoot};
use async_graphql::{EmptySubscription, MergedObject, Schema};

pub type TanoshiSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    SourceRoot,
    CatalogueRoot,
    LibraryRoot,
    UserRoot,
    StatusRoot,
    NotificationRoot,
);

#[derive(MergedObject, Default)]
pub struct MutationRoot(LibraryMutationRoot, UserMutationRoot, SourceMutationRoot);
