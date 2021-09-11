use crate::common::snackbar;
use crate::common::Modal;
use crate::common::Route;
use crate::common::Spinner;
use crate::query;
use crate::utils::window;
use crate::utils::{proxied_image_url, AsyncLoader};
use chrono::NaiveDateTime;
use dominator::with_node;
use dominator::{clone, events, html, routing, svg, Dom};
use futures_signals::signal::SignalExt;
use futures_signals::{
    signal::Mutable,
    signal_vec::{MutableVec, SignalVecExt},
};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

#[derive(Clone)]
struct Chapter {
    pub id: i64,
    pub title: String,
    pub number: f64,
    pub scanlator: String,
    pub uploaded: NaiveDateTime,
    pub read_at: Mutable<Option<NaiveDateTime>>,
    pub last_page_read: Mutable<Option<i64>>,
    pub selected: Mutable<bool>,
}

#[derive(Clone)]
pub enum Sort {
    Number,
    ReadAt,
}

#[derive(Clone)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Clone)]
struct ChapterSort {
    by: Sort,
    order: Order,
}

#[derive(Clone)]
pub enum Filter {
    None,
    Read,
    Unread,
}

pub struct Manga {
    pub id: Mutable<i64>,
    pub source_id: Mutable<i64>,
    pub source_name: Mutable<String>,
    pub path: Mutable<String>,
    title: Mutable<Option<String>>,
    author: MutableVec<String>,
    genre: MutableVec<String>,
    cover_url: Mutable<Option<String>>,
    description: Mutable<Option<String>>,
    status: Mutable<Option<String>>,
    is_favorite: Mutable<bool>,
    chapters: MutableVec<Rc<Chapter>>,
    is_edit_chapter: Mutable<bool>,
    chapter_menu: Rc<Modal>,
    order: Mutable<ChapterSort>,
    filter: Mutable<Filter>,
    loader: AsyncLoader,
}

impl Manga {
    pub fn new(id: i64, source_id: i64, path: String) -> Rc<Self> {
        Rc::new(Self {
            id: Mutable::new(id),
            source_id: Mutable::new(source_id),
            source_name: Mutable::new("".to_string()),
            path: Mutable::new(path),
            title: Mutable::new(None),
            author: MutableVec::new(),
            genre: MutableVec::new(),
            cover_url: Mutable::new(None),
            description: Mutable::new(None),
            status: Mutable::new(None),
            is_favorite: Mutable::new(false),
            chapters: MutableVec::new(),
            is_edit_chapter: Mutable::new(false),
            chapter_menu: Modal::new(),
            order: Mutable::new(ChapterSort {
                by: Sort::Number,
                order: Order::Desc,
            }),
            filter: Mutable::new(Filter::None),
            loader: AsyncLoader::new(),
        })
    }

