use std::{
    ffi::OsStr,
    fs::DirEntry,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use chrono::NaiveDateTime;
use fancy_regex::Regex;
use phf::phf_set;
use tanoshi_lib::prelude::{Chapter, Extension, ExtensionResult, Filters, Manga, Source};

pub static ID: i64 = 1;
// list of supported files, other archive may works but no tested
static SUPPORTED_FILES: phf::Set<&'static str> = phf_set! {
    "cbz",
    "cbr",
};

pub struct Local {
    path: PathBuf,
}

impl Local {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = PathBuf::new().join(path);
        Self { path }
    }

    fn default_cover_url() -> String {
        "/images/cover-placeholder.jpg".to_string()
    }

    fn filter_supported_files_and_folders(
        entry: Result<DirEntry, std::io::Error>,
    ) -> Option<DirEntry> {
        let entry = entry.ok()?;
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
            .unwrap_or_else(Self::default_cover_url)
    }

    // find first image from a directory
    fn find_cover_from_dir(path: &PathBuf) -> String {
        path.read_dir()
            .ok()
            .and_then(|dir| dir.into_iter().find_map(Result::ok))
            .map(|entry| entry.path().display().to_string())
            .unwrap_or_else(|| Self::default_cover_url())
    }

    fn find_cover_url(entry: &PathBuf) -> String {
        if entry.is_file() {
            return Self::find_cover_from_archive(entry);
        }

        let entry_read_dir = match entry.read_dir() {
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
            source_id: ID,
            title: file_name,
            path: format!("{}", path.display()),
            number,
            scanlator: "".to_string(),
            uploaded: NaiveDateTime::from_timestamp(modified as i64, 0),
        })
    }
}

impl Extension for Local {
    fn detail(&self) -> Source {
        Source {
            id: ID,
            name: "local".to_string(),
            url: format!("{}", self.path.display()),
            version: "1.0.0".to_string(),
            icon: "/icons/192.png".to_string(),
            need_login: false,
            languages: vec![],
        }
    }

    fn filters(&self) -> ExtensionResult<Option<Filters>> {
        ExtensionResult::ok(None)
    }

    fn get_manga_list(&self, _param: tanoshi_lib::prelude::Param) -> ExtensionResult<Vec<Manga>> {
        let read_dir = match std::fs::read_dir(&self.path) {
            Ok(read_dir) => read_dir,
            Err(e) => {
                return ExtensionResult::err(format!("{}", e).as_str());
            }
        };

        let mut files = read_dir
            .into_iter()
            .filter_map(Self::filter_supported_files_and_folders)
            .collect::<Vec<_>>();
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        let data = files
            .into_iter()
            .map(|entry| Manga {
                source_id: ID,
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
            })
            .collect();

        ExtensionResult::ok(data)
    }

    fn get_manga_info(&self, path: String) -> ExtensionResult<Manga> {
        let path = PathBuf::from(path);

        let title = path
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("")
            .to_string();
        ExtensionResult::ok(Manga {
            source_id: ID,
            title: title.clone(),
            author: vec![],
            genre: vec![],
            status: Some("".to_string()),
            description: Some(title),
            path: path.display().to_string(),
            cover_url: Self::find_cover_url(&path),
        })
    }

    fn get_chapters(&self, path: String) -> ExtensionResult<Vec<Chapter>> {
        let path = PathBuf::from(path);
        if path.is_file() {
            if let Some(data) = Self::map_entry_to_chapter(&path) {
                return ExtensionResult::ok(vec![data]);
            }
        }

        let read_dir = match std::fs::read_dir(&path) {
            Ok(read_dir) => read_dir,
            Err(e) => {
                return ExtensionResult::err(format!("{}", e).as_str());
            }
        };

        let data = read_dir
            .into_iter()
            .filter_map(Result::ok)
            .filter_map(|entry| Self::map_entry_to_chapter(&entry.path()))
            .collect();

        ExtensionResult::ok(data)
    }

