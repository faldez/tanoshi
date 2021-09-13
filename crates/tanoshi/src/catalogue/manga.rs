use super::{Chapter, Source};
use crate::{context::GlobalContext, user, utils};
use async_graphql::{Context, Object, Result};

/// A type represent manga details, normalized across source
#[derive(Debug)]
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
    pub unread_chapter_count: i64,
}

impl From<&tanoshi_lib::data::Manga> for Manga {
    fn from(m: &tanoshi_lib::data::Manga) -> Self {
        m.clone().into()
    }
}

impl From<tanoshi_lib::data::Manga> for Manga {
    fn from(m: tanoshi_lib::data::Manga) -> Self {
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
            unread_chapter_count: 0,
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
            unread_chapter_count: 0,
        }
    }
}

impl From<tanoshi_lib::data::Manga> for crate::db::model::Manga {
    fn from(m: tanoshi_lib::data::Manga) -> Self {
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

    async fn path(&self) -> String {
        self.path.as_str().to_string()
    }

    async fn cover_url(&self, ctx: &Context<'_>) -> String {
        if let Ok(ctx) = ctx.data::<GlobalContext>() {
            match utils::encrypt_url(&ctx.secret, &self.cover_url) {
                Ok(encrypted_url) => {
                    return encrypted_url;
                }
                Err(e) => {
                    error!("error encrypt url: {}", e);
                }
            }
        }

        "".to_string()
    }

    async fn is_favorite(&self, ctx: &Context<'_>) -> Result<bool> {
        let user = user::get_claims(ctx)?;
        let mangadb = &ctx.data_unchecked::<GlobalContext>().mangadb;

        let mut id = self.id;
        if id == 0 {
            if let Ok(manga) = mangadb
                .get_manga_by_source_path(self.source_id, &self.path)
                .await
            {
                id = manga.id;
            } else {
                return Ok(false);
            }
        }

        if let Ok(fav) = mangadb.is_user_library(user.sub, id).await {
            Ok(fav)
        } else {
            Err("error query".into())
        }
    }

    async fn date_added(&self) -> chrono::NaiveDateTime {
        self.date_added
    }

    async fn unread_chapter_count(&self, ctx: &Context<'_>) -> Result<i64> {
        let user = user::get_claims(ctx)?;
        let mangadb = &ctx.data::<GlobalContext>()?.mangadb;

        let unread_chapter_count = mangadb
            .get_user_library_unread_chapter(user.sub, self.id)
            .await?;

        Ok(unread_chapter_count)
    }

    async fn source(&self, ctx: &Context<'_>) -> Result<Source> {
        let ctx = ctx.data::<GlobalContext>()?;
        let source = ctx.extensions.detail(self.source_id).await?;
        Ok(source.into())
    }

    async fn chapters(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "refresh data from source", default = false)] refresh: bool,
    ) -> Result<Vec<Chapter>> {
        let ctx = ctx.data::<GlobalContext>()?;
        let db = ctx.mangadb.clone();

        if !refresh {
            if let Ok(chapters) = db.get_chapters_by_manga_id(self.id).await {
                return Ok(chapters.into_iter().map(|c| c.into()).collect());
            }
        }

        let chapters: Vec<crate::db::model::Chapter> = ctx
            .extensions
            .get_chapters(self.source_id, self.path.clone())
            .await?
            .into_iter()
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
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<Chapter>>();

        Ok(chapters)
    }

    async fn chapter(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "chapter id")] id: i64,
    ) -> Result<Chapter> {
        let db = ctx.data_unchecked::<GlobalContext>().mangadb.clone();
        Ok(db.get_chapter_by_id(id).await?.into())
    }

    async fn next_chapter(&self, ctx: &Context<'_>) -> Result<Option<Chapter>> {
        let db = ctx.data_unchecked::<GlobalContext>().mangadb.clone();
        let user = user::get_claims(ctx)?;

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
}
