mod source;
pub use source::{Source, SourceMutationRoot, SourceRoot};

mod manga;
pub use manga::Manga;

mod chapter;
pub use chapter::Chapter;

use crate::context::GlobalContext;

use async_graphql::{Context, Enum, Object, Result};
use tanoshi_lib::prelude::Param;

/// A type represent sort parameter for query manga from source, normalized across sources
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "tanoshi_lib::data::SortByParam")]
pub enum SortByParam {
    LastUpdated,
    Title,
    Comment,
    Views,
}

/// A type represent order parameter for query manga from source, normalized across sources
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "tanoshi_lib::data::SortOrderParam")]
pub enum SortOrderParam {
    Asc,
    Desc,
}

#[derive(Default)]
pub struct CatalogueRoot;

#[Object]
impl CatalogueRoot {
    async fn browse_source(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "source id")] source_id: i64,
        #[graphql(desc = "keyword of the manga")] keyword: Option<String>,
        #[graphql(desc = "genres of the manga")] genres: Option<Vec<String>>,
        #[graphql(desc = "page")] page: Option<i32>,
        #[graphql(desc = "sort by")] sort_by: Option<SortByParam>,
        #[graphql(desc = "sort order")] sort_order: Option<SortOrderParam>,
    ) -> Result<Vec<Manga>> {
        let sort_by = sort_by.map(|s| s.into());
        let sort_order = sort_order.map(|s| s.into());

        let ctx = ctx.data::<GlobalContext>()?;
        let fetched_manga = {
            let extensions = ctx.extensions.clone();
            extensions
                .get_manga_list(
                    source_id,
                    Param {
                        keyword,
                        genres,
                        page,
                        sort_by,
                        sort_order,
                        ..Default::default()
                    },
                )
                .await?
                .iter()
                .map(Manga::from)
                .collect()
        };

        Ok(fetched_manga)
    }

    async fn manga_by_source_path(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "source id")] source_id: i64,
        #[graphql(desc = "path to manga in source")] path: String,
    ) -> Result<Manga> {
        let ctx = ctx.data::<GlobalContext>()?;

        let db = ctx.mangadb.clone();
        let manga = if let Ok(manga) = db.get_manga_by_source_path(source_id, &path).await {
            manga
        } else {
            let mut m: crate::db::model::Manga = {
                let extensions = ctx.extensions.clone();
                extensions.get_manga_info(source_id, path).await?.into()
            };

            db.insert_manga(&mut m).await?;
            m
        };

        Ok(manga.into())
    }

    async fn manga(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "manga id")] id: i64,
        #[graphql(desc = "refresh data from source", default = false)] refresh: bool,
    ) -> Result<Manga> {
        let ctx = ctx.data::<GlobalContext>()?;
        let db = ctx.mangadb.clone();
        let manga = db.get_manga_by_id(id).await?;
        if refresh {
            let mut m: crate::db::model::Manga = {
                let extensions = ctx.extensions.clone();
                extensions
                    .get_manga_info(manga.source_id, manga.path)
                    .await?
                    .into()
            };

            db.insert_manga(&mut m).await?;

            Ok(m.into())
        } else {
            Ok(manga.into())
        }
    }

    async fn chapter(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "chapter id")] id: i64,
    ) -> Result<Chapter> {
        let db = ctx.data_unchecked::<GlobalContext>().mangadb.clone();
        Ok(db.get_chapter_by_id(id).await?.into())
    }
}
