#![crate_name = "tanoshi_lib"]

/// This is used to ensure both application and extension use the same version
pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Rust doesn't have stable ABI, this is used to ensure `rustc` version is match
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

/// This module contains model used in extensions and rest api
#[cfg(feature = "model")]
pub mod manga {
    use chrono::Local;
    use serde::{Deserialize, Serialize};

    /// Model to login to source that require login, like mangadex to search
    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
    pub struct SourceLogin {
        pub username: String,
        pub password: String,
        pub remember_me: Option<bool>,
        pub two_factor: Option<String>,
    }

    /// Result of source login
    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
    pub struct SourceLoginResult {
        pub source_id: i32,
        pub source_name: String,
        pub auth_type: String,
        pub value: String,
    }

    /// A type represent source
    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
    pub struct Source {
        pub id: i32,
        pub name: String,
        pub url: String,
        pub version: String,
    }

    /// A type represent manga details, normalized across source
    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
    pub struct Manga {
        pub id: i32,
        pub title: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub author: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub genre: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        pub path: String,
        pub thumbnail_url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub last_read: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub last_page: Option<i32>,
        pub is_favorite: bool,
    }

    /// A type represent chapter, normalized across source
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Chapter {
        pub id: i32,
        pub manga_id: i32,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub vol: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub no: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub title: Option<String>,
        pub url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub read: Option<i32>,
        pub uploaded: chrono::NaiveDateTime,
    }

    impl Default for Chapter {
        fn default() -> Self {
            Chapter {
                id: 0,
                manga_id: 0,
                vol: None,
                no: None,
                title: None,
                url: "".to_string(),
                read: None,
                uploaded: chrono::NaiveDateTime::from_timestamp(0, 0),
            }
        }
    }

    /// A type represent a page
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Image {
        pub source_id: i32,
        pub source_name: String,
        pub path: String,
        pub file_name: String,
        pub url: String,
    }

    /// A type represent sort parameter for query manga from source, normalized across source
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum SortByParam {
        LastUpdated,
        Title,
        Comment,
        Views,
    }

    /// A type represent order parameter for query manga from source, normalized across source
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum SortOrderParam {
        Asc,
        Desc,
    }

    /// A type represent parameter for query manga from source, normalized across source
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Params {
        pub keyword: Option<String>,
        pub genres: Option<Vec<String>>,
        pub page: Option<String>,
        pub sort_by: Option<SortByParam>,
        pub sort_order: Option<SortOrderParam>,
        pub refresh: Option<bool>,
    }

    /// A type represent parameter for query manga from source, normalized across source
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct GetParams {
        pub refresh: Option<bool>,
    }

    /// A type represent history
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct History {
        pub manga_id: i32,
        pub title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub thumbnail_url: Option<String>,
        pub chapter: String,
        pub chapter_id: i32,
        pub read: i32,
        pub at: chrono::NaiveDateTime,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub days: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub show_sep: Option<bool>,
    }

    /// A type represent chapter updates
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Update {
        pub manga_id: i32,
        pub title: String,
        pub thumbnail_url: String,
        pub number: String,
        pub chapter_id: i32,
        pub uploaded: chrono::NaiveDateTime,
        pub days: Option<i64>,
        pub show_sep: Option<bool>,
    }

    impl Default for Update {
        fn default() -> Self {
            Update {
                manga_id: 0,
                title: "".to_string(),
                thumbnail_url: "".to_string(),
                number: "".to_string(),
                chapter_id: 0,
                uploaded: Local::now().naive_local(),
                days: None,
                show_sep: None,
            }
        }
    }

    /// A type represent favorite manga
    #[derive(Debug, Serialize, Deserialize, Clone, Default)]
    pub struct FavoriteManga {
        pub manga_id: i32,
        pub title: String,
        pub path: String,
        pub thumbnail_url: String,
    }
}

/// This module contains request and response payload for rest api
#[cfg(feature = "rest")]
pub mod rest {
    use crate::manga::{Chapter, FavoriteManga, History, Manga, Source, Update};
    use serde::{Deserialize, Serialize};

