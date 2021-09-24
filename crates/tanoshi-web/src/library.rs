use std::rc::Rc;

use chrono::NaiveDateTime;
use dominator::{clone, html, svg, with_node, Dom};
use futures_signals::{
    signal::{Mutable, SignalExt},
    signal_vec::{MutableVec, SignalVecExt},
};
use web_sys::HtmlInputElement;

use crate::{
    common::{
        events, snackbar, Cover, LibraryFilter, LibraryOrder, LibrarySettings, LibrarySort,
        LibrarySortBy, Spinner,
    },
    query,
    utils::AsyncLoader,
};

pub struct Library {
    keyword: Mutable<String>,
    is_search: Mutable<bool>,
    loader: AsyncLoader,
    spinner: Rc<Spinner>,
    cover_list: MutableVec<Cover>,
    library_settings: Rc<LibrarySettings>,
}

impl Library {
    pub fn new() -> Rc<Self> {
        Rc::new(Library {
            keyword: Mutable::new("".to_string()),
            is_search: Mutable::new(false),
            loader: AsyncLoader::new(),
            spinner: Spinner::new_with_fullscreen(true),
            cover_list: MutableVec::new(),
            library_settings: LibrarySettings::new(false, true),
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
            .child_signal(library.is_search.signal().map(clone!(library => move |is_search|
                (!is_search).then(|| html!("div", {
                    .style("min-width", "5rem")
                    .children(&mut [
                        html!("button", {
                            .event(clone!(library => move |_: events::Click| {
                                Self::fetch_libraries(library.clone(), true);
                            }))
                            .children(&mut [
                                svg!("svg", {
                                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                                    .attribute("fill", "none")
                                    .attribute("viewBox", "0 0 24 24")
                                    .attribute("stroke", "currentColor")
                                    .class("icon")
                                    .children(&mut [
                                        svg!("path", {
                                            .attribute("stroke-linecap", "round")
                                            .attribute("stroke-linejoin", "round")
                                            .attribute("stroke-width", "2")
                                            .attribute("d", "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15")
                                        })
                                    ])
                                }),
                            ])
                        })
                    ])
                }))
            )))
            .child_signal(library.is_search.signal().map(clone!(library => move |is_search| {
                if is_search {
                    Some(html!("input" => HtmlInputElement, {
                        .style("flex-grow", "1")
                        .attribute("placeholder", "Search")
                        .attribute("type", "text")
                        .property_signal("value", library.keyword.signal_cloned())
                        .with_node!(input => {
                            .event(clone!(library => move |_: events::Input| {
                                library.keyword.set_neq(input.value());
                            }))
                            .event_preventable(|event: events::KeyDown| {
                                if event.key() == "Enter" {
                                    event.prevent_default();
                                }
                            })
                        })
                    }))
                } else {
                    Some(html!("span", {
                        .text("Library")
                    }))
                }
            })))
            .children(&mut [
                html!("div", {
                    .style("min-width", "5rem")
                    .child_signal(library.is_search.signal().map(clone!(library => move |is_search| {
                        if is_search {
                            Some(html!("button", {
                                .style("margin-left","0.5rem")
                                .style("margin-right","0.5rem")
                                .event(clone!(library => move |_: events::Click| {
                                    library.is_search.set_neq(false);
                                    if library.keyword.get_cloned() != "" {
                                        library.keyword.set_neq("".to_string());
                                    }
                                }))
                                .children(&mut [
                                    svg!("svg", {
                                        .attribute("xmlns", "http://www.w3.org/2000/svg")
                                        .attribute("fill", "none")
                                        .attribute("viewBox", "0 0 24 24")
                                        .attribute("stroke", "currentColor")
                                        .class("icon")
                                        .children(&mut [
                                            svg!("path", {
                                                .attribute("stroke-linecap", "round")
                                                .attribute("stroke-linejoin", "round")
                                                .attribute("stroke-width", "2")
                                                .attribute("d", "M6 18L18 6M6 6l12 12")
                                            })
                                        ])
                                    }),
                                ])
                            }))
                        } else {
                            Some(html!("button", {
                                .style("margin-left","0.5rem")
                                .style("margin-right","0.5rem")
                                .event(clone!(library => move |_: events::Click| {
                                    library.is_search.set_neq(true);
                                }))
                                .children(&mut [
                                    svg!("svg", {
                                        .attribute("xmlns", "http://www.w3.org/2000/svg")
                                        .attribute("fill", "none")
                                        .attribute("viewBox", "0 0 24 24")
                                        .attribute("stroke", "currentColor")
                                        .class("icon")
                                        .children(&mut [
                                            svg!("path", {
                                                .attribute("stroke-linecap", "round")
                                                .attribute("stroke-linejoin", "round")
                                                .attribute("stroke-width", "2")
                                                .attribute("d", "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z")
                                            })
                                        ])
                                    }),
                                ])
                            }))
                        }
                    })))
                    .children(&mut [
                        html!("button", {
                            .style("margin-left","0.5rem")
                            .style("margin-right","0.5rem")
                            .event(clone!(library => move |_: events::Click| {
                                library.library_settings.toggle_show();
                            }))
                            .children(&mut [
                                svg!("svg", {
                                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                                    .attribute("fill", "none")
                                    .attribute("viewBox", "0 0 24 24")
                                    .attribute("stroke", "currentColor")
                                    .class("icon")
                                    .children(&mut [
                                        svg!("path", {
                                            .attribute("stroke-linecap", "round")
                                            .attribute("stroke-linejoin", "round")
                                            .attribute("stroke-width", "2")
                                            .attribute("d", "M5 12h.01M12 12h.01M19 12h.01M6 12a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0z")
                                        })
                                    ])
                                }),
                            ])
                        })
                    ])
                })
            ])
        })
    }