    fn get_pages(&self, filename: String) -> ExtensionResult<Vec<String>> {
        let path = PathBuf::from(filename.clone());
        if path.is_dir() {
            match Self::get_pages_from_dir(&path) {
                Ok(pages) => ExtensionResult::ok(pages),
                Err(e) => ExtensionResult::err(format!("{}", e).as_str()),
            }
        } else if path.is_file() {
            match Self::get_pages_from_archive(&path, filename) {
                Ok(pages) => ExtensionResult::ok(pages),
                Err(e) => ExtensionResult::err(format!("{}", e).as_str()),
            }
        } else {
            ExtensionResult::err("filename neither file or dir")
        }
    }
}

#[cfg(test)]
mod test {
    use tanoshi_lib::prelude::Param;

    use super::*;

    #[test]
    fn test_positive_get_manga_list() {
        let local = Local::new("./test/data/manga");
        let manga = local.get_manga_list(Param::default());

        assert!(manga.data.is_some());
        assert!(manga.error.is_none());

        if let Some(data) = manga.data {
            assert_eq!(data.len(), 3);

            assert_eq!(data[0].title, "Space Adventures");
            #[cfg(target_family = "windows")]
            assert_eq!(
                data[0].cover_url,
                "./test/data/manga\\Space Adventures\\Space_Adventures_001__c2c__diff_ver.cbz\\SPA00401.JPG"
            );
            #[cfg(target_family = "unix")]
            assert_eq!(
                data[0].cover_url,
                "./test/data/manga/Space Adventures/Space_Adventures_001__c2c__diff_ver.cbz/SPA00401.JPG"
            );

            assert_eq!(data[1].title, "Space_Adventures_004__c2c__diff_ver");
            #[cfg(target_family = "windows")]
            assert_eq!(
                data[1].cover_url,
                "./test/data/manga\\Space_Adventures_004__c2c__diff_ver.cbz\\SPA00401.JPG"
            );
            #[cfg(target_family = "unix")]
            assert_eq!(
                data[1].cover_url,
                "./test/data/manga/Space_Adventures_004__c2c__diff_ver.cbz/SPA00401.JPG"
            );

            assert_eq!(data[2].title, "Super Duck");
            #[cfg(target_family = "windows")]
            assert_eq!(
                data[2].cover_url,
                "./test/data/manga\\Super Duck\\super_duck_1\\duck00.jpg"
            );
            #[cfg(target_family = "unix")]
            assert_eq!(
                data[2].cover_url,
                "./test/data/manga/Super Duck/super_duck_1/duck00.jpg"
            );
        }
    }

    #[test]
    fn test_negative_get_manga_list() {
        let local = Local::new("./test/data/not_manga");
        let manga = local.get_manga_list(Param::default());

        assert!(manga.data.is_none());
        assert!(manga.error.is_some());
    }

    #[test]
    fn test_get_manga_info() {
        let local = Local::new("./test/data/manga");
        #[cfg(target_family = "windows")]
        let manga = local.get_manga_info(
            "./test/data/manga\\Space_Adventures_004__c2c__diff_ver.cbz".to_string(),
        );
        #[cfg(target_family = "unix")]
        let manga = local.get_manga_info(
            "./test/data/manga/Space_Adventures_004__c2c__diff_ver.cbz".to_string(),
        );

        assert!(manga.data.is_some());
        assert!(manga.error.is_none());

        if let Some(data) = manga.data {
            assert_eq!(data.source_id, 1);
            assert_eq!(data.title, "Space_Adventures_004__c2c__diff_ver");
            #[cfg(target_family = "windows")]
            assert_eq!(
                data.cover_url,
                "./test/data/manga\\Space_Adventures_004__c2c__diff_ver.cbz\\SPA00401.JPG"
            );
            #[cfg(target_family = "unix")]
            assert_eq!(
                data.cover_url,
                "./test/data/manga/Space_Adventures_004__c2c__diff_ver.cbz/SPA00401.JPG"
            );
        }
    }

