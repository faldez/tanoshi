use std::rc::Rc;

use crate::{
    catalogue,
    common::{snackbar, Route},
    query,
    utils::{is_tauri_signal, local_storage},
};
use crate::{
    common::{Cover, Spinner},
    utils::AsyncLoader,
};
use dominator::{clone, events, html, link, routing, svg, with_node, Dom, EventOptions};
use futures_signals::signal::{Mutable, SignalExt};
use futures_signals::signal_map::MutableBTreeMap;
use futures_signals::signal_vec::{self, MutableVec, SignalVecExt};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

pub const STORAGE_KEY: &str = "catalogue_list";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct Source {
    id: i64,
    name: String,
    version: String,
    icon: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct SourceManga {
    name: String,
    covers: Vec<Cover>,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct CatalogueList {
    keyword: Mutable<String>,
    is_search: Mutable<bool>,
    cover_list_map: MutableBTreeMap<i64, SourceManga>,
    #[serde(skip)]
    sources: MutableVec<Source>,
    #[serde(skip)]
    loader: AsyncLoader,
    #[serde(skip)]
    spinner: Rc<Spinner>,
}

impl Default for CatalogueList {
    fn default() -> Self {
        Self {
            keyword: Mutable::new("".to_string()),
            is_search: Mutable::new(false),
            cover_list_map: MutableBTreeMap::new(),
            spinner: Spinner::new(),
            sources: MutableVec::new(),
            loader: AsyncLoader::new(),
        }
    }
}

impl CatalogueList {
    pub fn new() -> Rc<Self> {
        let catalogue = local_storage()
            .get(STORAGE_KEY)
            .unwrap_throw()
            .and_then(|object_str| serde_json::from_str(&object_str).ok())
            .unwrap_or_default();

        Rc::new(catalogue)
    }

    pub fn serialize_into_json(&self) -> String {
        serde_json::to_string(self).unwrap_throw()
    }

    pub fn fetch_manga_from_all_sources(catalogue: Rc<Self>) {
        let sources: Vec<Source> = catalogue.sources.lock_ref().iter().cloned().collect();
        let keyword = catalogue.keyword.get_cloned();
        catalogue.loader.load(clone!(catalogue => async move {
            for source in sources {
                match query::fetch_manga_from_source(source.id, 1, Some(keyword.clone()), None).await {
                    Ok(data) => {
                        let covers = data
                            .browse_source
                            .iter()
                            .map(|item| {
                                Cover::new(
                                    item.id,
                                    source.id,
                                    item.path.clone(),
                                    item.title.clone(),
                                    item.cover_url.clone(),
                                    item.is_favorite,
                                    None,
                                    0,
                                )
                            })
                            .collect();
                        let mut cover_list = catalogue.cover_list_map.lock_mut();
                        cover_list.insert_cloned(source.id, SourceManga { name: source.name.clone(), covers } );
                    }
                    Err(e) => {
                        // snackbar::show(format!("Fetch manga from source failed: {}", e))
                        error!("Fetch manga from source failed: {}", e);
                    }
                }

                let state = catalogue.serialize_into_json();
                local_storage().set("catalogue_list", state.as_str()).unwrap_throw();
            }
        }));
    }

    pub fn fetch_sources(catalogue: Rc<Self>) {
        catalogue.loader.load(clone!(catalogue => async move {
            match query::fetch_sources().await {
                Ok(result) => {
                    let sources: Vec<Source> = result.iter().map(|s| Source {
                        id: s.id,
                        name: s.name.clone(),
                        version: s.version.clone(),
                        icon: s.icon.clone(),
                    }).collect();

                    let mut cover_list_map = catalogue.cover_list_map.lock_mut();
                    for source in sources.iter() {
                        cover_list_map.insert_cloned(source.id, SourceManga{name: source.name.clone(), covers: vec![]});
                    }

                    catalogue.sources.lock_mut().replace_cloned(sources);
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    pub fn render_topbar(catalogue: Rc<Self>) -> Dom {
        html!("div", {
            .class("topbar")
            .class_signal("tauri", is_tauri_signal())
            .child_signal(catalogue.is_search.signal().map(|is_search| {
                (!is_search).then(|| html!("div", {
                    .style("width", "24px")
                }))
            }))
            .child_signal(catalogue.is_search.signal().map(clone!(catalogue => move |is_search| {
                if is_search {
                    Some(html!("input" => HtmlInputElement, {
                        .style("width", "100%")
                        .attr("placeholder", "Search")
                        .attr("type", "text")
                        .attr("value", &catalogue.keyword.get_cloned())
                        .with_node!(input => {
                            .event(clone!(catalogue => move |_: events::Input| {
                                catalogue.keyword.set_neq(input.value());
                            }))
                            .event_with_options(&EventOptions::preventable(), clone!(catalogue => move |event: events::KeyDown| {
                                if event.key() == "Enter" {
                                    event.prevent_default();
                                    Self::fetch_manga_from_all_sources(catalogue.clone());
                                }
                            }))
                        })
                    }))
                } else {
                    Some(html!("span", {
                        .class("title")
                        .text("Catalogue")
                    }))
                }
            })))
            .child_signal(catalogue.is_search.signal().map(clone!(catalogue => move |is_search| {
                if is_search {
                    Some(html!("button", {
                        .attr("id", "cancel")
                        .event(clone!(catalogue => move |_: events::Click| {
                            catalogue.is_search.set_neq(false);
                            if catalogue.keyword.get_cloned() != "" {
                                catalogue.keyword.set_neq("".to_string());
                            }

                            local_storage().delete(STORAGE_KEY).unwrap_throw();
                            Self::fetch_sources(catalogue.clone());
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
                        .attr("id", "search")
                        .event(clone!(catalogue => move |_: events::Click| {
                            catalogue.is_search.set_neq(true);
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
        })
    }

    fn render_source_list(source: &Source) -> Dom {
        html!("li", {
            .class("list-item")
            .children(&mut [
                link!(Route::Catalogue{id: source.id, latest: false, query: None}.url(), {
                    .class("source-item")
                    .children(&mut [
                        html!("img", {
                            .class_signal("invisible", Mutable::new(source.icon.clone()).signal_cloned().map(|icon| icon.is_empty()))
                            .attr("src", &source.icon)
                        }),
                        html!("span", {
                            .text(&source.name)
                        }),
                    ])
                }),
                link!(Route::Catalogue{id: source.id, latest: true, query: None}.url(), {
                    .class("source-action")
                    .text("latest")
                }),
            ])
        })
    }

    pub fn render_main(catalogue: Rc<Self>) -> Dom {
        html!("ul", {
            .class("content")
            .class("list")
            .children(&mut [
                html!("span", {
                    .style("margin", "0.5rem")
                    .text("Local Sources")
                })
            ])
            .children_signal_vec(catalogue.sources.signal_vec_cloned().filter(|s| s.id  >= 10000).map(|source| Self::render_source_list(&source)))
            .children(&mut [
                html!("span", {
                    .style("margin", "0.5rem")
                    .text("External Sources")
                })
            ])
            .children_signal_vec(catalogue.sources.signal_vec_cloned().filter(|s| s.id  < 10000).map(|source| Self::render_source_list(&source)))
        })
    }

    pub fn render_global_search(catalogue: Rc<Self>) -> Dom {
        html!("div", {
            .class("content")
            .style("margin", "0.5rem")
            .children_signal_vec(catalogue.cover_list_map.entries_cloned().map(clone!(catalogue => move |(source_id, source_manga)| html!("div", {
                .children(&mut [
                    html!("div", {
                        .style("display", "flex")
                        .style("justify-content", "space-between")
                        .style("margin", "0.25rem")
                        .children(&mut [
                            html!("span", {
                                .text(&source_manga.name)
                            }),
                            html!("button", {
                                .text("More")
                                .event(clone!(catalogue => move |_:events::Click| {
                                    let state =  catalogue.serialize_into_json();
                                    local_storage().set(STORAGE_KEY, state.as_str()).unwrap_throw();

                                    routing::go_to_url(Route::Catalogue{id: source_id, latest: false, query: Some(catalogue.keyword.get_cloned())}.url().as_str());
                                }))
                            }),
                        ])
                    }),
                    html!("div", {
                        .style("overflow-x", "scroll")
                        .style("overflow-y", "hidden")
                        .style("white-space", " nowrap")
                        .style("border-radius", "0.375rem")
                        .child_signal(catalogue.loader.is_loading().map(clone!(source_manga => move |is_loading| if is_loading && source_manga.covers.is_empty() {
                            Some(Spinner::render_spinner(false))
                        } else {
                            None
                        })))
                        .children_signal_vec(signal_vec::always(source_manga.covers).map(|cover| link!(cover.link(), {
                            .style("margin", "0.25rem")
                            .style("position", "relative")
                            .children(&mut [
                                html!("img", {
                                    .style("height", "12rem")
                                    .style("border-radius", "0.375rem")
                                    .attr("src", &cover.cover_url)
                                }),
                                html!("div", {
                                    .style("position", "absolute")
                                    .style("bottom", "0px")
                                    .style("left", "0px")
                                    .style("font-size", "0.875rem")
                                    .style("background-image", "linear-gradient(to top, #000000aa, transparent)")
                                    .style("width", "100%")
                                    .style("color", "#f7fafc")
                                    .style("padding-bottom", "0.25rem")
                                    .style("padding-top", "1.75rem")
                                    .style("overflow", "hidden")
                                    .style("text-overflow", "ellipsis")
                                    .style("white-space", "nowrap")
                                    .style("border-bottom-right-radius", "0.375rem")
                                    .style("border-bottom-left-radius", "0.375rem")
                                    .children(&mut [
                                        html!("span", {
                                            .style("margin", "0.125rem")
                                            .text(&cover.title)
                                        })
                                    ])
                                })
                            ])
                        })))
                    })
                ])
            }))))
        })
    }

    pub fn render(self: Rc<Self>) -> Dom {
        local_storage()
            .delete(catalogue::STORAGE_KEY)
            .unwrap_throw();
        if !self.is_search.get() {
            Self::fetch_sources(self.clone());
        }

        html!("div", {
            .class("main")
            .children(&mut [
                Self::render_topbar(self.clone()),
                html!("div", {
                    .class("topbar-spacing")
                })
            ])
            .child_signal(self.is_search.signal_cloned().map({
                let catalogue = self.clone();
                move |is_search| if is_search {
                    Some(Self::render_global_search(catalogue.clone()))
                } else {
                    Some(Self::render_main(catalogue.clone()))
                }
            }))
            .children(&mut [
                html!("div", {
                    .class("bottombar-spacing")
                })
            ])
        })
    }
}
