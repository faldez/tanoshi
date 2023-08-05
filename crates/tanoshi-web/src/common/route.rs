use base64::{engine::general_purpose, Engine};
use dominator::routing;
use futures_signals::signal::{Signal, SignalExt};
use wasm_bindgen::prelude::*;
use web_sys::Url;

#[derive(Debug, Clone, Copy)]
pub enum SettingCategory {
    None,
    Appearance,
    Chapters,
    Reader,
    Library,
    Category,
    SourceList,
    Source(i64),
    Users,
    CreateUser,
    User,
    DownloadQueue,
}

#[derive(Debug, Clone)]
pub enum Route {
    Root,
    Login,
    LibraryList,
    Library(Option<i64>),
    CatalogueList,
    Catalogue {
        id: i64,
        latest: bool,
        query: Option<String>,
    },
    Manga(i64),
    MangaBySourcePath(i64, String),
    Chapter(i64, i64),
    Updates,
    Histories,
    Settings(SettingCategory),
    TrackerLogin(String),
    TrackerRedirect {
        tracker: String,
        code: String,
        state: Option<String>,
    },
    NotFound,
}

impl Route {
    pub fn signal() -> impl Signal<Item = Self> {
        routing::url()
            .signal_ref(|url| Url::new(url).unwrap_throw())
            .map(|url| {
                let pathname = url.pathname();
                let mut paths = pathname.split('/').collect::<Vec<_>>();
                paths.retain(|path| !path.is_empty());

                match paths.as_slice() {
                    ["login"] => Route::Login,
                    [] => Route::Root,
                    ["libraries"] => Route::LibraryList,
                    ["library"] => Route::Library(None),
                    ["library", category_id] => Route::Library(category_id.parse().ok()),
                    ["updates"] => Route::Updates,
                    ["histories"] => Route::Histories,
                    ["catalogue"] => Route::CatalogueList,
                    ["catalogue", id] => {
                        if let Ok(id) = id.parse() {
                            let params = url.search_params();
                            let query = params.get("keyword");
                            Route::Catalogue {
                                id,
                                latest: false,
                                query,
                            }
                        } else {
                            Route::NotFound
                        }
                    }
                    ["catalogue", id, "latest"] => {
                        if let Ok(id) = id.parse() {
                            let params = url.search_params();
                            let query = params.get("keyword");
                            Route::Catalogue {
                                id,
                                latest: true,
                                query,
                            }
                        } else {
                            Route::NotFound
                        }
                    }
                    ["manga", id] => {
                        if let Ok(id) = id.parse() {
                            Route::Manga(id)
                        } else {
                            Route::NotFound
                        }
                    }
                    ["manga", source_id, path] => {
                        if let Ok(id) = source_id.parse() {
                            if let Ok(decoded) = general_purpose::URL_SAFE_NO_PAD.decode(path) {
                                if let Ok(path) = String::from_utf8(decoded) {
                                    Route::MangaBySourcePath(id, path)
                                } else {
                                    Route::NotFound
                                }
                            } else {
                                Route::NotFound
                            }
                        } else {
                            Route::NotFound
                        }
                    }
                    ["chapter", id] => {
                        if let Ok(id) = id.parse() {
                            let page = url
                                .hash()
                                .strip_prefix('#')
                                .and_then(|h| h.parse().ok())
                                .unwrap_or(1);
                            info!("chapter {} page {}", id, page);
                            Route::Chapter(id, page - 1)
                        } else {
                            Route::NotFound
                        }
                    }
                    ["settings"] => Route::Settings(SettingCategory::None),
                    ["settings", cat] => match *cat {
                        "appearance" => Route::Settings(SettingCategory::Appearance),
                        "chapters" => Route::Settings(SettingCategory::Chapters),
                        "library" => Route::Settings(SettingCategory::Library),
                        "category" => Route::Settings(SettingCategory::Category),
                        "reader" => Route::Settings(SettingCategory::Reader),
                        "sources" => Route::Settings(SettingCategory::SourceList),
                        "users" => Route::Settings(SettingCategory::Users),
                        "user" => Route::Settings(SettingCategory::User),
                        "downloads-queue" => Route::Settings(SettingCategory::DownloadQueue),
                        _ => Route::NotFound,
                    },
                    ["settings", "users", "create"] => Route::Settings(SettingCategory::CreateUser),
                    ["settings", "sources", id] => {
                        if let Ok(id) = id.parse() {
                            Route::Settings(SettingCategory::Source(id))
                        } else {
                            Route::NotFound
                        }
                    }
                    ["tracker", tracker, "login"] => Route::TrackerLogin(tracker.to_string()),
                    ["tracker", tracker, "redirect"] => {
                        let params = url.search_params();
                        let code = params.get("code");
                        let state = params.get("state");
                        match *tracker {
                            "myanimelist" if code.is_some() && state.is_some() => {
                                Route::TrackerRedirect {
                                    tracker: tracker.to_string(),
                                    code: code.unwrap(),
                                    state,
                                }
                            }
                            "anilist" if code.is_some() && state.is_some() => {
                                Route::TrackerRedirect {
                                    tracker: tracker.to_string(),
                                    code: code.unwrap(),
                                    state,
                                }
                            }
                            _ => Route::NotFound,
                        }
                    }
                    _ => Route::NotFound,
                }
            })
    }

