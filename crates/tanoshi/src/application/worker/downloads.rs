use crate::{
    db::{
        model::{Chapter, DownloadQueue},
        MangaDatabase,
    },
    infrastructure::notifier::Notifier,
};
use anyhow::{anyhow, Result};
use reqwest::Url;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use tanoshi_vm::extension::SourceBus;
use zip::{ZipArchive, ZipWriter};

use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

pub type DownloadSender = UnboundedSender<Command>;
type DownloadReceiver = UnboundedReceiver<Command>;

#[derive(Debug)]
pub enum Command {
    InsertIntoQueue(i64),
    InsertIntoQueueBySourcePath(i64, String),
    Download,
}

pub struct DownloadWorker {
    dir: PathBuf,
    client: reqwest::Client,
    db: MangaDatabase,
    ext: SourceBus,
    _notifier: Notifier,
    tx: DownloadSender,
    rx: DownloadReceiver,
}

impl DownloadWorker {
    pub fn new<P: AsRef<Path>>(
        dir: P,
        db: MangaDatabase,
        ext: SourceBus,
        notifier: Notifier,
    ) -> (Self, DownloadSender) {
        let (tx, rx) = unbounded_channel::<Command>();
        (
            Self {
                dir: PathBuf::new().join(dir),
                client: reqwest::ClientBuilder::new().build().unwrap(),
                db,
                ext,
                _notifier: notifier,
                rx,
                tx: tx.clone(),
            },
            tx,
        )
    }

    async fn insert_to_queue(&mut self, chapter: &Chapter) -> Result<(), anyhow::Error> {
        // numbe 1 and greater than 10000 reserved for local source
        if chapter.source_id >= 10000 {
            anyhow::bail!("local source can't be downloaded");
        }

        let priority = self
            .db
            .get_download_queue_last_priority()
            .await?
            .map(|p| p + 1)
            .unwrap_or(0);

        let manga = self.db.get_manga_by_id(chapter.manga_id).await?;
        let pages = self
            .ext
            .get_pages(manga.source_id, chapter.path.clone())
            .await?;

        let source = self.ext.get_source_info(manga.source_id)?;

        let mut queue = vec![];
        let date_added = chrono::Utc::now().naive_utc();
        for (rank, page) in pages.iter().enumerate() {
            queue.push(DownloadQueue {
                id: 0,
                source_id: source.id,
                source_name: source.name.to_string(),
                manga_id: manga.id,
                manga_title: manga.title.clone(),
                chapter_id: chapter.id,
                chapter_title: format!("{} - {}", chapter.number, chapter.title.clone()),
                rank: rank as _,
                url: page.clone(),
                priority,
                date_added,
            })
        }

        self.db.insert_download_queue(&queue).await?;

        Ok(())
    }

    async fn paused(&self) -> bool {
        self.dir.join(".pause").exists()
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

    async fn download(&mut self) -> Result<()> {
        let mut queue = self
            .db
            .get_single_download_queue()
            .await?
            .ok_or_else(|| anyhow!("no queue"))?;

        debug!("got {}", queue.url);

        let url = Url::parse(&queue.url)?;

        let filename = url
            .path_segments()
            .and_then(|seg| seg.last())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("no filename"))?;

        queue.source_name = queue
            .source_name
            .replace(&['\\', '/', ':', '*', '?', '\"', '<', '>', '|'][..], "");
        queue.manga_title = queue
            .manga_title
            .replace(&['\\', '/', ':', '*', '?', '\"', '<', '>', '|'][..], "");
        queue.chapter_title = queue
            .chapter_title
            .replace(&['\\', '/', ':', '*', '?', '\"', '<', '>', '|'][..], "");

        let manga_path = self.dir.join(&queue.source_name).join(&queue.manga_title);

        let archive_path = manga_path.join(format!("{}.cbz", queue.chapter_title));

        if let Ok(mut zip) = self.open_readable_zip_file(&archive_path) {
            if zip.by_name(&filename).is_ok() {
                debug!("file already downloaded, mark as compeleted then skip");
                self.db
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

        self.db
            .mark_single_download_queue_as_completed(queue.id)
            .await?;

        if self
            .db
            .get_single_chapter_download_status(queue.chapter_id)
            .await
            .unwrap_or_default()
        {
            self.db
                .update_chapter_downloaded_path(
                    queue.chapter_id,
                    Some(archive_path.display().to_string()),
                )
                .await?;

            self.db
                .delete_single_chapter_download_queue(queue.chapter_id)
                .await?;
        }

        zip.flush()?;

        if !self.paused().await {
            self.tx.send(Command::Download).unwrap();
        }

        Ok(())
    }

    pub async fn run(&mut self) {
        if !self.paused().await {
            self.tx.send(Command::Download).unwrap();
        }

        while let Some(cmd) = self.rx.recv().await {
            match cmd {
                Command::InsertIntoQueue(chapter_id) => {
                    match self.db.get_chapter_by_id(chapter_id).await {
                        Ok(chapter) => {
                            if let Err(e) = self.insert_to_queue(&chapter).await {
                                error!("failed to insert queue, reason {}", e);
                                continue;
                            }
                            self.tx.send(Command::Download).unwrap();
                        }
                        Err(e) => {
                            error!("chapter {} not found, {}", chapter_id, e);
                        }
                    }
                }
                Command::InsertIntoQueueBySourcePath(source_id, path) => {
                    match self.db.get_chapter_by_source_path(source_id, &path).await {
                        Some(chapter) => {
                            if let Err(e) = self.insert_to_queue(&chapter).await {
                                error!("failed to insert queue, reason {}", e);
                                continue;
                            }
                            self.tx.send(Command::Download).unwrap();
                        }
                        None => {
                            error!("chapter {} {} not found", source_id, path);
                        }
                    }
                }
                Command::Download => {
                    if self.paused().await {
                        continue;
                    }

                    if let Err(e) = self.download().await {
                        error!("{e}")
                    }
                }
            }
        }
    }
}

pub fn start<P: AsRef<Path>>(
    dir: P,
    mangadb: MangaDatabase,
    ext: SourceBus,
    notifier: Notifier,
) -> (DownloadSender, JoinHandle<()>) {
    let (mut download_worker, tx) = DownloadWorker::new(dir, mangadb, ext, notifier);

    let handle = tokio::spawn(async move {
        download_worker.run().await;
    });

    (tx, handle)
}
