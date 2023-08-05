use std::rc::Rc;

use dominator::{clone, html, link, svg, with_node, Dom, EventOptions};
use futures_signals::{
    map_ref,
    signal::{Mutable, SignalExt},
    signal_vec::{MutableVec, SignalVecExt},
};
use web_sys::HtmlInputElement;

use crate::{
    common::{
        events, snackbar, Cover, LibraryFilter, LibraryOrder, LibrarySettings, LibrarySort,
        LibrarySortBy, Route, Spinner,
    },
    query,
    utils::{is_tauri_signal, AsyncLoader},
};

pub struct Library {
    category_id: Option<i64>,
    category_name: Mutable<String>,
    keyword: Mutable<String>,
    is_search: Mutable<bool>,
    loader: AsyncLoader,
    spinner: Rc<Spinner>,
    cover_list: MutableVec<Cover>,
    categories_available: Mutable<bool>,
    library_settings: Rc<LibrarySettings>,
}

impl Library {
    pub fn new(category_id: Option<i64>) -> Rc<Self> {
        Rc::new(Library {
            category_id,
            category_name: Mutable::new("Default".to_string()),
            keyword: Mutable::new("".to_string()),
            is_search: Mutable::new(false),
            loader: AsyncLoader::new(),
            spinner: Spinner::new_with_fullscreen(true),
            cover_list: MutableVec::new(),
            categories_available: Mutable::new(category_id.is_some()),
            library_settings: LibrarySettings::new(false, true),
        })
    }

    pub fn fetch_category_detail(library: Rc<Self>) {
        library.spinner.set_active(true);
        if let Some(category_id) = library.category_id {
            library.loader.load(clone!(library => async move {
                match query::fetch_category_detail(category_id).await {
                    Ok(res) => {
                        library.category_name.set(res.name);
                        Self::fetch_libraries(library.clone());
                    }
                    Err(e) => {
                        snackbar::show(format!("failed to fetch library {}", e));
                    }
                }
                library.spinner.set_active(false);
            }));
        } else {
            library.loader.load(clone!(library => async move {
                match query::fetch_categories().await {
                    Ok(res) => {
                        library.categories_available.set(res.len() > 1);
                        Self::fetch_libraries(library.clone());
                    }
                    Err(e) => {
                        snackbar::show(format!("failed to fetch categories {}", e));
                    }
                }
                library.spinner.set_active(false);
            }));
        }
    }

    pub fn fetch_libraries(library: Rc<Self>) {
        let category_id = library.category_id;
        library.spinner.set_active(true);
        library.loader.load(clone!(library => async move {
            match query::fetch_manga_from_favorite(category_id).await {
                Ok(mut covers) => {
                    let sort = library.library_settings.sort.get();
                    covers.sort_by(|a, b| match sort {
                        LibrarySort { by: LibrarySortBy::Alphabetical, order: LibraryOrder::Asc } => a.title.partial_cmp(&b.title).unwrap_or(std::cmp::Ordering::Equal),
                        LibrarySort { by: LibrarySortBy::Alphabetical, order: LibraryOrder::Desc } => b.title.partial_cmp(&a.title).unwrap_or(std::cmp::Ordering::Equal),
                        LibrarySort { by: LibrarySortBy::RecentlyRead, order: LibraryOrder::Asc} => {
                            let a = a.last_read_at.unwrap_or_default();
                            let b = b.last_read_at.unwrap_or_default();
                            a.cmp(&b)
                        },
                        LibrarySort { by: LibrarySortBy::RecentlyRead, order: LibraryOrder::Desc} => {
                            let a = a.last_read_at.unwrap_or_default();
                            let b = b.last_read_at.unwrap_or_default();
                            b.cmp(&a)
                        },
                    });

                    library.cover_list.lock_mut().replace_cloned(covers);
                }
                Err(e) => {
                    snackbar::show(format!("failed to fetch library {}", e));
                }
            }
            library.spinner.set_active(false);
        }));
    }

