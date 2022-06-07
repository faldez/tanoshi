use super::{
    chapter::Chapter,
    loader::{
        UserFavoriteId, UserFavoritePath, UserLastReadId, UserTrackerMangaId, UserUnreadChaptersId,
    },
    source::Source,
};
use crate::{
    domain::services::{
        chapter::ChapterService, history::HistoryService, image::ImageService,
        source::SourceService,
    },
    infrastructure::{
        auth::Claims,
        config::Config,
        domain::repositories::{
            chapter::ChapterRepositoryImpl, history::HistoryRepositoryImpl,
            image::ImageRepositoryImpl, image_cache::ImageCacheRepositoryImpl,
            source::SourceRepositoryImpl,
        },
    },
    presentation::graphql::schema::DatabaseLoader,
};
use async_graphql::{dataloader::DataLoader, Context, Object, Result, SimpleObject};
use chrono::NaiveDateTime;
use rayon::prelude::*;
use tanoshi_vm::extension::ExtensionManager;

#[derive(Debug, SimpleObject)]
pub struct Tracker {
    pub tracker: String,
    pub tracker_manga_id: Option<String>,
}

/// A type represent manga details, normalized across source
#[derive(Debug, Clone)]
pub struct Manga {
    pub id: i64,
    pub source_id: i64,
    pub title: String,
    pub author: Vec<String>,
    pub genre: Vec<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub path: String,
    pub cover_url: String,
    pub date_added: chrono::NaiveDateTime,
}

impl Default for Manga {
    fn default() -> Self {
        Self {
            id: Default::default(),
            source_id: Default::default(),
            title: Default::default(),
            author: Default::default(),
            genre: Default::default(),
            status: Default::default(),
            description: Default::default(),
            path: Default::default(),
            cover_url: Default::default(),
            date_added: NaiveDateTime::from_timestamp(0, 0),
        }
    }
}

impl From<tanoshi_lib::models::MangaInfo> for Manga {
    fn from(m: tanoshi_lib::models::MangaInfo) -> Self {
        Self {
            id: 0,
            source_id: m.source_id,
            title: m.title,
            author: m.author,
            genre: m.genre,
            status: m.status,
            description: m.description,
            path: m.path,
            cover_url: m.cover_url,
            date_added: chrono::NaiveDateTime::from_timestamp(0, 0),
        }
    }
}

impl From<crate::domain::entities::manga::Manga> for Manga {
    fn from(val: crate::domain::entities::manga::Manga) -> Self {
        Self {
            id: val.id,
            source_id: val.source_id,
            title: val.title,
            author: val.author,
            genre: val.genre,
            status: val.status,
            description: val.description,
            path: val.path,
            cover_url: val.cover_url,
            date_added: val.date_added,
        }
    }
}

#[Object]
impl Manga {
    async fn id(&self) -> i64 {
        self.id
    }

    async fn title(&self) -> String {
        self.title.clone()
    }

    async fn author(&self) -> Vec<String> {
        self.author.clone()
    }

    async fn genre(&self) -> Vec<String> {
        self.genre.clone()
    }

    async fn status(&self) -> Option<String> {
        self.status.clone()
    }

    async fn description(&self) -> Option<String> {
        self.description.clone()
    }

    async fn link(&self, ctx: &Context<'_>) -> Result<String> {
        let detail = ctx
            .data::<ExtensionManager>()?
            .get_source_info(self.source_id)?;
        Ok(format!("{}{}", detail.url, self.path))
    }

    async fn path(&self) -> String {
        self.path.as_str().to_string()
    }

    async fn cover_url(&self, ctx: &Context<'_>) -> Result<String> {
        let secret = &ctx.data::<Config>()?.secret;

        Ok(ctx
            .data::<ImageService<ImageCacheRepositoryImpl, ImageRepositoryImpl>>()?
            .encrypt_image_url(secret, &self.cover_url)?)
    }

    async fn is_favorite(&self, ctx: &Context<'_>) -> Result<bool> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let loader = ctx.data::<DataLoader<DatabaseLoader>>()?;
        let is_favorite: Option<bool> = if self.id == 0 {
            loader
                .load_one(UserFavoritePath(user.sub, self.path.clone()))
                .await?
        } else {
            loader.load_one(UserFavoriteId(user.sub, self.id)).await?
        };

        Ok(is_favorite.unwrap_or(false))
    }

    async fn date_added(&self) -> chrono::NaiveDateTime {
        self.date_added
    }

    async fn unread_chapter_count(&self, ctx: &Context<'_>) -> Result<i64> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        let loader = ctx.data::<DataLoader<DatabaseLoader>>()?;
        Ok(loader
            .load_one(UserUnreadChaptersId(user.sub, self.id))
            .await?
            .unwrap_or(0))
    }

    async fn last_read_at(&self, ctx: &Context<'_>) -> Result<Option<NaiveDateTime>> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        let loader = ctx.data::<DataLoader<DatabaseLoader>>()?;
        Ok(loader.load_one(UserLastReadId(user.sub, self.id)).await?)
    }

    async fn source(&self, ctx: &Context<'_>) -> Result<Source> {
        let source = ctx
            .data::<SourceService<SourceRepositoryImpl>>()?
            .get_source_by_id(self.source_id)
            .await?
            .into();

        Ok(source)
    }

    async fn chapters(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "refresh data from source", default = false)] refresh: bool,
    ) -> Result<Vec<Chapter>> {
        let chapters = ctx
            .data::<ChapterService<ChapterRepositoryImpl>>()?
            .fetch_chapters_by_manga_id(self.source_id, &self.path, self.id, refresh)
            .await?
            .into_par_iter()
            .map(|c| c.into())
            .collect::<Vec<Chapter>>();

        Ok(chapters)
    }

    async fn chapter(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "chapter id")] id: i64,
    ) -> Result<Chapter> {
        let chapter = ctx
            .data::<ChapterService<ChapterRepositoryImpl>>()?
            .fetch_chapter_by_id(id)
            .await?
            .into();

        Ok(chapter)
    }

    async fn next_chapter(&self, ctx: &Context<'_>) -> Result<Option<Chapter>> {
        let claims = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let chapter = ctx
            .data::<HistoryService<ChapterRepositoryImpl, HistoryRepositoryImpl>>()?
            .get_next_chapter(claims.sub, self.id)
            .await?
            .map(|chapter| chapter.into());

        Ok(chapter)
    }

    async fn trackers(&self, ctx: &Context<'_>) -> Result<Vec<Tracker>> {
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;
        let loader = ctx.data::<DataLoader<DatabaseLoader>>()?;
        let data = loader
            .load_one(UserTrackerMangaId(user.sub, self.id))
            .await?
            .unwrap_or_default()
            .into_par_iter()
            .map(|(tracker, tracker_manga_id)| Tracker {
                tracker,
                tracker_manga_id,
            })
            .collect();

        Ok(data)
    }
}
