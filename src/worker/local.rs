use std::{
    ffi::OsStr,
    fs::{DirEntry, ReadDir},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use anyhow::anyhow;
use chrono::NaiveDateTime;
use fancy_regex::Regex;
use phf::phf_set;
use rayon::prelude::*;
use tanoshi_lib::prelude::{Chapter, Manga};

// list of supported files, other archive may works but no tested
static SUPPORTED_FILES: phf::Set<&'static str> = phf_set! {
    "cbz",
    "cbr",
};
static DEFAULT_COVER_URL: &'static str = "/images/cover-placeholder.jpg";

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
    libarchive_rs::list_archive_files(path.display().to_string().as_str())
        .ok()
        .and_then(|files| files.first().cloned())
        .map(|page| path.join(page).display().to_string())
        .unwrap_or_else(|| DEFAULT_COVER_URL.to_string())
}

// find first image from a directory
fn find_cover_from_dir(path: &PathBuf) -> String {
    path.read_dir()
        .ok()
        .map(sort_dir)
        .and_then(|dir| dir.into_iter().next())
        .map(|entry| entry.path().display().to_string())
        .unwrap_or_else(|| DEFAULT_COVER_URL.to_string())
}

fn sort_dir(dir: ReadDir) -> Vec<DirEntry> {
    sort_read_dir_with_reverse(dir, false)
}

fn sort_dir_reverse(dir: ReadDir) -> Vec<DirEntry> {
    sort_read_dir_with_reverse(dir, true)
}

fn sort_read_dir_with_reverse(dir: ReadDir, reverse: bool) -> Vec<DirEntry> {
    let mut dir: Vec<DirEntry> = dir.into_iter().filter_map(Result::ok).collect();
    dir.par_sort_by(|a, b| {
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
        return find_cover_from_archive(entry);
    }

    let entry_read_dir = match entry.read_dir().map(sort_dir_reverse) {
        Ok(entry_read_dir) => entry_read_dir,
        Err(_) => {
            return DEFAULT_COVER_URL.to_string();
        }
    };

    let path = match entry_read_dir
        .into_par_iter()
        .find_map_first(filter_supported_files_and_folders)
    {
        Some(entry) => entry.path(),
        None => {
            return DEFAULT_COVER_URL.to_string();
        }
    };

    if path.is_dir() {
        find_cover_from_dir(&path)
    } else if path.is_file() {
        find_cover_from_archive(&path)
    } else {
        DEFAULT_COVER_URL.to_string()
    }
}

fn get_pages_from_archive(
    path: &PathBuf,
    filename: String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    match libarchive_rs::list_archive_files(&filename) {
        Ok(files) => {
            let pages = files
                .into_iter()
                .map(|p| path.join(p).display().to_string())
                .collect();
            Ok(pages)
        }
        Err(e) => Err(e),
    }
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

fn map_entry_to_chapter(path: &PathBuf) -> Chapter {
    let modified = match path
        .metadata()
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
    {
        Some(modified) => modified.as_secs(),
        None => chrono::Local::now().timestamp() as u64,
    };

    let file_name = path
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .map(str::to_string)
        .unwrap_or_default();

    let number =
        match Regex::new(r"(?i)(?<=v)(\d+)|(?<=volume)\s*(\d+)|(?<=vol)\s*(\d+)|(?<=\s)(\d+)")
            .ok()
            .and_then(|re| re.find(&file_name).ok())
        {
            Some(Some(mat)) => mat.as_str().parse::<f64>().unwrap_or(0_f64),
            _ => 10000_f64,
        };

    Chapter {
        source_id: crate::local::ID,
        title: file_name,
        path: format!("{}", path.display()),
        number,
        scanlator: "".to_string(),
        uploaded: NaiveDateTime::from_timestamp(modified as i64, 0),
    }
}

pub fn get_manga_list(path: &PathBuf) -> Result<impl Iterator<Item = Manga>, anyhow::Error> {
    let data = std::fs::read_dir(path)?
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(filter_supported_files_and_folders)
        .map(|entry| Manga {
            source_id: crate::local::ID,
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
            cover_url: find_cover_url(&entry.path()),
        });

    Ok(data)
}

pub fn get_manga_info(path: String) -> Manga {
    let path = PathBuf::from(path);

    let title = path
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_string();

    Manga {
        source_id: crate::local::ID,
        title: title.clone(),
        author: vec![],
        genre: vec![],
        status: Some("".to_string()),
        description: Some(title),
        path: path.display().to_string(),
        cover_url: find_cover_url(&path),
    }
}

pub fn get_chapters(path: String) -> Result<impl Iterator<Item = Chapter>, anyhow::Error> {
    let path = PathBuf::from(path);
    if path.is_file() {
        let data = map_entry_to_chapter(&path);
        return Ok(vec![data].into_iter());
    }

    let data = std::fs::read_dir(&path)?
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| map_entry_to_chapter(&entry.path()));

    Ok(data)
}

pub fn get_pages(filename: String) -> Result<Vec<String>, anyhow::Error> {
    let path = PathBuf::from(filename.clone());
    let mut pages = if path.is_dir() {
        get_pages_from_dir(&path).map_err(|e| anyhow!("{}", e))?
    } else if path.is_file() {
        get_pages_from_archive(&path, filename).map_err(|e| anyhow!("{}", e))?
    } else {
        return Err(anyhow!("filename neither file or dir"));
    };

    pages.sort_by(|a, b| human_sort::compare(a, b));

    Ok(pages)
}
