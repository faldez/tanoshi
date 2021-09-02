use std::rc::Rc;

use dominator::routing;
use dominator::{clone, html, Dom};
use futures_signals::signal::{Mutable, SignalExt};
use wasm_bindgen::UnwrapThrowExt;

use crate::catalogue::Catalogue;
use crate::common::{snackbar, ServerStatus};
use crate::library::Library;
use crate::login::Login;
use crate::manga::Manga;
use crate::query;
use crate::reader::Reader;
use crate::utils::local_storage;
use crate::{
    common::{Bottombar, Route, Spinner},
    histories::Histories,
    settings::Settings,
    updates::Updates,
    utils::AsyncLoader,
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
            match query::server_status().await {
                Ok(server_status) => {
                    app.server_status.set_neq(Some(ServerStatus {
                        activated: server_status.activated,
                        version: server_status.version,
                    }));
                }
                Err(e) => {
                    snackbar::show(format!("error check server status: {}", e));
                }
            }
        }));
    }

    fn fetch_user(app: Rc<Self>) {
        app.loader.load(async move {
            match query::fetch_me().await {
                Ok(_) => {}
                Err(err) => {
                    snackbar::show(format!("{}", err));
                    error!("{}", err);
                    local_storage().delete("token").unwrap_throw();
                    routing::go_to_url(&Route::Login.url());
                }
            }
        });
    }

    pub fn render(app: Rc<Self>) -> Dom {
        Self::fetch_server_status(app.clone());

        html!("div", {
            .future(app.server_status.signal_cloned().for_each(clone!(app => move |server_status| {
                if let Some(server_status) = server_status {
                    let is_token_exist = local_storage().get("token").unwrap_throw().is_some();
                    if !server_status.activated {
                        info!("server inactivated, go to login");
                        local_storage().delete("token").unwrap_throw();
                        routing::go_to_url(&Route::Login.url());
                    } else if !is_token_exist {
                        routing::go_to_url(&Route::Login.url());
                    } else if server_status.activated && is_token_exist {
                        Self::fetch_user(app.clone());
                    }
                }

                async move {}
            })))
            .child_signal(Route::signal().map(clone!(app => move |x| {
                match x {
                    Route::Login => Some(
                        Login::render(Login::new(), app.clone())
                    ),
                    Route::Library => Some(
                        Library::render(Library::new()),
                    ),
                    Route::Catalogue{id, latest} => Some(
                        Catalogue::render(Catalogue::new(), id, latest),
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
                    Route::Settings(category) => Some(
                        Settings::render(Settings::new(), category),
                    ),
                    Route::NotFound => Some(
                        html!("div", {
                            .text("not found")
                        }),
                    )
                }
            })))
            .child_signal(Route::signal().map(|x| {
                match x {
                    Route::Login | Route::Manga(_) | Route::MangaBySourcePath(_, _) | Route::Chapter(_, _) => None,
                    _ => Some(html!("div", {
                        .children(&mut [
                            html!("div", {
                                .class("bottombar-spacing")
                            }),
                            Bottombar::render(),
                        ])
                    })),
                }
            }))
            .children(&mut [
                snackbar::render(),
            ])
        })
    }
}
