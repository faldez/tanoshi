use crate::{
    common::{events, snackbar, Spinner},
    query,
    utils::AsyncLoader,
};
use dominator::{clone, html, svg, with_node, Dom};

use futures_signals::{
    signal::{Mutable, SignalExt},
    signal_vec::MutableVec,
    signal_vec::SignalVecExt,
};
use std::rc::Rc;
use web_sys::HtmlInputElement;

#[derive(Debug, Clone)]
pub struct Entry {
    chapter_id: i64,
    manga_title: String,
    chapter_title: String,
    cursor: String,
    selected: Mutable<bool>,
}

#[allow(dead_code)]
pub struct SettingsManageDownloads {
    entries: MutableVec<Entry>,
    is_entries_empty: Mutable<bool>,
    loader: AsyncLoader,
    spinner: Rc<Spinner>,
}

#[allow(dead_code)]
impl SettingsManageDownloads {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            entries: MutableVec::new(),
            is_entries_empty: Mutable::new(true),
            loader: AsyncLoader::new(),
            spinner: Spinner::new(),
        })
    }

    fn fetch_downloaded_chapter(self: &Rc<Self>) {
        self.spinner.set_active(true);
        self.loader.load({
            let settings = self.clone();
            async move {
                let cursor = settings
                    .entries
                    .lock_ref()
                    .last()
                    .map(|entry| entry.cursor.clone());
                match query::fetch_downloaded_chapters(cursor).await {
                    Ok(result) => {
                        for edge in result.edges {
                            settings.entries.lock_mut().push_cloned(Entry {
                                manga_title: edge.node.manga.title.clone(),
                                chapter_id: edge.node.id,
                                chapter_title: edge.node.title.clone(),
                                cursor: edge.cursor.clone(),
                                selected: Mutable::new(false),
                            })
                        }
                        settings
                            .is_entries_empty
                            .set(settings.entries.lock_ref().is_empty());
                    }
                    Err(err) => {
                        snackbar::show(format!("{}", err));
                    }
                }

                settings.spinner.set_active(false);
            }
        });
    }

    fn remove_download_chapters(settings: Rc<Self>, ids: Vec<i64>) {
        settings.spinner.set_active(true);
        settings.loader.load(clone!(settings => async move {
            match query::remove_downloaded_chapters(&ids).await {
                Ok(_) => {},
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }

            settings.spinner.set_active(false);

            settings.entries.lock_mut().clear();
            settings.fetch_downloaded_chapter();
        }))
    }

    pub fn render(settings: Rc<Self>) -> Dom {
        settings.fetch_downloaded_chapter();
        html!("div", {
            .class("content")
            .children(&mut [
                html!("div",{
                    .style("font-size", "smaller")
                    .style("display", "flex")
                    .style("justify-content", "flex-end")
                    .children(&mut [
                        html!("button", {
                            .attr("id", "select-all")
                            .style("display", "flex")
                            .style("align-items", "center")
                            .children(&mut [
                                svg!("svg", {
                                    .attr("xmlns", "http://www.w3.org/2000/svg")
                                    .attr("viewBox", "0 0 20 20")
                                    .attr("fill", "currentColor")
                                    .class("icon")
                                    .children(&mut [
                                        svg!("path", {
                                            .attr("fill-rule", "evenodd")
                                            .attr("d", "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z")
                                            .attr("clip-rule", "evenodd")
                                        })
                                    ])
                                }),
                                html!("span", {
                                    .text("Select All")
                                })
                            ])
                            .event(clone!(settings => move |_:events::Click| {
                                let chapters = settings.entries.lock_ref();
                                for chapter in chapters.iter() {
                                    chapter.selected.set(true);
                                }
                            }))
                        }),
                        html!("button", {
                            .attr("id", "deselect-all")
                            .style("display", "flex")
                            .style("align-items", "center")
                            .children(&mut [
                                svg!("svg", {
                                    .attr("xmlns", "http://www.w3.org/2000/svg")
                                    .attr("viewBox", "0 0 20 20")
                                    .attr("fill", "currentColor")
                                    .class("icon")
                                    .children(&mut [
                                        svg!("path", {
                                            .attr("fill-rule", "evenodd")
                                            .attr("d", "M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z")
                                            .attr("clip-rule", "evenodd")
                                        })
                                    ])
                                }),
                                html!("span", {
                                    .text("Deselect All")
                                })
                            ])
                            .event(clone!(settings => move |_:events::Click| {
                                let chapters = settings.entries.lock_ref();
                                for chapter in chapters.iter() {
                                    chapter.selected.set(false);
                                }
                            }))
                        }),
                        html!("button", {
                            .attr("id", "remove-download")
                            .style("display", "flex")
                            .style("align-items", "center")
                            .children(&mut [
                                svg!("svg", {
                                    .attr("xmlns", "http://www.w3.org/2000/svg")
                                    .attr("viewBox", "0 0 20 20")
                                    .attr("fill", "currentColor")
                                    .class("icon")
                                    .children(&mut [
                                        svg!("path", {
                                            .attr("fill-rule", "evenodd")
                                            .attr("d", "M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z")
                                            .attr("clip-rule", "evenodd")
                                        })
                                    ])
                                }),
                                html!("span", {
                                    .text("Remove Selected")
                                })
                            ])
                            .event(clone!(settings => move |_:events::Click| {
                                let selected_chapter_id: Vec<i64> = settings
                                    .entries
                                    .lock_ref()
                                    .to_vec()
                                    .iter()
                                    .filter_map(|entry| entry.selected.get().then(|| entry.chapter_id))
                                    .collect();
                                Self::remove_download_chapters(settings.clone(), selected_chapter_id);
                            }))
                        }),
                    ])
                }),
                html!("ul", {
                    .class("list")
                    .children_signal_vec(settings.entries.signal_vec_cloned().map(clone!(settings => move |entry| html!("li", {
                        .class("list-item")
                        .children(&mut [
                            html!("input" => HtmlInputElement, {
                                .attr("type", "checkbox")
                                .style("height", "0.5rem")
                                .style("margin", "0.5rem")
                                .style("margin-top", "auto")
                                .style("margin-bottom", "auto")
                                .with_node!(input => {
                                    .future(entry.selected.signal().for_each(clone!(input => move |selected| {
                                        input.set_checked(selected);

                                        async{}
                                    })))
                                    .event(clone!(entry => move |_: events::Change| {
                                        entry.selected.set_neq(input.checked());
                                    }))
                                })
                            }),
                            html!("div", {
                                .class("update-item")
                                .children(&mut [
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
                                        ])
                                    })
                                ])
                            }),
                            html!("button", {
                                .style("padding", "0.5rem")
                                .style("color", "red")
                                .event(clone!(settings, entry => move |_:events::Click| {
                                    Self::remove_download_chapters(settings.clone(), vec![entry.chapter_id]);
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
                                                .attr("d", "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16")
                                            })
                                        ])
                                    }),
                                ])
                            })
                        ])
                    }))))
                }),
                html!("div", {
                    .class("load-more-btn")
                    .child_signal(settings.spinner.signal().map(clone!(settings => move |x| if x {
                        Some(Spinner::render(settings.spinner.clone()))
                    } else {
                        Some(html!("button", {
                            .class_signal("disabled", settings.is_entries_empty.signal())
                            .text_signal(settings.is_entries_empty.signal().map(|x|
                                if x {
                                    "No downloaded chapter(s)"
                                } else {
                                    "Load More"
                                }
                            ))
                            .event(clone!(settings => move |_: events::Click| {
                                settings.fetch_downloaded_chapter();
                            }))
                        }))
                    })))
                })
            ])
        })
    }
}