    #[test]
    fn test_single_chapter_manga_get_chapters() {
        let local = Local::new("./test/data/manga");
        #[cfg(target_family = "windows")]
        let chapter = local
            .get_chapters("./test/data/manga\\Space_Adventures_004__c2c__diff_ver.cbz".to_string());
        #[cfg(target_family = "unix")]
        let manga = local.get_manga_info(
            "./test/data/manga/Space_Adventures_004__c2c__diff_ver.cbz".to_string(),
        );

        assert!(chapter.data.is_some());
        assert!(chapter.error.is_none());

        if let Some(data) = chapter.data {
            assert_eq!(data.len(), 1);

            assert_eq!(data[0].source_id, 1);
            assert_eq!(data[0].title, "Space_Adventures_004__c2c__diff_ver");
            #[cfg(target_family = "windows")]
            assert_eq!(
                data[0].path,
                "./test/data/manga\\Space_Adventures_004__c2c__diff_ver.cbz"
            );
            #[cfg(target_family = "unix")]
            assert_eq!(
                data[0].cover_url,
                "./test/data/manga/Space_Adventures_004__c2c__diff_ver.cbz"
            );
        }
    }

    #[test]
    fn test_manga_get_chapters() {
        let local = Local::new("./test/data/manga");
        #[cfg(target_family = "windows")]
        let chapter = local.get_chapters("./test/data/manga\\Space Adventures".to_string());
        #[cfg(target_family = "unix")]
        let manga = local.get_manga_info("./test/data/manga/Space Adventures".to_string());

        assert!(chapter.data.is_some());
        assert!(chapter.error.is_none());

        if let Some(data) = chapter.data {
            assert_eq!(data.len(), 2);

            assert_eq!(data[0].source_id, 1);
            assert_eq!(data[0].title, "Space_Adventures_001__c2c__diff_ver");
            #[cfg(target_family = "windows")]
            assert_eq!(
                data[0].path,
                "./test/data/manga\\Space Adventures\\Space_Adventures_001__c2c__diff_ver.cbz"
            );
            #[cfg(target_family = "unix")]
            assert_eq!(
                data[1].cover_url,
                "./test/data/manga/Space Adventures/Space_Adventures_001__c2c__diff_ver.cbz"
            );

            assert_eq!(data[1].source_id, 1);
            assert_eq!(data[1].title, "Space_Adventures_004__c2c__diff_ver");
            #[cfg(target_family = "windows")]
            assert_eq!(
                data[1].path,
                "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver"
            );
            #[cfg(target_family = "unix")]
            assert_eq!(
                data[1].cover_url,
                "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver"
            );
        }
    }

