use super::Manga;
use crate::{context::GlobalContext, user, utils};
use async_graphql::{Context, Object, Result, SimpleObject};
use chrono::NaiveDateTime;

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
    pub prev: Option<i64>,
    pub next: Option<i64>,
    pub uploaded: chrono::NaiveDateTime,
    pub date_added: chrono::NaiveDateTime,
    pub read_progress: Option<ReadProgress>,
    pub pages: Vec<String>,
}

impl From<tanoshi_lib::data::Chapter> for Chapter {
    fn from(ch: tanoshi_lib::data::Chapter) -> Self {
        Self {
            id: 0,
            source_id: ch.source_id,
            manga_id: 0,
            title: ch.title,
            path: ch.path,
            number: ch.number,
            scanlator: ch.scanlator,
            prev: None,
            next: None,
            uploaded: ch.uploaded,
            date_added: chrono::NaiveDateTime::from_timestamp(chrono::Local::now().timestamp(), 0),
            read_progress: None,
            pages: vec![],
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
            prev: val.prev,
            next: val.next,
            uploaded: val.uploaded,
            date_added: val.date_added,
            read_progress: None,
            pages: val.pages,
        }
    }
}

impl From<tanoshi_lib::data::Chapter> for crate::db::model::Chapter {
    fn from(ch: tanoshi_lib::data::Chapter) -> Self {
        Self {
            id: 0,
            source_id: ch.source_id,
            manga_id: 0,
            title: ch.title,
            path: ch.path,
            number: ch.number,
            scanlator: ch.scanlator,
            prev: None,
            next: None,
            uploaded: ch.uploaded,
            date_added: chrono::NaiveDateTime::from_timestamp(chrono::Local::now().timestamp(), 0),
            pages: vec![],
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
            prev: val.prev,
            next: val.next,
            uploaded: val.uploaded,
            date_added: val.date_added,
            pages: val.pages,
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

    async fn prev(&self) -> Option<i64> {
        self.prev
    }

    async fn next(&self) -> Option<i64> {
        self.next
    }

    async fn read_progress(&self, ctx: &Context<'_>) -> Result<Option<ReadProgress>> {
        let user = user::get_claims(ctx)?;
        let progress = ctx
            .data_unchecked::<GlobalContext>()
            .mangadb
            .get_user_history_progress(user.sub, self.id)
            .await?
            .map(|r| r.into());

        Ok(progress)
    }

    async fn uploaded(&self) -> NaiveDateTime {
        self.uploaded
    }

    async fn date_added(&self) -> chrono::NaiveDateTime {
        self.date_added
    }

    async fn manga(&self, ctx: &Context<'_>) -> Result<Manga> {
        Ok(ctx
            .data_unchecked::<GlobalContext>()
            .mangadb
            .get_manga_by_id(self.manga_id)
            .await?
            .into())
    }

    async fn pages(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "fetch from source", default = false)] fetch: bool,
    ) -> Result<Vec<String>> {
        info!("pages: {}, fetch: {}", self.pages.len(), fetch);
        let pages = if !self.pages.is_empty() && !fetch {
            self.pages.clone()
        } else {
            let pages = ctx
                .data::<GlobalContext>()?
                .extensions
                .get_pages(self.source_id, self.path.clone())
                .await?;

            ctx.data::<GlobalContext>()?
                .mangadb
                .insert_pages(self.id, &pages)
                .await?;

            pages
        };

        let secret = ctx.data::<GlobalContext>()?.secret.clone();
        let pages = pages
            .iter()
            .map(|page| utils::encrypt_url(&secret, page).unwrap_or_default())
            .collect();

        Ok(pages)
    }
}
