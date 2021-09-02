use crate::common::Spinner;
use crate::common::snackbar;
use crate::common::Route;
use crate::query;
use crate::utils::window;
use crate::utils::{proxied_image_url, AsyncLoader};
use chrono::NaiveDateTime;
use dominator::{clone, events, html, link, routing, svg, Dom};
use futures_signals::signal::SignalExt;
use futures_signals::{
    signal::Mutable,
    signal_vec::{MutableVec, SignalVecExt},
};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[derive(Clone)]
struct Chapter {
    pub id: i64,
    pub title: String,
    pub uploaded: NaiveDateTime,
    pub read_at: Mutable<Option<NaiveDateTime>>,
    pub last_page_read: Mutable<Option<i64>>,
}

pub struct Manga {
    pub id: Mutable<i64>,
    pub source_id: Mutable<i64>,
    pub path: Mutable<String>,
    title: Mutable<Option<String>>,
    author: MutableVec<String>,
    genre: MutableVec<String>,
    cover_url: Mutable<Option<String>>,
    description: Mutable<Option<String>>,
    status: Mutable<Option<String>>,
    is_favorite: Mutable<bool>,
    chapters: MutableVec<Chapter>,
    loader: AsyncLoader,
}

impl Manga {
    pub fn new(id: i64, source_id: i64, path: String) -> Rc<Self> {
        Rc::new(Self {
            id: Mutable::new(id),
            source_id: Mutable::new(source_id),
            path: Mutable::new(path),
            title: Mutable::new(None),
            author: MutableVec::new(),
            genre: MutableVec::new(),
            cover_url: Mutable::new(None),
            description: Mutable::new(None),
            status: Mutable::new(None),
            is_favorite: Mutable::new(false),
            chapters: MutableVec::new(),
            loader: AsyncLoader::new(),
        })
    }

    pub fn fetch_detail(manga: Rc<Self>, refresh: bool) {
        manga.loader.load(clone!(manga => async move {
            match query::fetch_manga_detail(manga.id.get(), refresh).await {
                Ok(result) => {
                    manga.title.set_neq(Some(result.title));
                    manga.author.lock_mut().replace_cloned(result.author);
                    manga.genre.lock_mut().replace_cloned(result.genre);
                    manga.cover_url.set_neq(Some(result.cover_url));
                    manga.description.set_neq(result.description);
                    manga.status.set_neq(result.status);
                    manga.is_favorite.set_neq(result.is_favorite);
                    manga.chapters.lock_mut().replace_cloned(result.chapters.iter().map(|chapter| Chapter{
                        id: chapter.id,
                        title: chapter.title.clone(),
                        uploaded: NaiveDateTime::parse_from_str(&chapter.uploaded, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse uploaded date"),
                        read_at: Mutable::new(chapter.read_at.as_ref().map(|time| NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse read at date"))),
                        last_page_read: Mutable::new(chapter.last_page_read)
                    }).collect());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    pub fn fetch_detail_by_source_path(manga: Rc<Self>) {
        manga.loader.load(clone!(manga => async move {
            match query::fetch_manga_by_source_path(manga.source_id.get(), manga.path.get_cloned()).await {
                Ok(result) => {
                    manga.id.set_neq(result.id);
                    manga.title.set_neq(Some(result.title));
                    manga.author.lock_mut().replace_cloned(result.author);
                    manga.genre.lock_mut().replace_cloned(result.genre);
                    manga.cover_url.set_neq(Some(result.cover_url));
                    manga.description.set_neq(result.description);
                    manga.status.set_neq(result.status);
                    manga.is_favorite.set_neq(result.is_favorite);
                    manga.chapters.lock_mut().replace_cloned(result.chapters.iter().map(|chapter| Chapter{
                        id: chapter.id,
                        title: chapter.title.clone(),
                        uploaded: NaiveDateTime::parse_from_str(&chapter.uploaded, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse uploaded date"),
                        read_at: Mutable::new(chapter.read_at.as_ref().map(|time| NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse read at date"))),
                        last_page_read: Mutable::new(chapter.last_page_read)
                    }).collect());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    pub fn add_to_or_remove_from_library(manga: Rc<Self>) {
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
                                })
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
        html!("div", {
            .attribute("id", "chapters")
            .children(&mut [
                html!("span", {
                    .class("header")
                    .style("margin", "0.5rem")
                    .text("Chapters")
                }),
                html!("ul", {
                    .class("list")
                    // .class_signal("animate-pulse", manga.loader.is_loading())
                    .child_signal(manga.loader.is_loading().map(|x| {
                        if x {
                            Some(html!("div", {
                                .class(["flex", "flex-col"])
                                .children(&mut [
                                    html!("div", {
                                        .class(["w-full", "h-6", "my-2", "rounded", "bg-gray-200", "dark:bg-gray-800"])
                                    }),
                                    html!("div", {
                                        .class(["w-full", "h-6", "mb-2", "rounded", "bg-gray-200", "dark:bg-gray-800"])
                                    }),
                                    html!("div", {
                                        .class(["w-full", "h-6", "mb-2", "rounded", "bg-gray-200", "dark:bg-gray-800"])
                                    })
                                ])
                            }))
                        } else {
                            None
                        }
                    }))
                    .children_signal_vec(manga.chapters.signal_vec_cloned().map(|chapter| html!("li", {
                        .class("list-item")
                        .children(&mut [
                            link!(Route::Chapter(chapter.id, 0).url(), {
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
                                                        .children(&mut [
                                                            html!("span", {
                                                                .style("font-size", "small")
                                                                .style("font-weight", "400")
                                                                .style("margin-right", "0.5rem")
                                                                .text(&chapter.uploaded.date().to_string())
                                                            }),
                                                        ])
                                                        .child_signal(chapter.last_page_read.signal().map(|x| x.map(|page| html!("span", {
                                                            .style("font-size", "small")
                                                            .style("font-weight", "400")
                                                            .text(format!("Page: {}", page + 1).as_str())
                                                        }))))
                                                    })
                                                ])
                                            }),
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
                                                        .attribute("d", "M9 5l7 7-7 7")
                                                    })
                                                ])
                                            })
                                        ])
                                    })
                                ])
                            })
                        ])
                    })))
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
        })
    }
}
