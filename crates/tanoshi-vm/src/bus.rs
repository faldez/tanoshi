use bytes::Bytes;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tanoshi_lib::prelude::{Chapter, Extension, ExtensionResult, Filters, Manga, Param, Source};
use tokio::{
    sync::{mpsc::UnboundedSender, oneshot::Sender},
    time::timeout,
};

use crate::prelude::ExtensionProxy;

pub type ExtensionResultSender<T> = Sender<ExtensionResult<T>>;

#[derive(Debug)]
pub enum Command {
    Insert(i64, Arc<dyn Extension>),
    Load(String),
    Unload(i64),
    Exist(i64, Sender<bool>),
    List(Sender<Vec<Source>>),
    Detail(i64, Sender<Source>),
    Filters(i64, ExtensionResultSender<Option<Filters>>),
    GetMangaList(i64, Param, ExtensionResultSender<Vec<Manga>>),
    GetMangaInfo(i64, String, ExtensionResultSender<Manga>),
    GetChapters(i64, String, ExtensionResultSender<Vec<Chapter>>),
    GetPages(i64, String, ExtensionResultSender<Vec<String>>),
}

#[derive(Debug, Clone)]
pub struct ExtensionBus {
    path: PathBuf,
    tx: UnboundedSender<Command>,
}

impl ExtensionBus {
    pub fn new<P: AsRef<Path>>(path: P, tx: UnboundedSender<Command>) -> Self {
        Self {
            path: PathBuf::new().join(path),
            tx,
        }
    }

    pub async fn insert(
        &self,
        source_id: i64,
        proxy: Arc<dyn Extension>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(self.tx.send(Command::Insert(source_id, proxy))?)
    }

    pub async fn install(
        &self,
        name: String,
        contents: &Bytes,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.path.join(name).with_extension("tanoshi");
        ExtensionProxy::compile(contents, &path)?;

        Ok(self.tx.send(Command::Load(
            path.to_str().ok_or("path can't to string")?.to_string(),
        ))?)
    }

    pub async fn unload(&self, source_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        Ok(self.tx.send(Command::Unload(source_id))?)
    }

    pub async fn exist(&self, source_id: i64) -> Result<bool, Box<dyn std::error::Error>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Command::Exist(source_id, tx))?;

        let exist = timeout(Duration::from_secs(30), rx).await??;
        Ok(exist)
    }

    pub async fn list(&self) -> Result<Vec<Source>, Box<dyn std::error::Error>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Command::List(tx))?;

        let sources = timeout(Duration::from_secs(30), rx).await??;
        Ok(sources)
    }

    pub async fn detail(&self, source_id: i64) -> Result<Source, Box<dyn std::error::Error>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Command::Detail(source_id, tx))?;

        let source = timeout(Duration::from_secs(30), rx).await??;
        Ok(source)
    }

    pub async fn filters(
        &self,
        source_id: i64,
    ) -> Result<Option<Filters>, Box<dyn std::error::Error>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Command::Filters(source_id, tx))?;

        timeout(Duration::from_secs(30), rx).await??.result()
    }

    pub async fn get_manga_list(
        &self,
        source_id: i64,
        param: Param,
    ) -> Result<Vec<Manga>, Box<dyn std::error::Error>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Command::GetMangaList(source_id, param, tx))?;

        timeout(Duration::from_secs(30), rx).await??.result()
    }

    pub async fn get_manga_info(
        &self,
        source_id: i64,
        path: String,
    ) -> Result<Manga, Box<dyn std::error::Error>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Command::GetMangaInfo(source_id, path, tx))?;

        timeout(Duration::from_secs(30), rx).await??.result()
    }

    pub async fn get_chapters(
        &self,
        source_id: i64,
        path: String,
    ) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Command::GetChapters(source_id, path, tx))?;

        timeout(Duration::from_secs(30), rx).await??.result()
    }

    pub async fn get_pages(
        &self,
        source_id: i64,
        path: String,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Command::GetPages(source_id, path, tx))?;

        timeout(Duration::from_secs(30), rx).await??.result()
    }
}