    pub fn render_topbar(library: Rc<Self>) -> Dom {
        let categories_link_signal = map_ref! {
            let is_search = library.is_search.signal(),
            let categories_available = library.categories_available.signal() =>

            (*is_search, *categories_available)

        };

        html!("div", {
            .class("topbar")
            .class_signal("tauri", is_tauri_signal())
            .child_signal(categories_link_signal.map(|(is_search, categories_available)|
                (!is_search).then(|| if categories_available {
                    link!(Route::LibraryList.url(), {
                        .class("button")
                        .style("display", "flex")
                        .style("align-items", "center")
                        .children(&mut [
                            svg!("svg", {
                                .attr("xmlns", "http://www.w3.org/2000/svg")
                                .attr("fill", "none")
                                .attr("viewBox", "0 0 24 24")
                                .attr("stroke", "currentColor")
                                .class("icon")
                                .children(&mut [
                                    svg!("path", {
                                        .attr("stroke-linecap", "round")
                                        .attr("stroke-linejoin", "round")
                                        .attr("stroke-width", "2")
                                        .attr("d", "M15 19l-7-7 7-7")
                                    })
                                ])
                            }),
                            html!("span", {
                                .text("Categories")
                            })
                        ])
                    })
                } else {
                    html!("div", {
                        .style("min-width", "5rem")
                    })
                })
            ))
            .child_signal(library.is_search.signal().map(clone!(library => move |is_search| {
                if is_search {
                    Some(html!("input" => HtmlInputElement, {
                        .style("flex-grow", "1")
                        .attr("placeholder", "Search")
                        .attr("type", "text")
                        .with_node!(input => {
                            .event(clone!(library => move |_: events::Input| {
                                library.keyword.set_neq(input.value());
                            }))
                            .event_with_options(&EventOptions::preventable(), |event: events::KeyDown| {
                                if event.key() == "Enter" {
                                    event.prevent_default();
                                }
                            })
                        })
                    }))
                } else {
                    Some(html!("span", {
                        .class("title")
                        .text_signal(library.category_name.signal_cloned())
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
                                        .attr("xmlns", "http://www.w3.org/2000/svg")
                                        .attr("fill", "none")
                                        .attr("viewBox", "0 0 24 24")
                                        .attr("stroke", "currentColor")
                                        .class("icon")
                                        .children(&mut [
                                            svg!("path", {
                                                .attr("stroke-linecap", "round")
                                                .attr("stroke-linejoin", "round")
                                                .attr("stroke-width", "2")
                                                .attr("d", "M6 18L18 6M6 6l12 12")
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
                                        .attr("xmlns", "http://www.w3.org/2000/svg")
                                        .attr("fill", "none")
                                        .attr("viewBox", "0 0 24 24")
                                        .attr("stroke", "currentColor")
                                        .class("icon")
                                        .children(&mut [
                                            svg!("path", {
                                                .attr("stroke-linecap", "round")
                                                .attr("stroke-linejoin", "round")
                                                .attr("stroke-width", "2")
                                                .attr("d", "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z")
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
                                    .attr("xmlns", "http://www.w3.org/2000/svg")
                                    .attr("fill", "none")
                                    .attr("viewBox", "0 0 24 24")
                                    .attr("stroke", "currentColor")
                                    .class("icon")
                                    .children(&mut [
                                        svg!("path", {
                                            .attr("stroke-linecap", "round")
                                            .attr("stroke-linejoin", "round")
                                            .attr("stroke-width", "2")
                                            .attr("d", "M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z")
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
            .class("main")
            .style("padding", "0.5rem")
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
        Self::fetch_category_detail(library.clone());

        html!("div", {
            .future(library.library_settings.sort.signal_cloned().for_each(clone!(library => move |sort| {
                let mut covers = library.cover_list.lock_ref().to_vec();
                covers.sort_by(|a, b| match sort {
                    LibrarySort { by: LibrarySortBy::Alphabetical, order: LibraryOrder::Asc } => a.title.partial_cmp(&b.title).unwrap_or(std::cmp::Ordering::Equal),
                    LibrarySort { by: LibrarySortBy::Alphabetical, order: LibraryOrder::Desc } => b.title.partial_cmp(&a.title).unwrap_or(std::cmp::Ordering::Equal),
                    LibrarySort { by: LibrarySortBy::RecentlyRead, order: LibraryOrder::Asc} => {
                        let a = a.last_read_at.unwrap_or_default();
                        let b = b.last_read_at.unwrap_or_default();
                        a.cmp(&b)
                    },
                    LibrarySort { by: LibrarySortBy::RecentlyRead, order: LibraryOrder::Desc} => {
                        let a = a.last_read_at.unwrap_or_default();
                        let b = b.last_read_at.unwrap_or_default();
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
                Spinner::render(library.spinner.clone()),
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
