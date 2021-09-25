use base64::{decode_config, encode_config, URL_SAFE_NO_PAD};
use dominator::routing;
use futures_signals::signal::{Signal, SignalExt};
use wasm_bindgen::prelude::*;
use web_sys::Url;

#[derive(Debug, Clone)]
pub enum SettingCategory {
    None,
    Appearance,
    General,
    Reader,
    Source(i64),
    Users,
    CreateUser,
    User,
}

#[derive(Debug)]
pub enum Route {
    Login,
    Library,
    Catalogue { id: i64, latest: bool },
    Manga(i64),
    MangaBySourcePath(i64, String),
    Chapter(i64, i64),
    Updates,
    Histories,
    Settings(SettingCategory),
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
                    [] => Route::Library,
                    ["updates"] => Route::Updates,
                    ["histories"] => Route::Histories,
                    ["catalogue"] => Route::Catalogue {
                        id: 0,
                        latest: false,
                    },
                    ["catalogue", id] => {
                        if let Ok(id) = id.parse() {
                            Route::Catalogue { id, latest: false }
                        } else {
                            Route::NotFound
                        }
                    }
                    ["catalogue", id, "latest"] => {
                        if let Ok(id) = id.parse() {
                            Route::Catalogue { id, latest: true }
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
                            if let Ok(decoded) = decode_config(path, URL_SAFE_NO_PAD) {
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
                        "general" => Route::Settings(SettingCategory::General),
                        "reader" => Route::Settings(SettingCategory::Reader),
                        "sources" => Route::Settings(SettingCategory::Source(0)),
                        "users" => Route::Settings(SettingCategory::Users),
                        "user" => Route::Settings(SettingCategory::User),
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
                    _ => Route::NotFound,
                }
            })
    }

    pub fn url(&self) -> String {
        match self {
            Route::Login => "/login".to_string(),
            Route::Library => "/".to_string(),
            Route::Catalogue { id, latest } => {
                if *id > 0 && *latest {
                    [
                        "/catalogue".to_string(),
                        id.to_string(),
                        "latest".to_string(),
                    ]
                    .join("/")
                } else if *id > 0 && !*latest {
                    ["/catalogue".to_string(), id.to_string()].join("/")
                } else {
                    "/catalogue".to_string()
                }
            }
            Route::Manga(manga_id) => ["/manga".to_string(), manga_id.to_string()].join("/"),
            Route::MangaBySourcePath(source_id, path) => [
                "/manga".to_string(),
                source_id.to_string(),
                encode_config(path, URL_SAFE_NO_PAD),
            ]
            .join("/"),
            Route::Chapter(chapter_id, page) => {
                format!("/chapter/{}#{}", chapter_id, page + 1)
            }
            Route::Updates => "/updates".to_string(),
            Route::Histories => "/histories".to_string(),
            Route::Settings(SettingCategory::None) => "/settings".to_string(),
            Route::Settings(SettingCategory::Appearance) => "/settings/appearance".to_string(),
            Route::Settings(SettingCategory::General) => "/settings/general".to_string(),
            Route::Settings(SettingCategory::Reader) => "/settings/reader".to_string(),
            Route::Settings(SettingCategory::Source(source_id)) => {
                if *source_id > 0 {
                    format!("/settings/sources/{}", source_id)
                } else {
                    "/settings/sources".to_string()
                }
            }
            Route::Settings(SettingCategory::Users) => "/settings/users".to_string(),
            Route::Settings(SettingCategory::CreateUser) => "/settings/users/create".to_string(),
            Route::Settings(SettingCategory::User) => "/settings/user".to_string(),
            Route::NotFound => "/notfound".to_string(),
        }
    }
}
