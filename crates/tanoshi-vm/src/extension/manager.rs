use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use anyhow::{anyhow, bail, Result};
use fnv::FnvHashMap;
use libloading::Library;
use tanoshi_lib::prelude::{Input, PluginDeclaration, SourceInfo};

use crate::{prelude::Source, PLUGIN_EXTENSION};

#[derive(Clone)]
pub struct ExtensionManager {
    dir: PathBuf,
    extensions: Arc<RwLock<FnvHashMap<i64, Source>>>,
}

impl ExtensionManager {
    pub fn new<P: AsRef<Path>>(extension_dir: P) -> Self {
        Self {
            dir: PathBuf::new().join(extension_dir),
            extensions: Arc::new(RwLock::new(FnvHashMap::default())),
        }
    }

    fn read(&self) -> Result<RwLockReadGuard<FnvHashMap<i64, Source>>> {
        self.extensions
            .read()
            .map_err(|e| anyhow!("failed to lock read: {e}"))
    }

    fn write(&self) -> Result<RwLockWriteGuard<FnvHashMap<i64, Source>>> {
        self.extensions
            .write()
            .map_err(|e| anyhow!("failed to lock write: {e}"))
    }

    pub async fn load_all(&self) -> Result<()> {
        let mut read_dir = tokio::fs::read_dir(&self.dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let mut name = format!("{:?}", entry.file_name());
            name.remove(0);
            name.remove(name.len() - 1);
            if name.ends_with(PLUGIN_EXTENSION) {
                if let Err(e) = self.load(&name).await {
                    error!("failed to load {name}: {e}");
                }
            }
        }
        Ok(())
    }

    pub async fn exists(&self, source_id: i64) -> Result<bool> {
        Ok(self.read()?.get(&source_id).is_some())
    }

    pub async fn list(&self) -> Result<Vec<SourceInfo>> {
        Ok(self
            .read()?
            .values()
            .filter_map(|s| s.extension.get().map(|s| s.get_source_info()))
            .collect())
    }

    pub async fn install(&self, repo_url: &str, name: &str) -> Result<()> {
        let source_file_url = format!(
            "{}/{}/{}.{}",
            repo_url,
            env!("TARGET"),
            name.to_lowercase(),
            PLUGIN_EXTENSION
        );

        info!("downloading {}", source_file_url);

        let contents = reqwest::get(&source_file_url).await?.bytes().await?;

        tokio::fs::write(
            self.dir
                .join(&name.to_lowercase())
                .with_extension(PLUGIN_EXTENSION),
            contents,
        )
        .await?;

        let source = self.load_library(&name.to_lowercase())?;
        self.insert(source).await
    }

    fn load_library(&self, name: &str) -> Result<Source> {
        let library_path = PathBuf::new()
            .join(&self.dir)
            .join(name)
            .with_extension(PLUGIN_EXTENSION);
        info!("load {:?}", library_path.display());

        #[cfg(target_os = "macos")]
        if let Err(e) = std::process::Command::new("install_name_tool")
            .current_dir(library_path.parent().unwrap())
            .arg("-id")
            .arg("''")
            .arg(library_path.file_name().unwrap())
            .output()
        {
            error!("failed to run install_name_tool: {}", e);
        }

        unsafe {
            let library = Library::new(&library_path)?;

            let decl = library
                .get::<*mut PluginDeclaration>(b"plugin_declaration\0")?
                .read();

            if decl.rustc_version != tanoshi_lib::RUSTC_VERSION {
                bail!(
                    "Version mismatch: extension.rustc_version={} != tanoshi_lib.rustc_version={}",
                    decl.rustc_version,
                    tanoshi_lib::RUSTC_VERSION,
                );
            }

            if decl.core_version != tanoshi_lib::LIB_VERSION {
                bail!(
                    "Version mismatch: extension.lib_version={} != tanoshi_lib::lib_version={}",
                    decl.core_version,
                    tanoshi_lib::LIB_VERSION
                );
            }

            let mut registrar = Source::new(library, decl.rustc_version, decl.core_version);
            (decl.register)(&mut registrar);

            Ok(registrar)
        }
    }

    pub async fn load(&self, name: &str) -> Result<()> {
        let mut source = self.load_library(name)?;
        let source_name = source
            .extension
            .get()
            .ok_or_else(|| anyhow!("not initiated"))?
            .get_source_info()
            .name
            .to_lowercase();

        if let Some(preferences) =
            tokio::fs::read_to_string(self.dir.join(source_name).with_extension("json"))
                .await
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
        {
            info!("set preferences");
            source
                .extension
                .get_mut()
                .ok_or_else(|| anyhow!("not initiated"))?
                .set_preferences(preferences)?;
        }
        self.insert(source).await
    }

    pub async fn insert(&self, source: Source) -> Result<()> {
        let info = source
            .extension
            .get()
            .map(|s| s.get_source_info())
            .ok_or_else(|| anyhow!("error"))?;
        self.write()?.insert(info.id, source);
        Ok(())
    }

    pub async fn unload(&self, source_id: i64) -> Result<()> {
        if let Some(source) = self
            .write()?
            .remove(&source_id)
            .and_then(|s| s.extension.get().map(|s| s.get_source_info()))
        {
            std::fs::remove_file(
                self.dir
                    .join(&source.name.to_lowercase())
                    .with_extension(PLUGIN_EXTENSION),
            )?;
        }
        Ok(())
    }

