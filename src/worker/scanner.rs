use std::{
    fs::{DirEntry, ReadDir},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use crate::{catalogue::LOCAL_ID, db::MangaDatabase};
use chrono::NaiveDateTime;
use fancy_regex::Regex;
use tanoshi_lib::prelude::{Chapter, Manga};

const DEFAULT_COVER_URL: &str = "/images/cover-placeholder.jpg";
// list of supported files, other archive may works but not tested
static SUPPORTED_FILES: phf::Set<&'static str> = phf::phf_set! {
    "cbz",
    "cbr",
};

pub struct Scanner {
    path: PathBuf,

    mangadb: MangaDatabase,
}

impl Scanner {
    pub fn new<P: AsRef<Path>>(path: P, mangadb: MangaDatabase) -> Self {
        let path = PathBuf::new().join(path);
        Self { path, mangadb }
    }

    fn default_cover_url() -> String {
        DEFAULT_COVER_URL.to_string()
    }

    fn filter_supported_files_and_folders(entry: DirEntry) -> Option<DirEntry> {
        if entry.path().is_dir() {
            Some(entry)
        } else {
            entry
                .path()
                .extension()?
                .to_str()
                .map(|ext| SUPPORTED_FILES.contains(&ext.to_lowercase()))
                .and_then(|supported| if supported { Some(entry) } else { None })
        }
    }

    // find first image from an archvie
    fn find_cover_from_archive(path: &PathBuf) -> String {
        match libarchive_rs::ArchiveReader::new(path.display().to_string().as_str())
            .ok()
            .and_then(|mut r| r.next())
        {
            Some(page) => path.join(page).display().to_string(),
            None => Self::default_cover_url(),
        }
    }

    // find first image from a directory
    fn find_cover_from_dir(path: &PathBuf) -> String {
        path.read_dir()
            .ok()
            .map(Self::sort_dir)
            .and_then(|dir| dir.into_iter().next())
            .map(|entry| entry.path().display().to_string())
            .unwrap_or_else(|| Self::default_cover_url())
    }

    fn sort_dir(dir: ReadDir) -> Vec<DirEntry> {
        Self::sort_read_dir_with_reverse(dir, false)
    }

    fn sort_dir_reverse(dir: ReadDir) -> Vec<DirEntry> {
        Self::sort_read_dir_with_reverse(dir, true)
    }

    fn sort_read_dir_with_reverse(dir: ReadDir, reverse: bool) -> Vec<DirEntry> {
        let mut dir: Vec<DirEntry> = dir.into_iter().filter_map(Result::ok).collect();
        dir.sort_by(|a, b| {
            human_sort::compare(
                a.path().display().to_string().as_str(),
                b.path().display().to_string().as_str(),
            )
        });
        if reverse {
            dir.reverse();
        }
        dir
    }

    fn find_cover_url(entry: &PathBuf) -> String {
        if entry.is_file() {
            return Self::find_cover_from_archive(entry);
        }

        let entry_read_dir = match entry.read_dir().map(Self::sort_dir_reverse) {
            Ok(entry_read_dir) => entry_read_dir,
            Err(_) => {
                return Self::default_cover_url();
            }
        };

        let path = match entry_read_dir
            .into_iter()
            .find_map(Self::filter_supported_files_and_folders)
        {
            Some(entry) => entry.path(),
            None => {
                return Self::default_cover_url();
            }
        };

        if path.is_dir() {
            Self::find_cover_from_dir(&path)
        } else if path.is_file() {
            Self::find_cover_from_archive(&path)
        } else {
            Self::default_cover_url()
        }
    }

    fn get_pages_from_archive(
        path: &PathBuf,
        filename: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut files = vec![];
        let reader = libarchive_rs::ArchiveReader::new(filename)?;
        for file in reader {
            files.push(path.join(&file).display().to_string());
        }
        Ok(files)
    }

    fn get_pages_from_dir(path: &PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let pages = path
            .read_dir()?
            .into_iter()
            .filter_map(Result::ok)
            .filter_map(|f| (f.path().is_file()).then(|| f.path().display().to_string()))
            .collect();
        Ok(pages)
    }

