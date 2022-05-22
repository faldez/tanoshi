use super::{
    super::loader::{
        DatabaseLoader, UserFavoriteId, UserFavoritePath, UserLastReadId, UserTrackerMangaId,
        UserUnreadChaptersId,
    },
    Chapter, Source,
};
use crate::{
    db::MangaDatabase,
    infrastructure::{auth::Claims, config::GLOBAL_CONFIG, utils},
};
use async_graphql::{dataloader::DataLoader, Context, Object, Result, SimpleObject};
use chrono::NaiveDateTime;
use rayon::prelude::*;
use tanoshi_vm::extension::SourceBus;

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

impl From<crate::db::model::Manga> for Manga {
    fn from(val: crate::db::model::Manga) -> Self {
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

impl From<tanoshi_lib::models::MangaInfo> for crate::db::model::Manga {
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

impl From<Manga> for crate::db::model::Manga {
    fn from(val: Manga) -> Self {
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
        let detail = ctx.data::<SourceBus>()?.get_source_info(self.source_id)?;
        Ok(format!("{}{}", detail.url, self.path))
    }

    async fn path(&self) -> String {
        self.path.as_str().to_string()
    }

    async fn cover_url(&self) -> Result<String> {
        let secret = &GLOBAL_CONFIG.get().ok_or("secret not set")?.secret;
        Ok(utils::encrypt_url(secret, &self.cover_url)?)
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
        let source = ctx.data::<SourceBus>()?.get_source_info(self.source_id)?;
        Ok(source.into())
    }

    async fn chapters(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "refresh data from source", default = false)] refresh: bool,
    ) -> Result<Vec<Chapter>> {
        let db = ctx.data::<MangaDatabase>()?;

        if !refresh {
            if let Ok(chapters) = db.get_chapters_by_manga_id(self.id).await {
                return Ok(chapters.into_par_iter().map(|c| c.into()).collect());
            }
        }

        let chapters: Vec<crate::db::model::Chapter> = ctx
            .data::<SourceBus>()?
            .get_chapters(self.source_id, self.path.clone())
            .await?
            .into_par_iter()
            .map(|c| {
                let mut c: crate::db::model::Chapter = c.into();
                c.manga_id = self.id;
                c
            })
            .collect();

        if chapters.is_empty() {
            return Ok(vec![]);
        }

        db.insert_chapters(&chapters).await?;

        let chapters = db
            .get_chapters_by_manga_id(self.id)
            .await
            .unwrap_or_default()
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
        let db = ctx.data::<MangaDatabase>()?.clone();
        Ok(db.get_chapter_by_id(id).await?.into())
    }

    async fn next_chapter(&self, ctx: &Context<'_>) -> Result<Option<Chapter>> {
        let db = ctx.data::<MangaDatabase>()?.clone();
        let user = ctx
            .data::<Claims>()
            .map_err(|_| "token not exists, please login")?;

        let mut id = self.id;
        if id == 0 {
            if let Ok(manga) = db
                .get_manga_by_source_path(self.source_id, &self.path)
                .await
            {
                id = manga.id;
            } else {
                return Ok(None);
            }
        }

        Ok(db
            .get_next_chapter_by_manga_id(user.sub, id)
            .await?
            .map(|c| c.into()))
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
