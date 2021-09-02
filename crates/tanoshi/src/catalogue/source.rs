use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use crate::{context::GlobalContext, user};
use async_graphql::{Context, Json, Object, Result, SimpleObject};
use serde::{Deserialize, Serialize};
use tanoshi_lib::prelude::{FilterField, Version};

#[derive(Debug, Clone, Deserialize)]
pub struct SourceIndex {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub version: String,
    pub icon: String,
}

impl From<SourceIndex> for Source {
    fn from(index: SourceIndex) -> Self {
        Self {
            id: index.id,
            name: index.name,
            version: index.version,
            icon: index.icon,
            need_login: false,
            has_update: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, SimpleObject)]
pub struct Filters {
    default: String,
    fields: Json<BTreeMap<String, FilterField>>,
}

impl From<tanoshi_lib::data::Filters> for Filters {
    fn from(v: tanoshi_lib::data::Filters) -> Self {
        Self {
            default: v.default,
            fields: Json(v.fields),
        }
    }
}

#[derive(Clone)]
pub struct Source {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub icon: String,
    pub need_login: bool,
    pub has_update: bool,
}

impl From<tanoshi_lib::data::Source> for Source {
    fn from(s: tanoshi_lib::data::Source) -> Self {
        Self {
            id: s.id,
            name: s.name,
            version: s.version.to_string(),
            icon: s.icon,
            need_login: s.need_login,
            has_update: false,
        }
    }
}

#[Object]
impl Source {
    async fn id(&self) -> i64 {
        self.id
    }
    async fn name(&self) -> String {
        self.name.clone()
    }
    async fn version(&self) -> String {
        self.version.clone()
    }
    async fn icon(&self) -> String {
        self.icon.clone()
    }
    async fn need_login(&self) -> bool {
        self.need_login
    }
    async fn has_update(&self) -> bool {
        self.has_update
    }

    async fn filters(&self, ctx: &Context<'_>) -> Result<Option<Filters>> {
        let extensions = ctx.data::<GlobalContext>()?.extensions.clone();
        if let Some(res) = extensions.filters(self.id).await? {
            Ok(Some(res.into()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Default)]
pub struct SourceRoot;

#[Object]
impl SourceRoot {
    async fn installed_sources(&self, ctx: &Context<'_>) -> Result<Vec<Source>> {
        let available_sources_map = {
            let url = "https://raw.githubusercontent.com/faldez/tanoshi-extensions/repo/index.json"
                .to_string();
            let available_sources = reqwest::get(url).await?.json::<Vec<SourceIndex>>().await?;
            let mut available_sources_map = HashMap::new();
            for source in available_sources {
                available_sources_map.insert(source.id, source);
            }
            available_sources_map
        };

        let sources = {
            let extensions = ctx.data::<GlobalContext>()?.extensions.clone();
            let installed_sources = extensions.list().await?;

            let mut sources: Vec<Source> = vec![];
            for source in installed_sources {
                let mut source: Source = source.into();
                if let Some(index) = available_sources_map.get(&source.id) {
                    source.has_update =
                        Version::from_str(&index.version)? > Version::from_str(&source.version)?;
                }
                sources.push(source);
            }
            sources.sort_by(|a, b| a.id.cmp(&b.id));

            sources
        };

        Ok(sources)
    }

    async fn available_sources(&self, ctx: &Context<'_>) -> Result<Vec<Source>> {
        let url = "https://raw.githubusercontent.com/faldez/tanoshi-extensions/repo/index.json"
            .to_string();
        let source_indexes = reqwest::get(url).await?.json::<Vec<SourceIndex>>().await?;
        let extensions = ctx.data::<GlobalContext>()?.extensions.clone();

        let mut sources: Vec<Source> = vec![];
        for index in source_indexes {
            if !extensions.exist(index.id).await? {
                sources.push(index.into());
            }
        }
        Ok(sources)
    }

    async fn source(&self, ctx: &Context<'_>, source_id: i64) -> Result<Source> {
        let exts = ctx.data::<GlobalContext>()?.extensions.clone();
        Ok(exts.detail(source_id).await?.into())
    }
}

#[derive(Default)]
pub struct SourceMutationRoot;

#[Object]
impl SourceMutationRoot {
    async fn install_source(&self, ctx: &Context<'_>, source_id: i64) -> Result<i64> {
        if !user::check_is_admin(ctx)? {
            return Err("Forbidden".into());
        }

        let ctx = ctx.data::<GlobalContext>()?;
        let extensions = ctx.extensions.clone();
        if extensions.exist(source_id).await? {
            return Err("source installed, use updateSource to update".into());
        }

        let url = "https://raw.githubusercontent.com/faldez/tanoshi-extensions/repo/index.json"
            .to_string();
        let source_indexes = reqwest::get(url).await?.json::<Vec<SourceIndex>>().await?;
        let source: SourceIndex = source_indexes
            .iter()
            .find(|index| index.id == source_id)
            .ok_or("source not found")?
            .clone();

        let url = format!(
            "https://raw.githubusercontent.com/faldez/tanoshi-extensions/repo/{}",
            source.path,
        );

        let raw = reqwest::get(url).await?.bytes().await?;
        extensions.install(source.name, &raw).await?;

        Ok(source.id)
    }

    async fn uninstall_source(&self, ctx: &Context<'_>, source_id: i64) -> Result<i64> {
        if !user::check_is_admin(ctx)? {
            return Err("Forbidden".into());
        }

        let ctx = ctx.data::<GlobalContext>()?;
        let extensions = ctx.extensions.clone();

        extensions.unload(source_id).await?;

        Ok(source_id)
    }

    async fn update_source(&self, ctx: &Context<'_>, source_id: i64) -> Result<i64> {
        if !user::check_is_admin(ctx)? {
            return Err("Forbidden".into());
        }

        let ctx = ctx.data::<GlobalContext>()?;
        let extensions = ctx.extensions.clone();
        extensions.exist(source_id).await?;

        let url = "https://raw.githubusercontent.com/faldez/tanoshi-extensions/repo/index.json"
            .to_string();

        let source_indexes = reqwest::get(url).await?.json::<Vec<SourceIndex>>().await?;
        let source: SourceIndex = source_indexes
            .iter()
            .find(|index| index.id == source_id)
            .ok_or("source not found")?
            .clone();

        if extensions.detail(source_id).await?.version == Version::from_str(&source.version)? {
            return Err("No new version".into());
        }

        let url = format!(
            "https://raw.githubusercontent.com/faldez/tanoshi-extensions/repo/{}",
            source.path,
        );
        let raw = reqwest::get(url).await?.bytes().await?;

        extensions.unload(source_id).await?;
        extensions.install(source.name, &raw).await?;

        Ok(source_id)
    }
}
