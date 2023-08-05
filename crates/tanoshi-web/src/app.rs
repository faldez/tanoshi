use std::rc::Rc;

use dominator::{clone, html, routing, Dom};
use futures_signals::{
    map_ref,
    signal::{Mutable, Signal, SignalExt},
};
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_futures::spawn_local;

use crate::{
    catalogue::Catalogue,
    catalogue_list::CatalogueList,
    common::{snackbar, Bottombar, LibrarySettings, Route, ServerStatus, SettingCategory, Spinner},
    histories::Histories,
    library::Library,
    library_list::LibraryList,
    login::Login,
    manga::Manga,
    query,
    reader::Reader,
    settings::Settings,
    tracker_login::TrackerLogin,
    tracker_redirect::TrackerRedirect,
    updates::Updates,
    utils::{local_storage, AsyncLoader},
};

pub struct App {
    pub server_status: Mutable<Option<ServerStatus>>,
    pub spinner: Rc<Spinner>,
    pub loader: AsyncLoader,
}

impl App {
    pub fn new() -> Rc<Self> {
        Rc::new(App {
            server_status: Mutable::new(None),
            spinner: Spinner::new(),
            loader: AsyncLoader::new(),
        })
    }

    pub fn fetch_server_status(app: Rc<Self>) {
        app.loader.load(clone!(app => async move {
            match query::fetch_server_status().await {
                Ok(server_status) => {
                    app.server_status.set_neq(Some(ServerStatus {
                        activated: server_status.activated,
                        version: server_status.version,
                        loggedin: server_status.loggedin
                    }));
                }
                Err(e) => {
                    snackbar::show(format!("error check server status: {}", e));
                }
            }
        }));
    }

    pub fn signal(&self) -> impl Signal<Item = (Route, Option<ServerStatus>)> {
        map_ref! {
            let route = Route::signal(),
            let server_status = self.server_status.signal_cloned() =>

            (route.clone(), server_status.clone())
        }
    }

    pub fn render(app: Rc<Self>) -> Dom {
        Self::fetch_server_status(app.clone());

        html!("div", {
            .future(app.server_status.signal_cloned().for_each(|server_status| {
                info!("server_status: {server_status:?}");
                if let Some(server_status) = server_status {
                    if !server_status.activated {
                        info!("server inactivated, go to login");
                        local_storage().delete("token").unwrap_throw();
                        routing::go_to_url(&Route::Login.url());
                    } else if server_status.activated && !server_status.loggedin {
                        routing::go_to_url(&Route::Login.url());
                    } else if server_status.loggedin {
                        spawn_local(async {
                            loop {
                                info!("subscribing recent updates");
                                let _ = query::subscribe_recent_updates().await;

                                info!("reconnecting in 30s..");
                                TimeoutFuture::new(30_000).await;
                            }
                        });
                    }
                }

                async move {}
            }))
            .child_signal(app.signal().map(clone!(app => move |(route, server_status)| {
                if server_status.is_none() {
                    return None;
                }

                match route {
                    Route::Root => {
                        if let Some(default_category) = LibrarySettings::load(false, false).default_category.get_cloned() {
                            routing::go_to_url(&Route::Library(default_category.id).url());
                        } else {
                            routing::go_to_url(&Route::LibraryList.url());
                        }

                        None
                    }
                    Route::Login => Some(
                        Login::render(Login::new(), app.clone())
                    ),
                    Route::LibraryList => {
                        Some(LibraryList::render(LibraryList::new()))
                    }
                    Route::Library(category_id) => Some(
                        Library::render(Library::new(category_id)),
                    ),
                    Route::CatalogueList => Some(
                        CatalogueList::render(CatalogueList::new()),
                    ),
                    Route::Catalogue{id, latest, query} => Some(
                        Catalogue::render(Catalogue::new(id), latest, query),
                    ),
                    Route::Manga(manga_id) => Some(
                        Manga::render(Manga::new(manga_id, 0, "".to_string())),
                    ),
                    Route::MangaBySourcePath(source_id, path) => Some(
                        Manga::render(Manga::new(0, source_id, path)),
                    ),
                    Route::Chapter(chapter_id, page) => Some(
                        Reader::render(Reader::new(chapter_id, page)),
                    ),
                    Route::Updates => Some(
                        Updates::render(Updates::new(), app.clone()),
                    ),
                    Route::Histories => Some(
                        Histories::render(Histories::new(), app.clone()),
                    ),
                    Route::Settings(category) => {
                        let server_version = app.server_status.get_cloned().unwrap_or_default().version;
                        Some(Settings::new(server_version, category).render())
                    }
                    Route::TrackerLogin(tracker) => {
                        Some(TrackerLogin::render(TrackerLogin::new(tracker)))
                    }
                    Route::TrackerRedirect{tracker, code, state} => {
                        Some(TrackerRedirect::render(TrackerRedirect::new(tracker, code, state)))
                    }
                    Route::NotFound => Some(
                        html!("div", {
                            .text("not found")
                        }),
                    )
                }
            })))
            .child_signal(Route::signal().map(|x| {
                match x {
                    Route::LibraryList | Route::Library(_) | Route::CatalogueList | Route::Updates | Route::Histories | Route::Settings(SettingCategory::None) => Some(html!("div", {
                        .children(&mut [
                            html!("div", {
                                .class("bottombar-spacing")
                            }),
                            Bottombar::render(),
                        ])
                    })),
                    _ => None,
                }
            }))
            .children(&mut [
                snackbar::render(),
            ])
        })
    }
}