    pub async fn remove(&self, source_id: i64) -> Result<()> {
        self.unload(source_id).await
    }

    pub fn get_version(&self, source_id: i64) -> Result<(String, String)> {
        let lock = self.read()?;
        let source = lock
            .get(&source_id)
            .ok_or_else(|| anyhow!("no such source"))?;
        Ok((source.rustc_version.clone(), source.lib_version.clone()))
    }

    pub fn get_source_info(&self, source_id: i64) -> Result<SourceInfo> {
        Ok(self
            .read()?
            .get(&source_id)
            .ok_or_else(|| anyhow!("no such source"))?
            .extension
            .get()
            .ok_or_else(|| anyhow!("uninitiated"))?
            .get_source_info())
    }

    pub fn filter_list(&self, source_id: i64) -> Result<Vec<Input>> {
        Ok(self
            .read()?
            .get(&source_id)
            .ok_or_else(|| anyhow!("no such source"))?
            .extension
            .get()
            .ok_or_else(|| anyhow!("uninitiated"))?
            .filter_list())
    }

    pub fn get_preferences(&self, source_id: i64) -> Result<Vec<Input>> {
        self.read()?
            .get(&source_id)
            .ok_or_else(|| anyhow!("no such source"))?
            .extension
            .get()
            .ok_or_else(|| anyhow!("uninitiated"))?
            .get_preferences()
    }

    pub async fn set_preferences(&self, source_id: i64, preferences: Vec<Input>) -> Result<()> {
        self.write()?
            .get_mut(&source_id)
            .ok_or_else(|| anyhow!("no such source"))?
            .extension
            .get_mut()
            .ok_or_else(|| anyhow!("uninitiated"))?
            .set_preferences(preferences.clone())?;

        let source_info = self.get_source_info(source_id)?;
        tokio::fs::write(
            self.dir
                .join(source_info.name.to_lowercase())
                .with_extension("json"),
            serde_json::to_string_pretty(&preferences)?,
        )
        .await?;

        Ok(())
    }

    pub async fn get_popular_manga(
        &self,
        source_id: i64,
        page: i64,
    ) -> Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        let extensions = self.extensions.clone();
        tokio::task::spawn_blocking(move || {
            extensions
                .read()
                .map_err(|e| anyhow!("failed to lock read: {e}"))?
                .get(&source_id)
                .ok_or_else(|| anyhow!("no such source"))?
                .extension
                .get()
                .ok_or_else(|| anyhow!("uninitiated"))?
                .get_popular_manga(page)
        })
        .await?
    }

    pub async fn get_latest_manga(
        &self,
        source_id: i64,
        page: i64,
    ) -> Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        let extensions = self.extensions.clone();
        tokio::task::spawn_blocking(move || {
            extensions
                .read()
                .map_err(|e| anyhow!("failed to lock read: {e}"))?
                .get(&source_id)
                .ok_or_else(|| anyhow!("no such source"))?
                .extension
                .get()
                .ok_or_else(|| anyhow!("uninitiated"))?
                .get_latest_manga(page)
        })
        .await?
    }

    pub async fn search_manga(
        &self,
        source_id: i64,
        page: i64,
        query: Option<String>,
        filters: Option<Vec<Input>>,
    ) -> Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        let extensions = self.extensions.clone();
        tokio::task::spawn_blocking(move || {
            extensions
                .read()
                .map_err(|e| anyhow!("failed to lock read: {e}"))?
                .get(&source_id)
                .ok_or_else(|| anyhow!("no such source"))?
                .extension
                .get()
                .ok_or_else(|| anyhow!("uninitiated"))?
                .search_manga(page, query, filters)
        })
        .await?
    }

    pub async fn get_manga_detail(
        &self,
        source_id: i64,
        path: String,
    ) -> Result<tanoshi_lib::prelude::MangaInfo> {
        let extensions = self.extensions.clone();
        tokio::task::spawn_blocking(move || {
            extensions
                .read()
                .map_err(|e| anyhow!("failed to lock read: {e}"))?
                .get(&source_id)
                .ok_or_else(|| anyhow!("no such source"))?
                .extension
                .get()
                .ok_or_else(|| anyhow!("uninitiated"))?
                .get_manga_detail(path)
        })
        .await?
    }

    pub async fn get_chapters(
        &self,
        source_id: i64,
        path: String,
    ) -> Result<Vec<tanoshi_lib::prelude::ChapterInfo>> {
        let extensions = self.extensions.clone();
        tokio::task::spawn_blocking(move || {
            extensions
                .read()
                .map_err(|e| anyhow!("failed to lock read: {e}"))?
                .get(&source_id)
                .ok_or_else(|| anyhow!("no such source"))?
                .extension
                .get()
                .ok_or_else(|| anyhow!("uninitiated"))?
                .get_chapters(path)
        })
        .await?
    }

    pub async fn get_pages(&self, source_id: i64, path: String) -> Result<Vec<String>> {
        let extensions = self.extensions.clone();
        tokio::task::spawn_blocking(move || {
            extensions
                .read()
                .map_err(|e| anyhow!("failed to lock read: {e}"))?
                .get(&source_id)
                .ok_or_else(|| anyhow!("no such source"))?
                .extension
                .get()
                .ok_or_else(|| anyhow!("uninitiated"))?
                .get_pages(path)
        })
        .await?
    }
}
