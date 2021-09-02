use std::rc::Rc;

use dominator::{Dom, clone, events, html, link, with_node};
use futures_signals::signal::{Mutable, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVecExt};
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;

use crate::common::{snackbar};
use crate::{
    common::Route,
    query::{
        self,
        browse_source::{SortByParam, SortOrderParam},
    },
    utils::local_storage,
};
use crate::{
    common::{Cover, Spinner},
    utils::AsyncLoader,
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Source {
    id: i64,
    name: String,
    version: String,
    icon: String,
    need_login: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Catalogue {
    pub source_id: Mutable<i64>,
    keyword: Mutable<String>,
    page: Mutable<i64>,
    sort_by: Mutable<SortByParam>,
    sort_order: Mutable<SortOrderParam>,
    is_search: Mutable<bool>,
    cover_list: MutableVec<Cover>,
    #[serde(skip)]
    sources: MutableVec<Source>,
    #[serde(skip)]
    loader: AsyncLoader,
    #[serde(skip)]
    spinner: Rc<Spinner>,
}

impl Default for Catalogue {
    fn default() -> Self {
        Self {
            source_id: Mutable::new(0),
            keyword: Mutable::new("".to_string()),
            page: Mutable::new(1),
            sort_by: Mutable::new(SortByParam::VIEWS),
            sort_order: Mutable::new(SortOrderParam::DESC),
            is_search: Mutable::new(false),
            cover_list: MutableVec::new(),
            spinner: Spinner::new(),
            sources: MutableVec::new(),
            loader: AsyncLoader::new(),
        }
    }
}

impl Catalogue {
    pub fn new() -> Rc<Self> {
        let catalogue = local_storage()
            .get("catalogue")
            .unwrap_throw()
            .and_then(|object_str| serde_json::from_str(&object_str).ok())
            .unwrap_or_default();
        
        Rc::new(catalogue)
    }

    pub fn reset(&self) {
        local_storage().delete("catalogue").unwrap_throw();

        self.source_id.set(0);
        self.keyword.set("".to_string());
        self.page.set(1);
        self.sort_by.set(SortByParam::VIEWS);
        self.sort_order.set(SortOrderParam::DESC);
        self.is_search.set(false);
        self.cover_list.lock_mut().clear();
        self.sources.lock_mut().clear();
    }

    pub fn serialize_into_json(&self) -> String {
        serde_json::to_string(self).unwrap_throw()
    }

    pub fn fetch_mangas(catalogue: Rc<Self>) {
        catalogue.spinner.set_active(true);
        catalogue.loader.load(clone!(catalogue => async move {
            match query::fetch_manga_from_source(*catalogue.source_id.lock_ref(), catalogue.page.get(), Some(catalogue.keyword.get_cloned()), catalogue.sort_by.get_cloned(), catalogue.sort_order.get_cloned()).await {
                Ok(covers) => {
                    let mut cover_list = catalogue.cover_list.lock_mut();
                    if catalogue.page.get() == 1 {
                        cover_list.replace_cloned(covers);
                    } else if catalogue.page.get() > 0 {
                        cover_list.extend(covers);
                    }

                }
                Err(e) => {
                    snackbar::show(format!("Fetch manga from source failed: {}", e))
                }
            }
            
            let state =  catalogue.serialize_into_json();
            local_storage().set("catalogue", state.as_str()).unwrap_throw();
            catalogue.spinner.set_active(false);
        }));
    }

    pub fn fetch_sources(catalogue: Rc<Self>) {
        catalogue.loader.load(clone!(catalogue => async move {
            match query::fetch_sources().await {
                Ok(result) => {
                    catalogue.sources.lock_mut().replace_cloned(result.iter().map(|s| Source {
                        id: s.id,
                        name: s.name.clone(),
                        version: s.version.clone(),
                        icon: s.icon.clone(),
                        need_login: s.need_login,
                    }).collect()
                )},
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    pub fn render_topbar(catalogue: Rc<Self>) -> Dom {
        html!("div", {
            .class("topbar")
            .child_signal(catalogue.is_search.signal().map(|is_search| {
                if is_search {
                    None
                } else {
                    Some(html!("button", {
                        .text("Filter")
                    }))
                }
            }))
            .child_signal(catalogue.is_search.signal().map(clone!(catalogue => move |is_search| {
                if is_search {
                    Some(html!("input" => HtmlInputElement, {
                        .style("width", "100%")
                        .attribute("placeholder", "Search")
                        .attribute("type", "text")
                        .property_signal("value", catalogue.keyword.signal_cloned())
                        .with_node!(input => {
                            .event(clone!(catalogue => move |_: events::Input| {
                                catalogue.keyword.set_neq(input.value());
                            }))
                            .event_preventable(clone!(catalogue => move |event: events::KeyDown| {
                                if event.key() == "Enter" {
                                    event.prevent_default();
                                    catalogue.cover_list.lock_mut().clear();
                                    catalogue.page.set_neq(1);
                                    Self::fetch_mangas(catalogue.clone());
                                }
                            }))
                        })
                    }))
                } else {
                    Some(html!("span", {
                        .text("Catalogue")
                    }))
                }
            })))
            .child_signal(catalogue.is_search.signal().map(clone!(catalogue => move |is_search| {
                if is_search {
                    Some(html!("button", {
                        .text("Cancel")
                        .event(clone!(catalogue => move |_: events::Click| {
                            catalogue.is_search.set_neq(false);
                            if catalogue.keyword.get_cloned() != "" {
                                catalogue.keyword.set_neq("".to_string());
                                catalogue.cover_list.lock_mut().clear();
                                catalogue.page.set_neq(1);
                                Self::fetch_mangas(catalogue.clone());
                            }
                        }))
                    }))
                } else {
                    Some(html!("button", {
                        .text("Search")
                        .event(clone!(catalogue => move |_: events::Click| {
                            catalogue.is_search.set_neq(true);
                        }))
                    }))
                }
            })))
        })
    }

    #[allow(dead_code)]
    pub fn render_search(catalogue: Rc<Self>) -> Dom {
        html!("div", {
            .class("search-box")
            .class([
                "w-full",
                "mb-2",
                "ml-0",
                "xl:ml-48",
                "p-2",
                "inline-flex",
                "transition-all",
            ])
            .visible_signal(catalogue.is_search.signal())
            .children(&mut [
                html!("input" => HtmlInputElement, {
                    .class([
                        "border",
                        "rounded",
                        "focus:outline-none",
                        "w-full",
                        "mr-2",
                        "p-1"
                    ])
                    .attribute("placeholder", "Search")
                    .attribute("type", "text")
                    .property_signal("value", catalogue.keyword.signal_cloned())
                    .with_node!(input => {
                        .event(clone!(catalogue => move |_: events::Input| {
                            catalogue.keyword.set_neq(input.value());
                        }))
                        .event_preventable(clone!(catalogue => move |event: events::KeyDown| {
                            if event.key() == "Enter" {
                                event.prevent_default();
                                catalogue.cover_list.lock_mut().clear();
                                catalogue.page.set_neq(1);
                                Self::fetch_mangas(catalogue.clone());
                            }
                        }))
                    })
                }),
                html!("button", {
                    .text("Cancel")
                    .event(clone!(catalogue => move |_: events::Click| {
                        catalogue.is_search.set_neq(false);
                        if catalogue.keyword.get_cloned() != "" {
                            catalogue.keyword.set_neq("".to_string());
                            catalogue.cover_list.lock_mut().clear();
                            catalogue.page.set_neq(1);
                            Self::fetch_mangas(catalogue.clone());
                        }
                    }))
                })
            ])
        })
    }

    pub fn render_main(catalogue: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("div", {
                    .class("manga-grid")
                    .children_signal_vec(catalogue.cover_list.signal_vec_cloned().map(|cover| cover.render()))
                }),
                html!("div", {
                    .class("load-more-btn")
                    .child_signal(catalogue.spinner.signal().map(clone!(catalogue => move |x| if x {
                        Some(Spinner::render(&catalogue.spinner))
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

    pub fn render_select(catalogue: Rc<Self>) -> Dom {
        html!("ul", {
            .class("list")
            .children_signal_vec(catalogue.sources.signal_vec_cloned().map(|source| html!("li", {
                .class("list-item")
                .children(&mut [
                    link!(Route::Catalogue{id: source.id, latest: false}.url(), {
                        .class("source-item")
                        .children(&mut [
                            html!("img", {
                                .class_signal("invisible", Mutable::new(source.icon.clone()).signal_cloned().map(|icon| icon.is_empty()))
                                .attribute("src", &source.icon)
                            }),
                            html!("span", {
                                .text(&source.name)
                            }),
                        ])
                    }),
                    link!(Route::Catalogue{id: source.id, latest: true}.url(), {
                        .class("source-action")
                        .text("latest")
                    }),
                ])
            })))
        })
    }

    pub fn render(catalogue: Rc<Self>, source_id: i64, latest: bool) -> Dom {
        if source_id > 0 && source_id != catalogue.source_id.get() {
            catalogue.source_id.set(source_id);
            let (sort_by, sort_order) = if latest {
                (SortByParam::LAST_UPDATED, SortOrderParam::DESC)
            } else {
                (SortByParam::VIEWS, SortOrderParam::DESC)
            };
            catalogue.sort_by.set(sort_by);
            catalogue.sort_order.set(sort_order);
            Self::fetch_mangas(catalogue.clone());
        } else if source_id == 0 {
            catalogue.reset();
            Self::fetch_sources(catalogue.clone());
        }

        html!("div", {
            .class("page")
            .children(&mut [
                Self::render_topbar(catalogue.clone()),
                html!("div", {
                    .class("topbar-spacing")
                })
            ])
            .child_signal(catalogue.source_id.signal().map(clone!(catalogue => move |x| if x > 0 {
                Some(Self::render_main(catalogue.clone()))
            } else {
                Some(Self::render_select(catalogue.clone()))
            })))
            .children(&mut [
                html!("div", {
                    .class("bottombar-spacing")
                })
            ])
        })
    }
}