    pub fn render_main(library: Rc<Self>, keyword: String) -> Dom {
        html!("div", {
            .child_signal(library.library_settings.filter.signal_cloned().map(clone!(library => move |filter| Some(html!("div", {
                .class("manga-grid")
                .children_signal_vec(library.cover_list.signal_vec_cloned().filter_map(clone!(filter, keyword => move |cover| {
                    if keyword.is_empty() || cover.title.to_ascii_lowercase().contains(&keyword.to_ascii_lowercase()) {
                        match filter {
                            LibraryFilter::None => Some(cover.render()),
                            LibraryFilter::Read => (cover.unread_chapter_count == 0).then(|| cover.render()),
                            LibraryFilter::Unread => (cover.unread_chapter_count > 0).then(|| cover.render())
                        }
                    } else {
                        None
                    }
                })))
            })))))
        })
    }

    pub fn render(library: Rc<Self>) -> Dom {
        Self::fetch_libraries(library.clone(), false);

        html!("div", {
            .class("page")
            .future(library.library_settings.sort.signal_cloned().for_each(clone!(library => move |sort| {
                let mut covers = library.cover_list.lock_ref().to_vec();
                covers.sort_by(|a, b| match sort {
                    LibrarySort { by: LibrarySortBy::Alphabetical, order: LibraryOrder::Asc } => a.title.partial_cmp(&b.title).unwrap_or(std::cmp::Ordering::Equal),
                    LibrarySort { by: LibrarySortBy::Alphabetical, order: LibraryOrder::Desc } => b.title.partial_cmp(&a.title).unwrap_or(std::cmp::Ordering::Equal),
                    LibrarySort { by: LibrarySortBy::RecentlyRead, order: LibraryOrder::Asc} => {
                        let a = a.last_read_at.unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));
                        let b = b.last_read_at.unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));
                        a.cmp(&b)
                    },
                    LibrarySort { by: LibrarySortBy::RecentlyRead, order: LibraryOrder::Desc} => {
                        let a = a.last_read_at.unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));
                        let b = b.last_read_at.unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));
                        b.cmp(&a)
                    },
                });
                library.cover_list.lock_mut().replace_cloned(covers);

                async {}
            })))
            .children(&mut [
                Self::render_topbar(library.clone()),
                html!("div", {
                    .class("topbar-spacing")
                }),
                Spinner::render(&library.spinner),
                LibrarySettings::render(library.library_settings.clone()),
            ])
            .child_signal(
                library.keyword.signal_cloned().map(
                    clone!(library => move |keyword| Some(Self::render_main(library.clone(), keyword)))
                )
            )
        })
    }
}
