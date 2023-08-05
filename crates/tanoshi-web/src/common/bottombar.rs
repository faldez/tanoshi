use std::rc::Rc;

use crate::{
    catalogue_list,
    common::events,
    utils::{is_tauri_signal, local_storage},
};

use super::{Route, SettingCategory};
use dominator::{html, link, svg, Dom};
use futures_signals::signal::SignalExt;
use wasm_bindgen::UnwrapThrowExt;

pub struct Bottombar {}

impl Bottombar {
    #[allow(dead_code)]
    pub fn new() -> Rc<Self> {
        Rc::new(Bottombar {})
    }

    pub fn render() -> Dom {
        html!("div", {
            .class("bottombar")
            .class_signal("tauri", is_tauri_signal())
            .children(&mut [
                link!(Route::Root.url(), {
                    .class_signal("active", Route::signal().map(|x| matches!(x, Route::LibraryList | Route::Library(_))))
                    .children(&mut [
                        svg!("svg", {
                            .attr("xmlns", "http://www.w3.org/2000/svg")
                            .attr("viewBox", "0 0 24 24")
                            .attr("stroke", "currentColor")
                            .attr("fill", "none")
                            .children(&mut [
                                svg!("path", {
                                    .attr("stroke-linecap", "round")
                                    .attr("stroke-linejoin", "round")
                                    .attr("stroke-width", "1")
                                    .class("heroicon-ui")
                                    .attr("d", "M8 14v3m4-3v3m4-3v3M3 21h18M3 10h18M3 7l9-4 9 4M4 10h16v11H4V10z")
                                })
                            ])
                        }),
                        html!("span", {
                            .text("Library")
                        })
                    ])
                }),
                link!(Route::CatalogueList.url(), {
                    .class_signal("active", Route::signal().map(|x| matches!(x, Route::CatalogueList)))
                    .event(|_:events::Click| {
                        local_storage().delete(catalogue_list::STORAGE_KEY).unwrap_throw();
                    })
                    .children(&mut [
                        svg!("svg", {
                            .attr("xmlns", "http://www.w3.org/2000/svg")
                            .attr("viewBox", "0 0 24 24")
                            .attr("stroke", "currentColor")
                            .attr("fill", "none")
                            .children(&mut [
                                svg!("path", {
                                    .attr("stroke-linecap", "round")
                                    .attr("stroke-linejoin", "round")
                                    .attr("stroke-width", "1")
                                    .class("heroicon-ui")
                                    .attr("d", "M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10")
                                })
                            ])
                        }),
                        html!("span", {
                            .text("Catalogue")
                        })
                    ])
                }),
                link!(Route::Updates.url(), {
                    .class_signal("active", Route::signal().map(|x| matches!(x, Route::Updates)))
                    .children(&mut [
                        svg!("svg", {
                            .attr("xmlns", "http://www.w3.org/2000/svg")
                            .attr("viewBox", "0 0 24 24")
                            .attr("stroke", "currentColor")
                            .attr("fill", "none")
                            .children(&mut [
                                svg!("path", {
                                    .attr("stroke-linecap", "round")
                                    .attr("stroke-linejoin", "round")
                                    .attr("stroke-width", "1")
                                    .class("heroicon-ui")
                                    .attr("d", "M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9")
                                })
                            ])
                        }),
                        html!("span", {
                            .text("Updates")
                        })
                    ])
                }),
                link!(Route::Histories.url(), {
                    .class_signal("active", Route::signal().map(|x| matches!(x, Route::Histories)))
                    .children(&mut [
                        svg!("svg", {
                            .attr("xmlns", "http://www.w3.org/2000/svg")
                            .attr("viewBox", "0 0 24 24")
                            .attr("stroke", "currentColor")
                            .attr("fill", "none")
                            .children(&mut [
                                svg!("path", {
                                    .attr("stroke-linecap", "round")
                                    .attr("stroke-linejoin", "round")
                                    .attr("stroke-width", "1")
                                    .class("heroicon-ui")
                                    .attr("d", "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z")
                                })
                            ])
                        }),
                        html!("span", {
                            .text("History")
                        })
                    ])
                }),
                link!(Route::Settings(SettingCategory::None).url(), {
                    .class_signal("active", Route::signal().map(|x| matches!(x, Route::Settings(_))))
                    .children(&mut [
                        svg!("svg", {
                            .attr("xmlns", "http://www.w3.org/2000/svg")
                            .attr("viewBox", "0 0 24 24")
                            .attr("stroke", "currentColor")
                            .attr("fill", "none")
                            .children(&mut [
                                svg!("path", {
                                    .attr("stroke-linecap", "round")
                                    .attr("stroke-linejoin", "round")
                                    .attr("stroke-width", "1")
                                    .class("heroicon-ui")
                                    .attr("d", "M5 12h.01M12 12h.01M19 12h.01M6 12a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0z")
                                }),
                            ])
                        }),
                        html!("span", {
                            .text("More")
                        })
                    ])
                }),
            ])
        })
    }
}