    pub fn url(&self) -> String {
        match self {
            Route::Root => "/".to_string(),
            Route::Login => "/login".to_string(),
            Route::LibraryList => "/libraries".to_string(),
            Route::Library(category_id) => {
                if let Some(id) = category_id {
                    format!("/library/{}", id)
                } else {
                    "/library".to_string()
                }
            }
            Route::CatalogueList => "/catalogue".to_string(),
            Route::Catalogue { id, latest, query } => {
                if *latest {
                    return format!("/catalogue/{}/latest", id);
                }

                let mut param = vec![];
                if let Some(query) = query {
                    param.push(format!("query={}", query));
                }

                format!("/catalogue/{}?{}", id, param.join("&"))
            }
            Route::Manga(manga_id) => ["/manga".to_string(), manga_id.to_string()].join("/"),
            Route::MangaBySourcePath(source_id, path) => [
                "/manga".to_string(),
                source_id.to_string(),
                general_purpose::URL_SAFE_NO_PAD.encode(path),
            ]
            .join("/"),
            Route::Chapter(chapter_id, page) => {
                format!("/chapter/{}#{}", chapter_id, page + 1)
            }
            Route::Updates => "/updates".to_string(),
            Route::Histories => "/histories".to_string(),
            Route::Settings(SettingCategory::None) => "/settings".to_string(),
            Route::Settings(SettingCategory::Appearance) => "/settings/appearance".to_string(),
            Route::Settings(SettingCategory::Chapters) => "/settings/chapters".to_string(),
            Route::Settings(SettingCategory::Library) => "/settings/library".to_string(),
            Route::Settings(SettingCategory::Category) => "/settings/category".to_string(),
            Route::Settings(SettingCategory::Reader) => "/settings/reader".to_string(),
            Route::Settings(SettingCategory::SourceList) => "/settings/sources".to_string(),
            Route::Settings(SettingCategory::Source(source_id)) => {
                format!("/settings/sources/{}", source_id)
            }
            Route::Settings(SettingCategory::Users) => "/settings/users".to_string(),
            Route::Settings(SettingCategory::CreateUser) => "/settings/users/create".to_string(),
            Route::Settings(SettingCategory::User) => "/settings/user".to_string(),
            Route::Settings(SettingCategory::DownloadQueue) => {
                "/settings/downloads-queue".to_string()
            }
            Route::TrackerLogin(tracker) => format!("/tracker/{tracker}/login"),
            Route::TrackerRedirect {
                tracker,
                code,
                state: Some(state),
            } => format!("/tracker/{tracker}/redirect?code={code}&state={state}"),
            Route::TrackerRedirect {
                tracker,
                code,
                state: None,
            } => format!("/tracker/{tracker}/redirect?code={code}"),
            Route::NotFound => "/notfound".to_string(),
        }
    }
}
