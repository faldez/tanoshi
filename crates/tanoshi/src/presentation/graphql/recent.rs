use async_graphql::{Context, Object, Result};
use chrono::NaiveDateTime;

use crate::{
    domain::services::image::ImageService,
    infrastructure::{
        config::Config,
        domain::repositories::{image::ImageRepositoryImpl, image_cache::ImageCacheRepositoryImpl},
    },
};

pub struct RecentChapter {
    pub manga_id: i64,
    pub chapter_id: i64,
    pub manga_title: String,
    pub cover_url: String,
    pub chapter_title: String,
    pub read_at: NaiveDateTime,
    pub last_page_read: i64,
}

impl From<crate::domain::entities::history::HistoryChapter> for RecentChapter {
    fn from(other: crate::domain::entities::history::HistoryChapter) -> Self {
        Self {
            manga_id: other.manga_id,
            chapter_id: other.chapter_id,
            manga_title: other.manga_title,
            cover_url: other.cover_url,
            chapter_title: other.chapter_title,
            read_at: other.read_at,
            last_page_read: other.last_page_read,
        }
    }
}

#[Object]
impl RecentChapter {
    async fn manga_id(&self) -> i64 {
        self.manga_id
    }

    async fn chapter_id(&self) -> i64 {
        self.chapter_id
    }

    async fn manga_title(&self) -> String {
        self.manga_title.clone()
    }

    async fn cover_url(&self, ctx: &Context<'_>) -> Result<String> {
        let secret = &ctx.data::<Config>()?.secret;

        let cover_url = ctx
            .data::<ImageService<ImageCacheRepositoryImpl, ImageRepositoryImpl>>()?
            .encrypt_image_url(secret, &self.cover_url)?;

        Ok(cover_url)
    }

    async fn chapter_title(&self) -> String {
        self.chapter_title.clone()
    }

    async fn read_at(&self) -> NaiveDateTime {
        self.read_at
    }

    async fn last_page_read(&self) -> i64 {
        self.last_page_read
    }
}

pub struct RecentUpdate {
    pub manga_id: i64,
    pub chapter_id: i64,
    pub manga_title: String,
    pub cover_url: String,
    pub chapter_title: String,
    pub uploaded: NaiveDateTime,
}

impl From<crate::domain::entities::library::LibraryUpdate> for RecentUpdate {
    fn from(other: crate::domain::entities::library::LibraryUpdate) -> Self {
        Self {
            manga_id: other.manga_id,
            chapter_id: other.chapter_id,
            manga_title: other.manga_title,
            cover_url: other.cover_url,
            chapter_title: other.chapter_title,
            uploaded: other.uploaded,
        }
    }
}

#[Object]
impl RecentUpdate {
    async fn manga_id(&self) -> i64 {
        self.manga_id
    }

    async fn chapter_id(&self) -> i64 {
        self.chapter_id
    }

    async fn manga_title(&self) -> String {
        self.manga_title.clone()
    }

    async fn cover_url(&self, ctx: &Context<'_>) -> Result<String> {
        let secret = &ctx.data::<Config>()?.secret;

        let cover_url = ctx
            .data::<ImageService<ImageCacheRepositoryImpl, ImageRepositoryImpl>>()?
            .encrypt_image_url(secret, &self.cover_url)?;

        Ok(cover_url)
    }

    async fn chapter_title(&self) -> String {
        self.chapter_title.clone()
    }

    async fn uploaded(&self) -> NaiveDateTime {
        self.uploaded
    }
}
