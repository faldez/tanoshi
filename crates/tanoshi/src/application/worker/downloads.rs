use crate::{
    domain::{
        entities::{chapter::Chapter, download::DownloadQueue, manga::Manga},
        repositories::{
            chapter::ChapterRepository, download::DownloadRepository, manga::MangaRepository,
        },
    },
    infrastructure::{
        domain::repositories::user::UserRepositoryImpl, local::LocalMangaInfo,
        notification::Notification,
    },
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use reqwest::Url;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use tanoshi_vm::extension::ExtensionManager;
use zip::{ZipArchive, ZipWriter};

use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

use super::updates::ChapterUpdateReceiver;

pub type DownloadSender = UnboundedSender<Command>;
type DownloadReceiver = UnboundedReceiver<Command>;

#[derive(Debug)]
pub enum Command {
    InsertIntoQueue(i64),
    InsertIntoQueueBySourcePath(i64, String),
    Download,
}

pub struct DownloadWorker<C, D, M>
where
    C: ChapterRepository + 'static,
    D: DownloadRepository + 'static,
    M: MangaRepository + 'static,
{
    download_dir: PathBuf,
    client: reqwest::Client,
    chapter_repo: C,
    manga_repo: M,
    download_repo: D,
    ext: ExtensionManager,
    _notifier: Notification<UserRepositoryImpl>,
    tx: DownloadSender,
    rx: DownloadReceiver,
    chapter_update_receiver: ChapterUpdateReceiver,
    auto_download_chapter: bool,
}

impl<C, D, M> DownloadWorker<C, D, M>
where
    C: ChapterRepository + 'static,
    D: DownloadRepository + 'static,
    M: MangaRepository + 'static,
{
    pub fn new<P: AsRef<Path>>(
        dir: P,
        chapter_repo: C,
        manga_repo: M,
        download_repo: D,
        ext: ExtensionManager,
        notifier: Notification<UserRepositoryImpl>,
        download_sender: DownloadSender,
        download_receiver: DownloadReceiver,
        chapter_update_receiver: ChapterUpdateReceiver,
        auto_download_chapter: bool,
    ) -> Self {
        Self {
            download_dir: PathBuf::new().join(dir),
            client: reqwest::ClientBuilder::new().build().unwrap(),
            chapter_repo,
            manga_repo,
            download_repo,
            ext,
            _notifier: notifier,
            tx: download_sender,
            rx: download_receiver,
            chapter_update_receiver,
            auto_download_chapter,
        }
    }

    async fn insert_to_queue(&mut self, chapter: &Chapter) -> Result<(), anyhow::Error> {
        // numbe 1 and greater than 10000 reserved for local source
        if chapter.source_id >= 10000 {
            anyhow::bail!("local source can't be downloaded");
        }

        let priority = self
            .download_repo
            .get_download_queue_last_priority()
            .await?
            .map(|p| p + 1)
            .unwrap_or(0);

        let manga = self.manga_repo.get_manga_by_id(chapter.manga_id).await?;
        let pages = self
            .ext
            .get_pages(manga.source_id, chapter.path.clone())
            .await?;

        let source = self.ext.get_source_info(manga.source_id)?;
        let source_name = source
            .name
            .replace(&['\\', '/', ':', '*', '?', '\"', '<', '>', '|'][..], "");
        let manga_title = manga
            .title
            .replace(&['\\', '/', ':', '*', '?', '\"', '<', '>', '|'][..], "");
        let chapter_title = format!("{} - {}", chapter.number, chapter.title)
            .replace(&['\\', '/', ':', '*', '?', '\"', '<', '>', '|'][..], "");

        let manga_path = self.download_dir.join(&source_name).join(&manga_title);

        self.save_manga_info_if_not_exists(&manga_path, &manga)?;

        let mut queue = vec![];
        let date_added = Utc::now().naive_utc();
        for (rank, page) in pages.iter().enumerate() {
            queue.push(DownloadQueue {
                id: 0,
                source_id: source.id,
                source_name: source_name.clone(),
                manga_id: manga.id,
                manga_title: manga_title.clone(),
                chapter_id: chapter.id,
                chapter_title: chapter_title.clone(),
                rank: rank as _,
                url: page.clone(),
                priority,
                date_added,
            })
        }

        self.download_repo.insert_download_queue(&queue).await?;

        Ok(())
    }

    async fn paused(&self) -> bool {
        self.download_dir.join(".pause").exists()
    }

    fn open_readable_zip_file<P: AsRef<Path>>(&self, archive_path: P) -> Result<ZipArchive<File>> {
        let file = std::fs::OpenOptions::new().read(true).open(&archive_path)?;
        Ok(zip::ZipArchive::new(file)?)
    }

    fn open_or_create_writeble_zip_file<P: AsRef<Path>>(
        &self,
        manga_path: P,
        archive_path: P,
    ) -> Result<ZipWriter<File>> {
        if let Ok(file) = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&archive_path)
        {
            return Ok(zip::ZipWriter::new_append(file)?);
        } else if let Ok(file) =
            std::fs::create_dir_all(manga_path).and_then(|_| std::fs::File::create(&archive_path))
        {
            return Ok(zip::ZipWriter::new(file));
        }

        Err(anyhow!("cannot open or create new zip file"))
    }

    fn save_manga_info_if_not_exists(&self, manga_path: &PathBuf, manga: &Manga) -> Result<()> {
        let path = manga_path.join("details.json");
        if path.exists() {
            return Ok(());
        }

        info!("creating directory: {}", path.display());
        std::fs::create_dir_all(&manga_path)?;

        let manga_info = LocalMangaInfo {
            title: Some(manga.title.clone()),
            author: if manga.author.is_empty() {
                None
            } else {
                Some(manga.author.clone())
            },
            genre: Some(manga.genre.clone()),
            status: manga.status.clone(),
            description: manga.description.clone(),
            cover_path: None,
        };

        let mut file = std::fs::File::create(&path)?;
        serde_json::to_writer_pretty(&mut file, &manga_info)?;

        Ok(())
    }

    async fn download(&mut self) -> Result<()> {
        let queue = match self.download_repo.get_single_download_queue().await? {
            Some(queue) => queue,
            None => {
                info!("no queue");
                return Ok(());
            }
        };

        debug!("got {}", queue.url);

        let url = Url::parse(&queue.url)?;

        let filename = url
            .path_segments()
            .and_then(|seg| seg.last())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("no filename"))?;

        let manga_path = self
            .download_dir
            .join(&queue.source_name)
            .join(&queue.manga_title);

        let archive_path = manga_path.join(format!("{}.cbz", queue.chapter_title));

        if let Ok(mut zip) = self.open_readable_zip_file(&archive_path) {
            if zip.by_name(&filename).is_ok() {
                debug!("file already downloaded, mark as compeleted then skip");
                self.download_repo
                    .mark_single_download_queue_as_completed(queue.id)
                    .await?;
                if !self.paused().await {
                    self.tx.send(Command::Download).unwrap();
                }
                return Ok(());
            }
        }

        let mut zip = self.open_or_create_writeble_zip_file(&manga_path, &archive_path)?;

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let referrer = self
            .ext
            .get_source_info(queue.source_id)
            .map(|s| s.url)
            .unwrap_or_default();

        let contents = self
            .client
            .request(reqwest::Method::GET, url.clone())
            .header("referer", referrer)
            .send()
            .await?
            .bytes()
            .await?;

        zip.start_file(&filename, Default::default())?;

        zip.write_all(contents.to_vec().as_slice())?;

        self.download_repo
            .mark_single_download_queue_as_completed(queue.id)
            .await?;

        if self
            .download_repo
            .get_single_chapter_download_status(queue.chapter_id)
            .await
            .unwrap_or_default()
        {
            self.download_repo
                .update_chapter_downloaded_path(
                    queue.chapter_id,
                    Some(archive_path.display().to_string()),
                )
                .await?;

            self.download_repo
                .delete_single_chapter_download_queue(queue.chapter_id)
                .await?;
        }

        zip.flush()?;

        if !self.paused().await {
            self.tx.send(Command::Download).unwrap();
        }

        Ok(())
    }

    pub async fn run(mut self) {
        if !self.paused().await {
            self.tx.send(Command::Download).unwrap();
        }

        loop {
            tokio::select! {
                Ok(chapter) = self.chapter_update_receiver.recv() => {
                    if self.auto_download_chapter {
                        if let Err(e) = self.insert_to_queue(&chapter.chapter).await {
                            error!("failed to insert queue, reason {e}");
                        } else {
                            let _ = self.tx.send(Command::Download);
                        }
                    }
                }
                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        Command::InsertIntoQueue(chapter_id) => {
                            match self.chapter_repo.get_chapter_by_id(chapter_id).await {
                                Ok(chapter) => {
                                    if let Err(e) = self.insert_to_queue(&chapter).await {
                                        error!("failed to insert queue, reason {}", e);
                                    } else {
                                        self.tx.send(Command::Download).unwrap();
                                    }
                                }
                                Err(e) => {
                                    error!("chapter {} not found, {}", chapter_id, e);
                                }
                            }
                        }
                        Command::InsertIntoQueueBySourcePath(source_id, path) => {
                            match self
                                .chapter_repo
                                .get_chapter_by_source_id_path(source_id, &path)
                                .await
                            {
                                Ok(chapter) => {
                                    if let Err(e) = self.insert_to_queue(&chapter).await {
                                        error!("failed to insert queue, reason {e}");
                                    } else {
                                        let _ = self.tx.send(Command::Download);
                                    }
                                }
                                Err(e) => {
                                    error!("chapter {source_id} {path} not found: {e}");
                                }
                            }
                        }
                        Command::Download => {
                            if !self.paused().await {
                                if let Err(e) = self.download().await {
                                    error!("{e}")
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn channel() -> (DownloadSender, DownloadReceiver) {
    tokio::sync::mpsc::unbounded_channel::<Command>()
}

pub fn start<C, D, M, P>(
    dir: P,
    chapter_repo: C,
    manga_repo: M,
    download_repo: D,
    ext: ExtensionManager,
    notifier: Notification<UserRepositoryImpl>,
    download_sender: DownloadSender,
    download_receiver: DownloadReceiver,
    chapter_update_receiver: ChapterUpdateReceiver,
    auto_download_chapter: bool,
) -> JoinHandle<()>
where
    C: ChapterRepository + 'static,
    D: DownloadRepository + 'static,
    M: MangaRepository + 'static,
    P: AsRef<Path>,
{
    let download_worker = DownloadWorker::new(
        dir,
        chapter_repo,
        manga_repo,
        download_repo,
        ext,
        notifier,
        download_sender,
        download_receiver,
        chapter_update_receiver,
        auto_download_chapter,
    );

    tokio::spawn(download_worker.run())
}