    fn fetch_detail(manga: Rc<Self>, refresh: bool) {
        manga.loader.load(clone!(manga => async move {
            match query::fetch_manga_detail(manga.id.get(), refresh).await {
                Ok(result) => {
                    manga.source_name.set(result.source.name);
                    manga.title.set_neq(Some(result.title));
                    manga.author.lock_mut().replace_cloned(result.author);
                    manga.genre.lock_mut().replace_cloned(result.genre);
                    manga.cover_url.set_neq(Some(result.cover_url));
                    manga.description.set_neq(result.description);
                    manga.status.set_neq(result.status);
                    manga.is_favorite.set_neq(result.is_favorite);
                    manga.chapters.lock_mut().replace_cloned(result.chapters.iter().map(|chapter| Rc::new(Chapter{
                        id: chapter.id,
                        title: chapter.title.clone(),
                        number: chapter.number,
                        scanlator: chapter.scanlator.clone(),
                        uploaded: NaiveDateTime::parse_from_str(&chapter.uploaded, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse uploaded date"),
                        read_at: Mutable::new(chapter.read_at.as_ref().map(|time| NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse read at date"))),
                        last_page_read: Mutable::new(chapter.last_page_read),
                        selected: Mutable::new(false)
                    })).collect());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    fn fetch_detail_by_source_path(manga: Rc<Self>) {
        manga.loader.load(clone!(manga => async move {
            match query::fetch_manga_by_source_path(manga.source_id.get(), manga.path.get_cloned()).await {
                Ok(result) => {
                    manga.id.set_neq(result.id);
                    manga.source_name.set(result.source.name);
                    manga.title.set_neq(Some(result.title));
                    manga.author.lock_mut().replace_cloned(result.author);
                    manga.genre.lock_mut().replace_cloned(result.genre);
                    manga.cover_url.set_neq(Some(result.cover_url));
                    manga.description.set_neq(result.description);
                    manga.status.set_neq(result.status);
                    manga.is_favorite.set_neq(result.is_favorite);
                    manga.chapters.lock_mut().replace_cloned(result.chapters.iter().map(|chapter| Rc::new(Chapter{
                        id: chapter.id,
                        title: chapter.title.clone(),
                        number: chapter.number,
                        scanlator: chapter.scanlator.clone(),
                        uploaded: NaiveDateTime::parse_from_str(&chapter.uploaded, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse uploaded date"),
                        read_at: Mutable::new(chapter.read_at.as_ref().map(|time| NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse read at date"))),
                        last_page_read: Mutable::new(chapter.last_page_read),
                        selected: Mutable::new(false)
                    })).collect());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    fn mark_chapter_as_read(manga: Rc<Self>) {
        let mut selected_chapter_id: Vec<i64> = vec![];
        for chapter in manga.chapters.lock_ref().to_vec() {
            if chapter.selected.get() {
                selected_chapter_id.push(chapter.id);
            }
        }

        manga.loader.load(clone!(manga => async move {
            match query::mark_chapter_as_read(&selected_chapter_id).await {
                Ok(_) => {},
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }

            manga.is_edit_chapter.set(false);
            if manga.id.get() != 0 {
                Self::fetch_detail(manga.clone(), false);
            } else if manga.source_id.get() != 0 && manga.path.get_cloned() != "" {
                Self::fetch_detail_by_source_path(manga.clone());
            }
        }))
    }

    fn mark_chapter_as_unread(manga: Rc<Self>) {
        let mut selected_chapter_id: Vec<i64> = vec![];
        for chapter in manga.chapters.lock_ref().to_vec() {
            if chapter.selected.get() {
                selected_chapter_id.push(chapter.id);
            }
        }

        manga.loader.load(clone!(manga => async move {
            match query::mark_chapter_as_unread(&selected_chapter_id).await {
                Ok(_) => {},
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }

            manga.is_edit_chapter.set(false);
            if manga.id.get() != 0 {
                Self::fetch_detail(manga.clone(), false);
            } else if manga.source_id.get() != 0 && manga.path.get_cloned() != "" {
                Self::fetch_detail_by_source_path(manga.clone());
            }
        }))
    }

    fn add_to_or_remove_from_library(manga: Rc<Self>) {
        if manga.id.get() == 0 {
            return;
        }
        manga.loader.load(clone!(manga => async move {
            if manga.is_favorite.get() {
                match query::delete_from_library(manga.id.get()).await {
                    Ok(_) => {
                        manga.is_favorite.set_neq(false);
                    },
                    Err(err) => {
                        snackbar::show(format!("{}", err));
                    }
                }
            } else {
                match query::add_to_library(manga.id.get()).await {
                    Ok(_) => {
                        manga.is_favorite.set_neq(true);
                    },
                    Err(err) => {
                        snackbar::show(format!("{}", err));
                    }
                }
            }
        }));
    }

    pub fn render_topbar(manga: Rc<Self>) -> Dom {
        html!("div", {
            .class("topbar")
            .children(&mut [
                html!("button", {
                    .style("display", "flex")
                    .style("align-items", "center")
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
                                    .attribute("d", "M15 19l-7-7 7-7")
                                })
                            ])
                        }),
                    ])
                    .child_signal(manga.source_id.signal().map(|source_id|
                        match source_id {
                            0 => Some(html!("span", {
                                .text("Library")
                                .event(|_: events::Click| {
                                    routing::go_to_url("/");
                                })
                            })),
                            _ => Some(html!("span", {
                                .text("Catalogue")
                                .event(|_: events::Click| {
                                    let history = window().history().unwrap();
                                    if history.length().unwrap() > 1 {
                                        let _ = history.back();
                                    } else {
                                        routing::go_to_url("/");
                                    }
                                })
                            })),
                        }
                    ))
                }),
                html!("span", {
                    .style("overflow", "hidden")
                    .style("text-overflow", "ellipsis")
                    .style("white-space", "nowrap")
                    .style("margin-left", "0.5rem")
                    .style("margin-right", "0.5rem")
                    .text_signal(manga.title.signal_cloned().map(|x| x.unwrap_or_else(|| "".to_string())))
                }),
                html!("button", {
                    .text("Refresh")
                    .event(clone!(manga => move |_: events::Click| {
                        if manga.id.get() != 0 {
                            manga.title.set_neq(None);
                            manga.author.lock_mut().clear();
                            manga.genre.lock_mut().clear();
                            manga.cover_url.set_neq(None);
                            manga.description.set_neq(None);
                            manga.status.set_neq(None);
                            manga.is_favorite.set_neq(false);
                            manga.chapters.lock_mut().clear();

                            Self::fetch_detail(manga.clone(), true);
                        }
                    }))
                }),
            ])
        })
    }

    pub fn render_header(manga: Rc<Self>) -> Dom {
        html!("div", {
            .class("manga-detail-header")
            .attribute("id", "detail")
            .children(&mut [
                html!("div", {
                    .style("display", "flex")
                    // .class_signal("animate-pulse", manga.loader.is_loading())
                    .children(&mut [
                        html!("div", {
                            .child_signal(manga.cover_url.signal_cloned().map(|x| {
                                if let Some(cover_url) = x {
                                    Some(html!("img", {
                                        .style("border-radius", "0.5rem")
                                        .style("width", "8rem")
                                        .style("height", "auto")
                                        .attribute("src", &proxied_image_url(&cover_url))
                                    }))
                                } else {
                                    Some(html!("div", {
                                        // .class(["w-32", "h-44", "rounded", "bg-gray-200", "dark:bg-gray-800"])
                                    }))
                                }
                            }))
                        }),
                        html!("div", {
                            .style("display", "flex")
                            .style("flex-direction", "column")
                            .children(&mut [
                                html!("div", {
                                    .style("margin-left", "0.5rem")
                                    .style("margin-bottom", "0.5rem")
                                    .style("font-size", "large")
                                    .text_signal(manga.title.signal_cloned().map(|x| {
                                        if let Some(title) = x {
                                            // Some(html!("span", {
                                            //     .class(["md:text-xl", "sm:text-md", "text-gray-900", "dark:text-gray-300", "mr-2"])
                                            //     .text(title.as_str())
                                            // }))
                                            title
                                        } else {
                                            // Some(html!("div", {
                                                // .class(["w-32", "md:w-48", "lg:w-64", "xl:w-72", "h-6", "mb-2", "rounded", "bg-gray-200", "dark:bg-gray-800"])
                                            // }))
                                            "".to_string()
                                        }
                                    }))
                                }),
                                html!("div", {
                                    .style("margin-left", "0.5rem")
                                    .style("margin-bottom", "0.5rem")
                                    .child_signal(manga.loader.is_loading().map(|x| {
                                        if x {
                                            Some(html!("div", {
                                                // .class(["w-32", "md:w-48", "lg:w-64", "xl:w-72", "h-6", "mb-2", "rounded", "bg-gray-200", "dark:bg-gray-800"])
                                            }))
                                        } else {
                                            None
                                        }
                                    }))
                                    .children_signal_vec(manga.author.signal_vec_cloned().map(|x| {
                                        html!("span", {
                                            // .class(["md:text-lg", "sm:text-sm", "text-gray-900", "dark:text-gray-300", "mr-2"])
                                            .style("margin-right", "0.5rem")
                                            .text(&x)
                                        })
                                    }))
                                }),
                                html!("div", {
                                    .style("margin-left", "0.5rem")
                                    .style("margin-bottom", "0.5rem")
                                    .text_signal(manga.status.signal_cloned().map(|x| if let Some(status) = x {
                                        status
                                    } else {
                                        "".to_string()
                                    }))
                                    .class_signal("skeleton", manga.status.signal_cloned().map(|x| x.is_none()))
                                    // .child_signal(manga.status.signal_cloned().map(|x|
                                    //     if let Some(status) = x {
                                    //         Some(html!("span", {
                                    //             .class(["md:text-lg", "sm:text-sm", "text-gray-900", "dark:text-gray-300", "mr-2"])
                                    //             .text(&status)
                                    //         }))
                                    //     } else {
                                    //         Some(html!("div", {
                                    //             .class(["w-32", "md:w-48", "lg:w-64", "xl:w-72", "h-6", "mb-2", "rounded", "bg-gray-200", "dark:bg-gray-800"])
                                    //         }))
                                    //     }
                                    // ))
                                }),

                                html!("div", {
                                    .style("margin-left", "0.5rem")
                                    .style("margin-top", "0.5rem")
                                    .text_signal(manga.source_name.signal_cloned())
                                    .class_signal("skeleton", manga.source_name.signal_cloned().map(|x| x.is_empty()))
                                    // .child_signal(manga.status.signal_cloned().map(|x|
                                    //     if let Some(status) = x {
                                    //         Some(html!("span", {
                                    //             .class(["md:text-lg", "sm:text-sm", "text-gray-900", "dark:text-gray-300", "mr-2"])
                                    //             .text(&status)
                                    //         }))
                                    //     } else {
                                    //         Some(html!("div", {
                                    //             .class(["w-32", "md:w-48", "lg:w-64", "xl:w-72", "h-6", "mb-2", "rounded", "bg-gray-200", "dark:bg-gray-800"])
                                    //         }))
                                    //     }
                                    // ))
                                }),
                            ])
                        })
                    ])
                })
            ])
        })
    }

    pub fn render_action(manga: Rc<Self>) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("width", "100%")
            .children(&mut [
                html!("button", {
                    .style("display", "flex")
                    .style("padding", "0.5rem")
                    .style("align-items", "center")
                    .children(&mut [
                        svg!("svg", {
                            .attribute("xmlns", "http://www.w3.org/2000/svg")
                            .attribute_signal("fill", manga.is_favorite.signal().map(|x| if x { "currentColor" } else { "none" }))
                            .attribute("viewBox", "0 0 24 24")
                            .attribute("stroke", "currentColor")
                            .class("icon")
                            .children(&mut [
                                svg!("path", {
                                    .attribute("stroke-linecap", "round")
                                    .attribute("stroke-linejoin", "round")
                                    .attribute("stroke-width", "1")
                                    .attribute("d", "M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z")
                                })
                            ])
                        }),
                        html!("span", {
                            .text("Favorite")
                        })
                    ])
                    .event(clone!(manga => move |_: events::Click| {
                        Self::add_to_or_remove_from_library(manga.clone());
                    }))
                })
            ])
        })
    }

    pub fn render_description(manga: Rc<Self>) -> Dom {
        html!("div", {
            .attribute("id", "description")
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("margin", "0.5rem")
            // .class_signal("animate-pulse", manga.loader.is_loading())
            .children(&mut [
                html!("span", {
                    .class("header")
                    .text("Description")
                })
            ])
            .child_signal(manga.description.signal_cloned().map(|x| {
                if let Some(description) = x {
                    Some(html!("p", {
                        .text(&description)
                    }))
                } else {
                    Some(html!("div", {
                        .class(["flex", "flex-col"])
                        .children(&mut [
                            html!("div", {
                                .class(["w-full", "h-6", "mb-2", "rounded", "bg-gray-200", "dark:bg-gray-800"])
                            }),
                            html!("div", {
                                .class(["w-full", "h-6", "mb-2", "rounded", "bg-gray-200", "dark:bg-gray-800"])
                            }),
                            html!("div", {
                                .class(["w-full", "h-6", "mb-2", "rounded", "bg-gray-200", "dark:bg-gray-800"])
                            })
                        ])
                    }))
                }
            }))
            .children(&mut [
                html!("botton", {
                    .style("display", "flex")
                    .style("flex-wrap", "wrap")
                    .children_signal_vec(manga.genre.signal_vec_cloned().map(|x| {
                        html!("span", {
                            .class("chip")
                            .text(&x)
                        })
                    }))
                })
            ])
        })
    }

    pub fn render_chapters(manga: Rc<Self>) -> Dom {
        let is_edit_chapter = manga.is_edit_chapter.clone();
        let filter = manga.filter.clone();

        html!("div", {
            .attribute("id", "chapters")
            .children(&mut [
                html!("div", {
                    .style("display", "flex")
                    .style("justify-content", "space-between")
                    .children(&mut [
                        html!("span", {
                            .class("header")
                            .style("margin-left", "0.5rem")
                            .text("Chapters")
                        }),
                        html!("div", {
                            .children(&mut [
                                html!("button", {
                                    .style("margin", "0.25rem")
                                    .child_signal( manga.is_edit_chapter.signal().map(|is_edit| if is_edit {
                                        Some(svg!("svg", {
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
                                        }))
                                    } else {
                                        Some(svg!("svg", {
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
                                                    .attribute("d", "M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z")
                                                })
                                            ])
                                        }))
                                    }))
                                    .event(clone!(manga => move |_: events::Click| {
                                        manga.is_edit_chapter.set(!manga.is_edit_chapter.get());
                                    }))
                                }),
                                html!("button", {
                                    .style("margin", "0.25rem")
                                    .event(clone!(manga => move |_: events::Click| {
                                        manga.chapter_menu.toggle_show()
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
                }),
                html!("ul", {
                    .class("list")
                    .children_signal_vec(manga.chapters.signal_vec_cloned().map(clone!(filter, is_edit_chapter => move |chapter| html!("li", {
                        .class("list-item")
                        .visible_signal(filter.signal_cloned().map(clone!(chapter => move |filter| match filter {
                            Filter::None => true,
                            Filter::Read => chapter.read_at.get().is_some(),
                            Filter::Unread => chapter.read_at.get().is_none(),
                        })))
                        .child_signal(is_edit_chapter.signal().map(clone!(chapter => move |is_edit_chapter| if is_edit_chapter {
                            Some(html!("input" => HtmlInputElement, {
                                .attribute("type", "checkbox")
                                .style("height", "0.5rem")
                                .style("margin", "0.5rem")
                                .style("margin-top", "auto")
                                .style("margin-bottom", "auto")
                                .with_node!(input => {
                                    .future(chapter.selected.signal().for_each(clone!(input => move |selected| {
                                        input.set_checked(selected);

                                        async{}
                                    })))
                                    .event(clone!(chapter => move |_: events::Change| {
                                        chapter.selected.set_neq(input.checked());
                                    }))
                                })
                            }))
                        } else {
                            None
                        })))
                        .children(&mut [
                            html!("div", {
                                .event_preventable(clone!(is_edit_chapter, chapter => move |e: events::Click| {
                                    e.prevent_default();
                                    if is_edit_chapter.get() {
                                        chapter.selected.set(!chapter.selected.get());
                                    }  else {
                                        routing::go_to_url(Route::Chapter(chapter.id, 0).url().as_str());
                                    }
                                }))
                                .style("display", "inline-flex")
                                .style("padding", "0.5rem")
                                .style("width", "100%")
                                .style_signal("opacity", chapter.read_at.signal().map(|x| if x.is_some() {"0.5"} else {"1"}))
                                .children(&mut [
                                    html!("div", {
                                        .style("display", "flex")
                                        .style("justify-content", "space-between")
                                        .style("align-items", "center")
                                        .style("width", "100%")
                                        .children(&mut [
                                            html!("div", {
                                                .style("display", "flex")
                                                .style("flex-direction", "column")
                                                .children(&mut [
                                                    html!("span", {
                                                        .style("font-weight", "500")
                                                        .style("margin-bottom", "0.5rem")
                                                        .text(&chapter.title)
                                                    }),
                                                    html!("div", {
                                                        .style("margin-bottom", "0.5rem")
                                                        .children(&mut [
                                                            html!("span", {
                                                                .style("font-size", "smaller")
                                                                .style("font-weight", "400")
                                                                .style("margin-right", "0.5rem")
                                                                .text(&chapter.uploaded.date().to_string())
                                                            }),
                                                            html!("span", {
                                                                .style("font-size", "small")
                                                                .style("font-weight", "400")
                                                                .style("margin-right", "0.5rem")
                                                                .text(&chapter.scanlator)
                                                            })
                                                        ])
                                                    })
                                                ])
                                                .child_signal(chapter.last_page_read.signal().map(|x| x.map(|page| html!("span", {
                                                    .style("font-size", "smaller")
                                                    .style("font-weight", "400")
                                                    .text(format!("Page: {}", page + 1).as_str())
                                                }))))
                                            }),
                                        ])
                                    })
                                ])
                            }),
                        ])
                    }))))
                })
            ])
        })
    }

    fn render_sort_setting(manga: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .text("Sort")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", manga.order.signal_cloned().map(|sort| matches!(sort.by, Sort::Number)))
                            .text("Number")
                            .event(clone!(manga => move |_: events::Click| manga.order.set(ChapterSort { by: Sort::Number, order: manga.order.get_cloned().order})))
                        }),
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", manga.order.signal_cloned().map(|sort| matches!(sort.by, Sort::ReadAt)))
                            .text("Recently Read")
                            .event(clone!(manga => move |_: events::Click| manga.order.set(ChapterSort { by: Sort::ReadAt, order: manga.order.get_cloned().order})))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_order_setting(manga: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .text("Order")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", manga.order.signal_cloned().map(|sort| matches!(sort.order, Order::Asc)))
                            .text("Ascending")
                            .event(clone!(manga => move |_: events::Click| manga.order.set(ChapterSort { by: manga.order.get_cloned().by.clone(), order: Order::Asc})))
                        }),
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", manga.order.signal_cloned().map(|sort| matches!(sort.order, Order::Desc)))
                            .text("Descending")
                            .event(clone!(manga => move |_: events::Click| manga.order.set(ChapterSort { by: manga.order.get_cloned().by.clone(), order: Order::Desc})))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_filter_setting(manga: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .text("Filter")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "33%")
                            .class_signal("active", manga.filter.signal_cloned().map(|x| matches!(x, Filter::None)))
                            .text("None")
                            .event(clone!(manga => move |_: events::Click| manga.filter.set(Filter::None)))
                        }),
                        html!("button", {
                            .style("width", "33%")
                            .class_signal("active", manga.filter.signal_cloned().map(|x| matches!(x, Filter::Read)))
                            .text("Read")
                            .event(clone!(manga => move |_: events::Click| manga.filter.set(Filter::Read)))
                        }),
                        html!("button", {
                            .style("width", "33%")
                            .class_signal("active", manga.filter.signal_cloned().map(|x| matches!(x, Filter::Unread)))
                            .text("Unread")
                            .event(clone!(manga => move |_: events::Click| manga.filter.set(Filter::Unread)))
                        }),
                    ])
                })
            ])
        })
    }

    pub fn render(manga_page: Rc<Self>) -> Dom {
        if manga_page.id.get() != 0 {
            Self::fetch_detail(manga_page.clone(), false);
        } else if manga_page.source_id.get() != 0 && manga_page.path.get_cloned() != "" {
            Self::fetch_detail_by_source_path(manga_page.clone());
        }

        html!("div", {
            .future(manga_page.order.signal_cloned().for_each(clone!(manga_page => move |sort| {
                let mut chapters = manga_page.chapters.lock_ref().to_vec();
                chapters.sort_by(|a, b| match sort {
                    ChapterSort { by: Sort::Number, order: Order::Asc} => a.number.partial_cmp(&b.number).unwrap_or(std::cmp::Ordering::Equal),
                    ChapterSort { by: Sort::Number, order: Order::Desc} => b.number.partial_cmp(&a.number).unwrap_or(std::cmp::Ordering::Equal),
                    ChapterSort { by: Sort::ReadAt, order: Order::Asc} => {
                        let a = a.read_at.get_cloned().unwrap_or(NaiveDateTime::from_timestamp(0, 0));
                        let b = b.read_at.get_cloned().unwrap_or(NaiveDateTime::from_timestamp(0, 0));
                        a.cmp(&b)
                    },
                    ChapterSort { by: Sort::ReadAt, order: Order::Desc} => {
                        let a = a.read_at.get_cloned().unwrap_or(NaiveDateTime::from_timestamp(0, 0));
                        let b = b.read_at.get_cloned().unwrap_or(NaiveDateTime::from_timestamp(0, 0));
                        b.cmp(&a)
                    }
                });
                manga_page.chapters.lock_mut().replace_cloned(chapters);

                async {}
            })))
            .style("display", "flex")
            .style("flex-direction", "column")
            .children(&mut [
                Self::render_topbar(manga_page.clone()),
                html!("div", {
                   .class("topbar-spacing")
                }),
            ])
            .child_signal(manga_page.loader.is_loading().map(clone!(manga_page => move |x| if x {
                Some(Spinner::render_spinner(false))
            } else {
                Some(html!("div", {
                    .children(&mut [
                        Self::render_header(manga_page.clone()),
                        Self::render_action(manga_page.clone()),
                        Self::render_description(manga_page.clone()),
                        Self::render_chapters(manga_page.clone())
                    ])
                }))
            })))
            .children(&mut [
                html!("div", {
                    .visible_signal(manga_page.is_edit_chapter.signal())
                    .class("bottombar-spacing")
                }),
                Modal::render(manga_page.chapter_menu.clone(), clone!(manga_page => html!("div", {
                    .style("width", "100%")
                    .style("display", "flex")
                    .style("flex-direction", "column")
                    .children(&mut [
                        Self::render_sort_setting(manga_page.clone()),
                        Self::render_order_setting(manga_page.clone()),
                        Self::render_filter_setting(manga_page.clone()),
                    ])
                })))
            ])
            .child_signal(manga_page.is_edit_chapter.signal().map(clone!(manga_page => move |is_edit| if is_edit {
                Some(html!("div",{
                    .class("bottombar")
                    .children(&mut [
                        html!("button", {
                            .style("margin", "auto")
                            .text("Mark as Read")
                            .event(clone!(manga_page => move |_:events::Click| {
                                Self::mark_chapter_as_read(manga_page.clone());
                            }))
                        }),
                        html!("button", {
                            .style("margin", "auto")
                            .text("Mark as Unread")
                            .event(clone!(manga_page => move |_:events::Click| {
                                Self::mark_chapter_as_unread(manga_page.clone());
                            }))
                        }),
                    ])
                }))
            } else {
                None
            })))
        })
    }
}
