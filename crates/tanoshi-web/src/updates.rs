use std::rc::Rc;

use crate::common::{snackbar};
use crate::query;
use crate::utils::{proxied_image_url, AsyncLoader};
use crate::{
    app::App,
    common::{Route, Spinner},
};
use dominator::{clone, events, html, link, Dom};
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

    pub fn fetch_recent_chapters(updates: Rc<Self>) {
        updates.spinner.set_active(true);
        updates.loader.load(clone!(updates => async move {
            let cursor = updates.entries.lock_ref().last().map(|entry| entry.cursor.clone());
            match query::fetch_recent_updates(cursor).await {
                Ok(result) => {
                    for edge in result.edges.unwrap_throw() {
                        updates.entries.lock_mut().push_cloned(Entry{
                            manga_id: edge.as_ref().unwrap_throw().node.manga_id,
                            manga_title: edge.as_ref().unwrap_throw().node.manga_title.clone(),
                            cover_url: edge.as_ref().unwrap_throw().node.cover_url.clone(),
                            chapter_id: edge.as_ref().unwrap_throw().node.chapter_id,
                            chapter_title: edge.as_ref().unwrap_throw().node.chapter_title.clone(),
                            uploaded: chrono::NaiveDateTime::parse_from_str(&edge.as_ref().unwrap_throw().node.uploaded, "%Y-%m-%dT%H:%M:%S%.f").unwrap_throw(),
                            cursor: edge.as_ref().unwrap_throw().cursor.clone(),
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

    pub fn render_topbar() -> Dom {
        html!("div", {
            .class("topbar")
            .children(&mut [
                html!("div", {
                }),
                html!("span", {
                    .text("Updates")
                }),
                html!("div", {
                })
            ])
        })
    }

    pub fn render_main(updates: Rc<Self>) -> Dom {
        html!("ul", {
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
                                        .attribute("src", &proxied_image_url(&entry.cover_url))
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
                    })
                ])
            })))
        })
    }

    fn calculate_days(at: chrono::NaiveDateTime) -> String {
        let timestamp = js_sys::Date::now();
        let secs: i64 = (timestamp / 1000.0).floor() as i64;
        let nanoes: u32 = (timestamp as u32 % 1000) * 1_000_000;
        let today = chrono::NaiveDateTime::from_timestamp(secs, nanoes);
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
            .children(&mut [
                Self::render_topbar(),
                html!("div", {
                    .class("topbar-spacing")
                }),
                Self::render_main(updates.clone()),
                html!("div", {
                    .class("load-more-btn")
                    .child_signal(updates.spinner.signal().map(clone!(updates => move |x| if x {
                        Some(Spinner::render(&updates.spinner))
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
