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
    read_at: chrono::NaiveDateTime,
    last_page_read: i64,
    cursor: String,
}

pub struct Histories {
    loader: AsyncLoader,
    spinner: Rc<Spinner>,
    entries: MutableVec<Entry>,
    is_entries_empty: Mutable<bool>,
}

impl Histories {
    pub fn new() -> Rc<Self> {
        Rc::new(Histories {
            spinner: Spinner::new(),
            loader: AsyncLoader::new(),
            entries: MutableVec::new(),
            is_entries_empty: Mutable::new(true),
        })
    }

    pub fn fetch_read_histories(histories: Rc<Self>) {
        histories.spinner.set_active(true);
        histories.loader.load(clone!(histories => async move {
            let cursor = histories.entries.lock_ref().last().map(|entry| entry.cursor.clone());
            match query::fetch_histories(cursor).await {
                Ok(result) => {
                    for edge in result.edges.unwrap_throw() {
                        histories.entries.lock_mut().push_cloned(Entry{
                            manga_id: edge.as_ref().unwrap_throw().node.manga_id,
                            manga_title: edge.as_ref().unwrap_throw().node.manga_title.clone(),
                            cover_url: edge.as_ref().unwrap_throw().node.cover_url.clone(),
                            chapter_id: edge.as_ref().unwrap_throw().node.chapter_id,
                            chapter_title: edge.as_ref().unwrap_throw().node.chapter_title.clone(),
                            read_at: chrono::NaiveDateTime::parse_from_str(&edge.as_ref().unwrap_throw().node.read_at, "%Y-%m-%dT%H:%M:%S%.f").unwrap_throw(),
                            last_page_read: edge.as_ref().unwrap_throw().node.last_page_read,
                            cursor: edge.as_ref().unwrap_throw().cursor.clone(),
                        })
                    }
                    histories.is_entries_empty.set(histories.entries.lock_ref().is_empty());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
            histories.spinner.set_active(false);
        }));
    }

    pub fn render_topbar() -> Dom {
        html!("div", {
            .class("topbar")
            .children(&mut [
                html!("div", {
                }),
                html!("span", {
                    .text("History")
                }),
                html!("div", {
                })
            ])
        })
    }

    pub fn render_main(histories: Rc<Self>) -> Dom {
        html!("ul", {
            .class("list")
            .children_signal_vec(histories.entries.signal_vec_cloned().map(|entry| html!("li", {
                .class("list-item")
                .children(&mut [
                    link!(Route::Chapter(entry.chapter_id, entry.last_page_read).url(), {
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
                                        .text(&Self::calculate_days(entry.read_at))
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

    pub fn render(histories: Rc<Self>, _app: Rc<App>) -> Dom {
        Self::fetch_read_histories(histories.clone());
        html!("div", {
            .children(&mut [
                Self::render_topbar(),
                html!("div", {
                    .class("topbar-spacing")
                }),
                Self::render_main(histories.clone()),
                html!("div", {
                    .class("load-more-btn")
                    .child_signal(histories.spinner.signal().map(clone!(histories => move |x| if x {
                        Some(Spinner::render(&histories.spinner))
                    } else {
                        Some(html!("button", {
                            .class_signal("disabled", histories.is_entries_empty.signal())
                            .text_signal(histories.is_entries_empty.signal().map(|x|
                                if x {
                                    "No recent histories, favorite manga to see recent histories"
                                } else {
                                    "Load More"
                                }
                            ))
                            .event(clone!(histories => move |_: events::Click| {
                                Self::fetch_read_histories(histories.clone());
                            }))
                        }))
                    })))
                })
            ])
        })
    }
}
