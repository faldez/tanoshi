use std::rc::Rc;

use crate::common::snackbar;
use crate::query;
use crate::utils::{is_tauri_signal, proxied_image_url, AsyncLoader};
use crate::{
    app::App,
    common::{Route, Spinner},
};
use dominator::{clone, events, html, link, routing, svg, Dom};
use futures_signals::signal::{Mutable, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVecExt};
use wasm_bindgen::UnwrapThrowExt;

#[derive(Debug, Clone)]
pub struct Entry {
    manga_id: i64,
    manga_title: String,
    cover_url: String,
    chapter_id: i64,
    chapter_title: String,
    uploaded: chrono::NaiveDateTime,
    cursor: String,
}

pub struct Updates {
    loader: AsyncLoader,
    spinner: Rc<Spinner>,
    entries: MutableVec<Entry>,
    is_entries_empty: Mutable<bool>,
}

impl Updates {
    pub fn new() -> Rc<Self> {
        Rc::new(Updates {
            spinner: Spinner::new(),
            loader: AsyncLoader::new(),
            entries: MutableVec::new(),
            is_entries_empty: Mutable::new(true),
        })
    }

    fn refresh_chapters(updates: Rc<Self>) {
        updates.loader.load(async move {
            snackbar::show(format!("Updating chapters..."));
            if let Err(e) = query::refresh_chapters(None, false).await {
                snackbar::show(format!("failed to refresh chapters: {e}"));
            }
        });
        
    }

    pub fn fetch_recent_chapters(updates: Rc<Self>) {
        updates.spinner.set_active(true);
        updates.loader.load(clone!(updates => async move {
            let cursor = updates.entries.lock_ref().last().map(|entry| entry.cursor.clone());
            match query::fetch_recent_updates(cursor).await {
                Ok(result) => {
                    for edge in result.edges {
                        updates.entries.lock_mut().push_cloned(Entry{
                            manga_id: edge.node.manga_id,
                            manga_title: edge.node.manga_title.clone(),
                            cover_url: edge.node.cover_url.clone(),
                            chapter_id: edge.node.chapter_id,
                            chapter_title: edge.node.chapter_title.clone(),
                            uploaded: chrono::NaiveDateTime::parse_from_str(&edge.node.uploaded, "%Y-%m-%dT%H:%M:%S%.f").unwrap_throw(),
                            cursor: edge.cursor.clone(),
                        })
                    }
                    updates.is_entries_empty.set(updates.entries.lock_ref().is_empty());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
            
            updates.spinner.set_active(false);
        }));
    }

    pub fn render_topbar(updates: Rc<Self>) -> Dom {
        html!("div", {
            .class("topbar")
            .class_signal("tauri", is_tauri_signal())
            .children(&mut [
                html!("div", {
                }),
                html!("span", {
                    .class("title")
                    .text("Updates")
                }),
                html!("div", {
                    .style("min-width", "2.25rem")
                    .children(&mut [
                        html!("button", {
                            .attr("id", "refresh-btn")
                            .style("padding", "0.25rem")
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
                                            .attr("d", "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15")
                                        })
                                    ])
                                }),
                            ])
                            .event(clone!(updates => move |_: events::Click| {
                                Self::refresh_chapters(updates.clone());
                            }))
                        }),
                    ])
                })
            ])
        })
    }

    pub fn render_main(updates: Rc<Self>) -> Dom {
        html!("ul", {
            .class("content")
            .class("list")
            .children_signal_vec(updates.entries.signal_vec_cloned().map(|entry| html!("li", {
                .class("list-item")
                .children(&mut [
                    link!(Route::Chapter(entry.chapter_id, 0).url(), {
                        .class("update-item")
                        .children(&mut [
                            html!("div", {
                                .class("update-item-thumbnail")
                                .children(&mut [
                                    html!("img", {
                                        .attr("src", &proxied_image_url(&entry.cover_url))
                                    })
                                ])
                            }),
                            html!("div", {
                                .class("update-item-detail")
                                .children(&mut [
                                    html!("span", {
                                        .class("manga-title")
                                        .text(&entry.manga_title)
                                    }),
                                    html!("span", {
                                        .class("last-chapter")
                                        .text(&entry.chapter_title)
                                    }),
                                    html!("span", {
                                        .class("date-updated")
                                        .text(&Self::calculate_days(entry.uploaded))
                                    })
                                ])
                            })
                        ])
                    }),
                    html!("button", {
                        .style("padding", "0.5rem")
                        .event(clone!(entry => move |_:events::Click| {
                            routing::go_to_url(Route::Manga(entry.manga_id).url().as_str());
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
                                        .attr("d", "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z")
                                    })
                                ])
                            }),
                        ])
                    })
                ])
            })))
        })
    }

    fn calculate_days(at: chrono::NaiveDateTime) -> String {
        let timestamp = js_sys::Date::now();
        let secs: i64 = (timestamp / 1000.0).floor() as i64;
        let nanoes: u32 = (timestamp as u32 % 1000) * 1_000_000;
        let today = chrono::NaiveDateTime::from_timestamp_opt(secs, nanoes).unwrap_or_default();
        let days = today.date().signed_duration_since(at.date()).num_days();

        if days == 0 {
            "Today".to_string()
        } else if days == 1 {
            "Yesterday".to_string()
        } else if days > 1 && days <= 7 {
            format!("{} Days Ago", days)
        } else if days > 7 && days < 31 {
            format!("{} Weeks Ago", days / 7)
        } else {
            format!("{} Months Ago", days / 30)
        }
    }

    pub fn render(updates: Rc<Self>, _app: Rc<App>) -> Dom {
        Self::fetch_recent_chapters(updates.clone());
        html! {"div", {
            .class("main")
            .children(&mut [
                Self::render_topbar(updates.clone()),
                html!("div", {
                    .class("topbar-spacing")
                }),
                Self::render_main(updates.clone()),
                html!("div", {
                    .class("load-more-btn")
                    .child_signal(updates.spinner.signal().map(clone!(updates => move |x| if x {
                        Some(Spinner::render(updates.spinner.clone()))
                    } else {
                        Some(html!("button", {
                            .class_signal("disabled", updates.is_entries_empty.signal())
                            .text_signal(updates.is_entries_empty.signal().map(|x|
                                if x {
                                    "No recent updates, favorite manga to see recent updates"
                                } else {
                                    "Load More"
                                }
                            ))
                            .event(clone!(updates => move |_: events::Click| {
                                Self::fetch_recent_chapters(updates.clone());
                            }))
                        }))
                    })))
                })
            ])
        }}
    }
}
