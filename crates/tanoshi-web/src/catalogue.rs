use std::rc::Rc;

use crate::{
    common::{snackbar, Cover, InputList, Spinner},
    query,
    utils::{history, local_storage, window, AsyncLoader},
};
use dominator::{clone, events, html, routing, svg, with_node, Dom, EventOptions};
use futures_signals::signal::{Mutable, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVecExt};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::HtmlInputElement;

pub const STORAGE_KEY: &str = "catalogue";

macro_rules! map_data_to_cover {
    ($data:ident, $catalogue:ident, $field:tt) => {
        let covers = $data
            .$field
            .iter()
            .map(|item| {
                Cover::new(
                    item.id,
                    $catalogue.source_id,
                    item.path.clone(),
                    item.title.clone(),
                    item.cover_url.clone(),
                    item.is_favorite,
                    None,
                    0,
                )
            })
            .collect();
        let mut cover_list = $catalogue.cover_list.lock_mut();
        if $catalogue.page.get() == 1 {
            cover_list.replace_cloned(covers);
        } else if $catalogue.page.get() > 0 {
            cover_list.extend(covers);
        }
    };
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Catalogue {
    source_id: i64,
    source_name: Mutable<String>,
    keyword: Mutable<Option<String>>,
    latest: Mutable<bool>,
    page: Mutable<i64>,
    is_search: Mutable<bool>,
    is_filter: Mutable<bool>,
    cover_list: MutableVec<Cover>,
    input_list_modal: Rc<InputList>,
    #[serde(skip)]
    loader: AsyncLoader,
    #[serde(skip)]
    spinner: Rc<Spinner>,
}

impl Default for Catalogue {
    fn default() -> Self {
        Self {
            source_id: 0,
            source_name: Mutable::new("Catalogue".to_string()),
            keyword: Mutable::new(None),
            latest: Mutable::new(false),
            page: Mutable::new(1),
            is_search: Mutable::new(false),
            is_filter: Mutable::new(false),
            cover_list: MutableVec::new(),
            input_list_modal: Rc::new(InputList::new(true)),
            spinner: Spinner::new(),
            loader: AsyncLoader::new(),
        }
    }
}

impl Catalogue {
    pub fn new(source_id: i64) -> Rc<Self> {
        let catalogue = local_storage()
            .get(STORAGE_KEY)
            .unwrap_throw()
            .and_then(|object_str| serde_json::from_str(&object_str).ok())
            .unwrap_or_else(|| Self {
                source_id,
                ..Default::default()
            });

        Rc::new(catalogue)
    }

    pub fn serialize_into_json(&self) -> String {
        serde_json::to_string(self).unwrap_throw()
    }

    pub fn fetch_source_filters(catalogue: Rc<Self>) {
        catalogue.spinner.set_active(true);
        catalogue.loader.load(clone!(catalogue => async move {
            match query::fetch_source_filters(catalogue.source_id).await {
                Ok(data) => {
                    catalogue.input_list_modal.set(data.source.filters);
                }
                Err(e) => {
                    snackbar::show(format!("Fetch manga from source failed: {}", e))
                }
            }
            catalogue.spinner.set_active(false);
        }))
    }

    pub fn fetch_mangas(catalogue: Rc<Self>) {
        catalogue.replace_state_with_url();
        catalogue.spinner.set_active(true);
        catalogue.loader.load(clone!(catalogue => async move {
            if let Some(keyword) = catalogue.keyword.get_cloned() {
                match query::fetch_manga_from_source(catalogue.source_id, catalogue.page.get(), Some(keyword), None).await {
                    Ok(data) => {
                        catalogue.source_name.set(data.source.name);
                        map_data_to_cover!(data, catalogue, browse_source);
                    }
                    Err(e) => {
                        snackbar::show(format!("Fetch manga from source failed: {}", e))
                    }
                }
            } else if catalogue.is_filter.get() {
                match query::fetch_manga_from_source(catalogue.source_id, catalogue.page.get(), None, Some(catalogue.input_list_modal.input_list.lock_ref().to_vec())).await {
                    Ok(data) => {
                        catalogue.source_name.set(data.source.name.clone());
                        map_data_to_cover!(data, catalogue, browse_source);
                    }
                    Err(e) => {
                        snackbar::show(format!("Fetch manga from source failed: {}", e))
                    }
                }
            } else if catalogue.latest.get() {
                match query::get_latest_manga(catalogue.source_id, catalogue.page.get()).await {
                    Ok(data) => {
                        catalogue.source_name.set(data.source.name.clone());
                        map_data_to_cover!(data, catalogue, get_latest_manga);
                    }
                    Err(e) => {
                        snackbar::show(format!("Fetch manga from source failed: {}", e))
                    }
                }
            } else {
                match query::get_popular_manga(catalogue.source_id, catalogue.page.get()).await {
                    Ok(data) => {
                        catalogue.source_name.set(data.source.name.clone());
                        map_data_to_cover!(data, catalogue, get_popular_manga);
                    }
                    Err(e) => {
                        snackbar::show(format!("Fetch manga from source failed: {}", e))
                    }
                }
            }

            let state =  catalogue.serialize_into_json();
            local_storage().set(STORAGE_KEY, state.as_str()).unwrap_throw();
            catalogue.spinner.set_active(false);
        }));
    }

    fn replace_state_with_url(&self) {
        let url = if self.latest.get() {
            format!("/catalogue/{}/latest", self.source_id,)
        } else {
            let mut param = vec![];

            if let Some(query) = self.keyword.get_cloned() {
                param.push(format!("query={}", query));
            };

            // if let Ok(filters) = serde_json::to_string(&self.input_list_modal.input_list) {
            //     param.push(format!("filters={}", filters));
            // };

            if param.is_empty() {
                format!("/catalogue/{}", self.source_id,)
            } else {
                format!("/catalogue/{}?{}", self.source_id, param.join("&"))
            }
        };

        if let Err(e) = history().replace_state_with_url(&JsValue::null(), "", Some(&url)) {
            let message = if let Some(msg) = e.as_string() {
                msg
            } else {
                "unknown reason".to_string()
            };

            error!("error replace_state_with_url: {}", message);
        }
    }

    pub fn render_topbar(catalogue: Rc<Self>) -> Dom {
        html!("div", {
            .class("topbar")
            .child_signal(catalogue.is_search.signal().map(|is_search| {
                (!is_search).then(|| html!("button", {
                    .style("display", "flex")
                    .style("align-items", "center")
                    .event(|_: events::Click| {
                        let history = window().history().unwrap();
                        if history.length().unwrap() > 1 {
                            let _ = history.back();
                        } else {
                            routing::go_to_url("/");
                        }
                    })
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
                            .text("Sources")
                        })
                    ])
                }))
            }))
            .child_signal(catalogue.is_search.signal().map(clone!(catalogue => move |is_search| {
                if is_search {
                    Some(html!("input" => HtmlInputElement, {
                        .style("width", "100%")
                        .attr("placeholder", "Search")
                        .attr("type", "text")
                        .attr("value", &catalogue.keyword.get_cloned().unwrap_or_else(|| "".to_string()))
                        .with_node!(input => {
                            .event_with_options(&EventOptions::preventable(), clone!(catalogue => move |e: events::KeyDown| {
                                if e.key() == "Enter" {
                                    e.prevent_default();
                                    catalogue.cover_list.lock_mut().clear();
                                    catalogue.page.set_neq(1);
                                    if !input.value().is_empty() {
                                        catalogue.keyword.set_neq(Some(input.value()));
                                        Self::fetch_mangas(catalogue.clone());
                                    }
                                }
                            }))
                        })
                    }))
                } else {
                    Some(html!("span", {
                        .class("title")
                        .text_signal(catalogue.source_name.signal_cloned())
                    }))
                }
            })))
            .child_signal(catalogue.is_search.signal().map(clone!(catalogue => move |is_search| {
                if is_search {
                    Some(html!("button", {
                        .attr("id", "cancel")
                        .event(clone!(catalogue => move |_: events::Click| {
                            catalogue.is_search.set_neq(false);
                            if catalogue.keyword.get_cloned().is_some() {
                                catalogue.keyword.set_neq(None);
                                catalogue.cover_list.lock_mut().clear();
                                catalogue.page.set_neq(1);
                                Self::fetch_mangas(catalogue.clone());
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
                    Some(html!("div", {
                        .style("min-width", "5rem")
                        .style("display", "flex")
                        .style("justify-content", "flex-end")
                        .children(&mut [
                            html!("button", {
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
                            }),
                            html!("button", {
                                .attr("id", "filter")
                                .style("margin-left", "0.5rem")
                                .event(clone!(catalogue => move |_: events::Click| {
                                    if catalogue.input_list_modal.input_list.lock_ref().is_empty() {
                                        Self::fetch_source_filters(catalogue.clone());
                                    }
                                    catalogue.input_list_modal.show();
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
                    }))
                }
            })))
        })
    }

    pub fn render_main(catalogue: Rc<Self>) -> Dom {
        html!("div", {
            .style("padding", "0.5rem")
            .children(&mut [
                html!("div", {
                    .class("manga-grid")
                    .children_signal_vec(catalogue.cover_list.signal_vec_cloned().map(|cover| cover.render()))
                }),
                html!("div", {
                    .class("load-more-btn")
                    .child_signal(catalogue.spinner.signal().map(clone!(catalogue => move |x| if x {
                        Some(Spinner::render(catalogue.spinner.clone()))
                    } else {
                        Some(html!("button", {
                            .text("Load More")
                            .event(clone!(catalogue => move |_: events::Click| {
                                catalogue.page.set(catalogue.page.get() + 1);
                                Self::fetch_mangas(catalogue.clone());
                            }))
                        }))
                    })))
                })
            ])
        })
    }

    pub fn render(self: Rc<Self>, latest: bool, query: Option<String>) -> Dom {
        if self.cover_list.lock_ref().is_empty() {
            Self::fetch_mangas(self.clone());
        }

        // let s = map_ref! {
        //     let keyword = self.keyword.signal_cloned(),

        //     (keyword.clone())
        // };

        self.is_search.set_neq(query.is_some());
        self.keyword.set_neq(query);
        self.latest.set_neq(latest);

        html!("div", {
            .future(self.keyword.signal_cloned().for_each({
                let catalogue = self.clone();
                move |_| {
                    catalogue.replace_state_with_url();

                    async {}
                }
            }))
            .children(&mut [
                Self::render_topbar(self.clone()),
                html!("div", {
                    .class("topbar-spacing")
                })
            ])
            .children(&mut [
                Self::render_main(self.clone()),
                InputList::render(self.input_list_modal.clone(), {
                    let catalogue = self.clone();
                    move || {
                        catalogue.cover_list.lock_mut().clear();
                        catalogue.page.set(1);
                        catalogue.keyword.set(None);
                        catalogue.is_filter.set(true);
                        Self::fetch_mangas(catalogue.clone());
                    }
                }),
                html!("div", {
                    .class("bottombar-spacing")
                })
            ])
        })
    }
}