    fn map_entry_to_chapter(path: &PathBuf) -> Option<Chapter> {
        let modified = match path
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        {
            Some(modified) => modified.as_secs(),
            None => {
                return None;
            }
        };
        let number_re = match Regex::new(
            r"(?i)(?<=v)(\d+)|(?<=volume)\s*(\d+)|(?<=vol)\s*(\d+)|(?<=\s)(\d+)",
        ) {
            Ok(re) => re,
            Err(_) => {
                return None;
            }
        };
        let file_name = match path.file_stem().and_then(|file_stem| file_stem.to_str()) {
            Some(file_stem) => file_stem.to_string(),
            None => {
                return None;
            }
        };
        let number = match number_re.find(&file_name).ok().and_then(|m| m) {
            Some(mat) => mat.as_str().parse().unwrap_or(0_f64),
            None => 10000_f64,
        };

        Some(Chapter {
            source_id: LOCAL_ID,
            title: file_name,
            path: format!("{}", path.display()),
            number,
            scanlator: "".to_string(),
            uploaded: NaiveDateTime::from_timestamp(modified as i64, 0),
        })
    }

    pub async fn scan(&self) -> Result<(), anyhow::Error> {
        let mut read_dir = tokio::fs::read_dir(&self.path).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            info!("found {}", entry.path().display());

            let manga = if let Ok(manga) = self
                .mangadb
                .get_manga_by_source_path(LOCAL_ID, entry.path().display().to_string().as_str())
                .await
            {
                manga
            } else {
                let mut manga: crate::db::model::Manga = {
                    let m = Manga {
                        source_id: LOCAL_ID,
                        title: entry
                            .path()
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or_default()
                            .to_string(),
                        author: vec![],
                        genre: vec![],
                        status: None,
                        description: None,
                        path: entry.path().to_str().unwrap_or("").to_string(),
                        cover_url: Self::find_cover_url(&entry.path()),
                    };

                    m.into()
                };

                self.mangadb.insert_manga(&mut manga).await?;

                manga
            };

            self.get_chapters(manga.id, &manga.path).await?;
        }
        Ok(())
    }

    async fn check_or_insert_chapter(
        &self,
        manga_id: i64,
        path: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        let chapter = if let Ok(chapter) = self
            .mangadb
            .get_chapter_by_source_path(LOCAL_ID, &path.display().to_string())
            .await
        {
            chapter
        } else if let Some(chapter) = Self::map_entry_to_chapter(path) {
            let mut ch: crate::db::model::Chapter = chapter.into();
            ch.manga_id = manga_id;
            ch.id = self.mangadb.insert_chapter(&ch).await?;
            ch
        } else {
            return Err(anyhow::anyhow!("failed to get chapter"));
        };

        if chapter.pages.is_empty() {
            self.get_pages(chapter.id, &chapter.path).await?;
        }

        Ok(())
    }

    async fn get_chapters(&self, manga_id: i64, path: &str) -> Result<(), anyhow::Error> {
        debug!("scan {} for chapter", path);
        let path = PathBuf::from(path);
        if path.is_file() {
            self.check_or_insert_chapter(manga_id, &path).await?;
        } else {
            let mut read_dir = tokio::fs::read_dir(&path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                self.check_or_insert_chapter(manga_id, &entry.path())
                    .await?;
            }
        }

        Ok(())
    }

    async fn get_pages(&self, chapter_id: i64, filename: &str) -> Result<(), anyhow::Error> {
        debug!("scan {} for pages", filename);

        let path = PathBuf::from(filename.clone());
        let pages = if path.is_dir() {
            Self::get_pages_from_dir(&path).map_err(|e| anyhow::anyhow!("{}", e))?
        } else if path.is_file() {
            Self::get_pages_from_archive(&path, filename).map_err(|e| anyhow::anyhow!("{}", e))?
        } else {
            return Err(anyhow::anyhow!("filename neither file or dir"));
        };

        self.mangadb.insert_pages(chapter_id, &pages).await?;

        Ok(())
    }
}
