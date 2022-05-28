use super::{
    super::loader::{DatabaseLoader, MangaId, NextChapterId, PrevChapterId, UserHistoryId},
    Manga, Source,
};
use crate::{
    domain::services::{chapter::ChapterService, image::ImageService, source::SourceService},
    infrastructure::{
        auth::Claims,
        config::GLOBAL_CONFIG,
        repositories::{
            chapter::ChapterRepositoryImpl, image::ImageRepositoryImpl,
            source::SourceRepositoryImpl,
        },
    },
};
use async_graphql::{dataloader::DataLoader, Context, Object, Result, SimpleObject};
use chrono::NaiveDateTime;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

#[derive(Debug, Clone, SimpleObject)]
pub struct ReadProgress {
    pub at: NaiveDateTime,
    pub last_page: i64,
    pub is_complete: bool,
}

impl From<crate::db::model::ReadProgress> for ReadProgress {
    fn from(val: crate::db::model::ReadProgress) -> Self {
        Self {
            at: val.at,
            last_page: val.last_page,
            is_complete: val.is_complete,
        }
    }
}

/// A type represent chapter, normalized across source
#[derive(Debug, Clone)]
pub struct Chapter {
    pub id: i64,
    pub source_id: i64,
    pub manga_id: i64,
    pub title: String,
    pub path: String,
    pub number: f64,
    pub scanlator: String,
    pub uploaded: chrono::NaiveDateTime,
    pub date_added: chrono::NaiveDateTime,
    pub read_progress: Option<ReadProgress>,
    pub downloaded_path: Option<String>,
}

impl From<tanoshi_lib::models::ChapterInfo> for Chapter {
    fn from(ch: tanoshi_lib::models::ChapterInfo) -> Self {
        Self {
            id: 0,
            source_id: ch.source_id,
            manga_id: 0,
            title: ch.title,
            path: ch.path,
            number: ch.number,
            scanlator: ch.scanlator.unwrap_or_default(),
            uploaded: chrono::NaiveDateTime::from_timestamp(ch.uploaded, 0),
            date_added: chrono::NaiveDateTime::from_timestamp(chrono::Local::now().timestamp(), 0),
            read_progress: None,
            downloaded_path: None,
        }
    }
}

impl From<crate::domain::entities::chapter::Chapter> for Chapter {
    fn from(val: crate::domain::entities::chapter::Chapter) -> Self {
        Self {
            id: val.id,
            source_id: val.source_id,
            manga_id: val.manga_id,
            title: val.title,
            path: val.path,
            number: val.number,
            scanlator: val.scanlator,
            uploaded: val.uploaded,
            date_added: val.date_added,
            read_progress: None,
            downloaded_path: val.downloaded_path,
        }
    }
}

impl From<Chapter> for crate::domain::entities::chapter::Chapter {
    fn from(val: Chapter) -> Self {
        Self {
            id: val.id,
            source_id: val.source_id,
            manga_id: val.manga_id,
            title: val.title,
            path: val.path,
            number: val.number,
            scanlator: val.scanlator,
            uploaded: val.uploaded,
            date_added: val.date_added,
            downloaded_path: val.downloaded_path,
        }
    }
}

impl From<crate::db::model::Chapter> for Chapter {
    fn from(val: crate::db::model::Chapter) -> Self {
        Self {
            id: val.id,
            source_id: val.source_id,
            manga_id: val.manga_id,
            title: val.title,
            path: val.path,
            number: val.number,
            scanlator: val.scanlator,
            uploaded: val.uploaded,
            date_added: val.date_added,
            read_progress: None,
            downloaded_path: val.downloaded_path,
        }
    }
}

impl From<tanoshi_lib::models::ChapterInfo> for crate::db::model::Chapter {
    fn from(ch: tanoshi_lib::models::ChapterInfo) -> Self {
        Self {
            id: 0,
            source_id: ch.source_id,
            manga_id: 0,
            title: ch.title,
            path: ch.path,
            number: ch.number,
            scanlator: ch.scanlator.unwrap_or_default(),
            uploaded: chrono::NaiveDateTime::from_timestamp(ch.uploaded, 0),
            date_added: chrono::NaiveDateTime::from_timestamp(chrono::Local::now().timestamp(), 0),
            downloaded_path: None,
        }
    }
}

impl From<Chapter> for crate::db::model::Chapter {
    fn from(val: Chapter) -> Self {
        Self {
            id: val.id,
            source_id: val.source_id,
            manga_id: val.manga_id,
            title: val.title,
            path: val.path,
            number: val.number,
            scanlator: val.scanlator,
            uploaded: val.uploaded,
            date_added: val.date_added,
            downloaded_path: val.downloaded_path,
        }
    }
}

#[Object]
impl Chapter {
    async fn id(&self) -> i64 {
        self.id
    }

    async fn title(&self) -> String {
        self.title.clone()
    }

    async fn path(&self) -> String {
        self.path.clone()
    }

    async fn number(&self) -> f64 {
        self.number
    }

    async fn scanlator(&self) -> String {
        self.scanlator.clone()
    }

    async fn prev(&self, ctx: &Context<'_>) -> Result<Option<i64>> {
        let loader = ctx.data::<DataLoader<DatabaseLoader>>()?;
        Ok(loader.load_one(PrevChapterId(self.id)).await?)
    }

    async fn next(&self, ctx: &Context<'_>) -> Result<Option<i64>> {
        let loader = ctx.data::<DataLoader<DatabaseLoader>>()?;
        Ok(loader.load_one(NextChapterId(self.id)).await?)
    }

    async fn read_progress(&self, ctx: &Context<'_>) -> Result<Option<ReadProgress>> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let loader = ctx.data::<DataLoader<DatabaseLoader>>()?;
        Ok(loader.load_one(UserHistoryId(user.sub, self.id)).await?)
    }

    async fn uploaded(&self) -> NaiveDateTime {
        self.uploaded
    }

    async fn date_added(&self) -> chrono::NaiveDateTime {
        self.date_added
    }

    async fn source(&self, ctx: &Context<'_>) -> Result<Source> {
        let source = ctx
            .data::<SourceService<SourceRepositoryImpl>>()?
            .get_source_by_id(self.source_id)
            .await?
            .into();

        Ok(source)
    }

    async fn manga(&self, ctx: &Context<'_>) -> Result<Manga> {
        let loader = ctx.data::<DataLoader<DatabaseLoader>>()?;
        loader
            .load_one(MangaId(self.manga_id))
            .await?
            .ok_or_else(|| "manga not found".into())
    }

    async fn pages(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "fetch from source", default = false)] _fetch: bool,
        #[graphql(desc = "encrypt url", default = true)] encrypt: bool,
    ) -> Result<Vec<String>> {
        let mut pages = ctx
            .data::<ChapterService<ChapterRepositoryImpl>>()?
            .fetch_chapter_pages(self.source_id, &self.path, &self.downloaded_path)
            .await?;

        let image_svc = ctx.data::<ImageService<ImageRepositoryImpl>>()?;

        if encrypt {
            let secret = &GLOBAL_CONFIG.get().ok_or("secret not set")?.secret;
            pages
                .par_iter_mut()
                .for_each(|p| *p = image_svc.encrypt_image_url(secret, p).unwrap());
        }

        Ok(pages)
    }

    async fn downloaded_path(&self) -> Option<String> {
        self.downloaded_path.clone()
    }
}
