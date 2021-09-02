use std::rc::Rc;

use dominator::{Dom, clone, html};
use futures_signals::signal_vec::{MutableVec, SignalVecExt};

use crate::{common::{Cover, Spinner, events, snackbar}, query, utils::AsyncLoader};

pub struct Library {
    loader: AsyncLoader,
    spinner: Rc<Spinner>,
    cover_list: MutableVec<Cover>,
}

impl Library {
    pub fn new() -> Rc<Self> {
        Rc::new(Library {
            loader: AsyncLoader::new(),
            spinner: Spinner::new_with_fullscreen(true),
            cover_list: MutableVec::new(),
        })
    }

    pub fn fetch_libraries(library: Rc<Self>, refresh: bool) {
        library.spinner.set_active(true);
        library.loader.load(clone!(library => async move {
            match query::fetch_manga_from_favorite(refresh).await {
                Ok(covers) => {
                    let mut cover_list = library.cover_list.lock_mut();
                    cover_list.replace_cloned(covers);
                }
                Err(e) => {
                    snackbar::show(format!("failed to fetch library {}", e));
                }
            }
            library.spinner.set_active(false);
        }));
    }

    pub fn render_topbar(library: Rc<Self>) -> Dom {
        html!("div", {
            .class("topbar")
            .children(&mut [
                html!("button", {
                    .text("Filter")
                }),
                html!("span", {
                    .text("Library")
                }),
                html!("button", {
                    .text("Refresh")
                    .event(clone!(library => move |_: events::Click| {
                        Self::fetch_libraries(library.clone(), true);
                    }))
                })
            ])
        })
    }

    pub fn render_main(library: &Self) -> Dom {
        html!("div", {
            .class("manga-grid")
            .children_signal_vec(library.cover_list.signal_vec_cloned().map(move |cover| cover.render()))
        })
    }

    pub fn render(library: Rc<Self>) -> Dom {
        Self::fetch_libraries(library.clone(), false);

        html!("div", {
            .class("page")
            .children(&mut [
                Self::render_topbar(library.clone()),
                html!("div", {
                    .class("topbar-spacing")
                }),
                Self::render_main(&library),
                Spinner::render(&library.spinner)
            ])
        })
    }
}