    #[test]
    fn test_archive_get_pages() {
        let local = Local::new("./test/data/manga");
        #[cfg(target_family = "windows")]
        let pages = local.get_pages(
            "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver".to_string(),
        );
        #[cfg(target_family = "unix")]
        let pages = local.get_manga_info(
            "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver".to_string(),
        );

        assert!(pages.data.is_some());
        assert!(pages.error.is_none());

        if let Some(data) = pages.data {
            assert_eq!(data.len(), 36);

            #[cfg(target_family = "windows")]
            {
                assert_eq!(data[0], "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00401.JPG");
                assert_eq!(data[1], "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00402.JPG");
                assert_eq!(data[2], "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00403.JPG");
                assert_eq!(data[3], "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00404.JPG");
                assert_eq!(data[4], "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00405.JPG");
                assert_eq!(data[5], "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00406.JPG");
                assert_eq!(data[6], "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00407.JPG");
                assert_eq!(data[7], "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00408.JPG");
                assert_eq!(data[8], "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00409.JPG");
                assert_eq!(data[9], "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00410.JPG");
                assert_eq!(
                    data[10],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00411.JPG"
                );
                assert_eq!(
                    data[11],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00412.JPG"
                );
                assert_eq!(
                    data[12],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00413.JPG"
                );
                assert_eq!(
                    data[13],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00414.JPG"
                );
                assert_eq!(
                    data[14],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00415.JPG"
                );
                assert_eq!(
                    data[15],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00416.JPG"
                );
                assert_eq!(
                    data[16],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00417.JPG"
                );
                assert_eq!(
                    data[17],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00418.JPG"
                );
                assert_eq!(
                    data[18],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00419.JPG"
                );
                assert_eq!(
                    data[19],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00420.JPG"
                );
                assert_eq!(
                    data[20],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00421.JPG"
                );
                assert_eq!(
                    data[21],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00422.JPG"
                );
                assert_eq!(
                    data[22],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00423.JPG"
                );
                assert_eq!(
                    data[23],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00424.JPG"
                );
                assert_eq!(
                    data[24],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00425.JPG"
                );
                assert_eq!(
                    data[25],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00426.JPG"
                );
                assert_eq!(
                    data[26],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00427.JPG"
                );
                assert_eq!(
                    data[27],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00428.JPG"
                );
                assert_eq!(
                    data[28],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00429.JPG"
                );
                assert_eq!(
                    data[29],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00430.JPG"
                );
                assert_eq!(
                    data[30],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00431.JPG"
                );
                assert_eq!(
                    data[31],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00432.JPG"
                );
                assert_eq!(
                    data[32],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00433.JPG"
                );
                assert_eq!(
                    data[33],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00434.JPG"
                );
                assert_eq!(
                    data[34],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00435.JPG"
                );
                assert_eq!(
                    data[35],
                    "./test/data/manga\\Space Adventures\\Space_Adventures_004__c2c__diff_ver\\SPA00436.JPG"
                );
            }

            #[cfg(target_family = "unix")]
            {
                assert_eq!(data[0], "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00401.JPG");
                assert_eq!(data[1], "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00402.JPG");
                assert_eq!(data[2], "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00403.JPG");
                assert_eq!(data[3], "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00404.JPG");
                assert_eq!(data[4], "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00405.JPG");
                assert_eq!(data[5], "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00406.JPG");
                assert_eq!(data[6], "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00407.JPG");
                assert_eq!(data[7], "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00408.JPG");
                assert_eq!(data[8], "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00409.JPG");
                assert_eq!(data[9], "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00410.JPG");
                assert_eq!(
                    data[10],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00411.JPG"
                );
                assert_eq!(
                    data[11],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00412.JPG"
                );
                assert_eq!(
                    data[12],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00413.JPG"
                );
                assert_eq!(
                    data[13],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00414.JPG"
                );
                assert_eq!(
                    data[14],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00415.JPG"
                );
                assert_eq!(
                    data[15],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00416.JPG"
                );
                assert_eq!(
                    data[16],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00417.JPG"
                );
                assert_eq!(
                    data[17],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00418.JPG"
                );
                assert_eq!(
                    data[18],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00419.JPG"
                );
                assert_eq!(
                    data[19],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00420.JPG"
                );
                assert_eq!(
                    data[20],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00421.JPG"
                );
                assert_eq!(
                    data[21],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00422.JPG"
                );
                assert_eq!(
                    data[22],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00423.JPG"
                );
                assert_eq!(
                    data[23],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00424.JPG"
                );
                assert_eq!(
                    data[24],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00425.JPG"
                );
                assert_eq!(
                    data[25],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00426.JPG"
                );
                assert_eq!(
                    data[26],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00427.JPG"
                );
                assert_eq!(
                    data[27],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00428.JPG"
                );
                assert_eq!(
                    data[28],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00429.JPG"
                );
                assert_eq!(
                    data[29],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00430.JPG"
                );
                assert_eq!(
                    data[30],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00431.JPG"
                );
                assert_eq!(
                    data[31],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00432.JPG"
                );
                assert_eq!(
                    data[32],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00433.JPG"
                );
                assert_eq!(
                    data[33],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00434.JPG"
                );
                assert_eq!(
                    data[34],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00435.JPG"
                );
                assert_eq!(
                    data[35],
                    "./test/data/manga/Space Adventures/Space_Adventures_004__c2c__diff_ver/SPA00436.JPG"
                );
            }
        }
    }
}