    /// Reponse for get sources request
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct GetSourceResponse {
        pub sources: Vec<Source>,
        pub status: String,
    }

    /// Response for get list of manga request
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct GetMangasResponse {
        pub mangas: Vec<Manga>,
        pub status: String,
    }

    /// Response for get manga detail request
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct GetMangaResponse {
        pub manga: Manga,
        pub status: String,
    }

    /// Response for get chapters of a manga
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct GetChaptersResponse {
        pub chapters: Vec<Chapter>,
        pub status: String,
    }

    /// Reponse for get pages of a chapter
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct GetPagesResponse {
        pub manga_id: i32,
        pub pages: Vec<String>,
        pub status: String,
    }

    /// Parameter for history request
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct HistoryRequest {
        pub chapter_id: i32,
        pub read: i32,
        pub at: chrono::NaiveDateTime,
    }

    /// Response for history request
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct HistoryResponse {
        pub history: Vec<History>,
        pub status: String,
    }

    /// Response for chapter updates request
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct UpdatesResponse {
        pub updates: Vec<Update>,
        pub status: String,
    }

    /// Response for get favorites request
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct GetFavoritesResponse {
        pub favorites: Option<Vec<FavoriteManga>>,
        pub status: String,
    }

    /// Response for add new favorite request
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct AddFavoritesResponse {
        pub status: String,
    }
}

/// This module contains `Extension` trait, and function for interacting with `Extension`
#[cfg(feature = "extensions")]
pub mod extensions {
    use crate::manga::{Chapter, Manga, Params, Source, SourceLogin, SourceLoginResult};
    use anyhow::{anyhow, Result};
    use serde_yaml;
    use std::io::Read;

    /// `Extension` trait is an implementation for building extensions
    pub trait Extension: Send + Sync {
        /// Returns the information of the source
        fn info(&self) -> Source;

        /// Returns list of manga from the source
        ///
        /// # Arguments
        ///
        /// * `url` - An url to specified page in source that can be parsed into a list of mangas
        /// * `param` - Parameter to filter manga from source
        fn get_mangas(&self, url: &String, param: Params, auth: String) -> Result<Vec<Manga>>;

        /// Returns detail of manga
        fn get_manga_info(&self, url: &String) -> Result<Manga>;

        /// Returns list of chapters of a manga
        fn get_chapters(&self, url: &String) -> Result<Vec<Chapter>>;

        /// Returns list of pages from a chapter of a manga
        fn get_pages(&self, url: &String) -> Result<Vec<String>>;

        /// Returns an image by download to disk first then serve to web
        fn get_page(&self, url: &String) -> Result<Vec<u8>> {
            let bytes = {
                let resp = ureq::get(&url).call();
                let mut reader = resp.into_reader();
                let mut bytes = vec![];
                if reader.read_to_end(&mut bytes).is_err() {
                    return Err(anyhow!("error read image"));
                }
                bytes
            };

            Ok(bytes)
        }

        /// Login to source
        fn login(&self, _: SourceLogin) -> Result<SourceLoginResult> {
            Err(anyhow!("not implemented"))
        }
    }

    /// A type represents an extension
    pub struct PluginDeclaration {
        pub rustc_version: &'static str,
        pub core_version: &'static str,
        pub register: unsafe extern "C" fn(&mut dyn PluginRegistrar, Option<&serde_yaml::Value>),
    }

    /// A trait for register an extension
    pub trait PluginRegistrar {
        fn register_function(&mut self, name: &str, extension: Box<dyn Extension>);
    }

    /// macro for export an extension
    #[macro_export]
    macro_rules! export_plugin {
        ($register:expr) => {
            #[doc(hidden)]
            #[no_mangle]
            pub static plugin_declaration: $crate::extensions::PluginDeclaration =
                $crate::extensions::PluginDeclaration {
                    rustc_version: $crate::RUSTC_VERSION,
                    core_version: $crate::CORE_VERSION,
                    register: $register,
                };
        };
    }
}
