use std::rc::Rc;

use crate::common::{events, snackbar, ReaderSettings, Spinner};
use crate::utils::{document, proxied_image_url, window, AsyncLoader};
use crate::{
    common::{Background, Direction, DisplayMode, ReaderMode},
    query,
    utils::history,
};
use dominator::{clone, html, routing, svg, with_node, Dom};
use futures_signals::signal::{Mutable, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVec, SignalVecExt};
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::HtmlImageElement;

enum Nav {
    None,
    Prev,
    Next,
}

pub struct Reader {
    chapter_id: Mutable<i64>,
    manga_id: Mutable<i64>,
    manga_title: Mutable<String>,
    chapter_title: Mutable<String>,
    next_chapter: Mutable<Option<i64>>,
    prev_chapter: Mutable<Option<i64>>,
    prev_page: Mutable<Option<usize>>,
    current_page: Mutable<usize>,
    next_page: Mutable<Option<usize>>,
    pages: MutableVec<(String, bool)>,
    pages_len: Mutable<usize>,
    reader_settings: Rc<ReaderSettings>,
    is_bar_visible: Mutable<bool>,
    loader: AsyncLoader,
    spinner: Rc<Spinner>,
}

impl Reader {
    pub fn new(chapter_id: i64, page: i64) -> Rc<Self> {
        Rc::new(Self {
            chapter_id: Mutable::new(chapter_id),
            manga_id: Mutable::new(0),
            manga_title: Mutable::new("".to_string()),
            chapter_title: Mutable::new("".to_string()),
            next_chapter: Mutable::new(None),
            prev_chapter: Mutable::new(None),
            prev_page: Mutable::new(None),
            current_page: Mutable::new(page as usize),
            next_page: Mutable::new(None),
            pages: MutableVec::new(),
            pages_len: Mutable::new(0),
            reader_settings: ReaderSettings::new(false, true),
            is_bar_visible: Mutable::new(true),
            loader: AsyncLoader::new(),
            spinner: Spinner::new_with_fullscreen(true),
        })
    }

    fn fetch_detail(reader: Rc<Self>, chapter_id: i64, nav: Nav) {
        reader.spinner.set_active(true);
        reader.loader.load(clone!(reader => async move {
            match query::fetch_chapter(chapter_id).await {
                Ok(result) => {
                    reader.manga_id.set_neq(result.manga.id);
                    reader.manga_title.set_neq(result.manga.title);
                    reader.chapter_title.set_neq(result.title);
                    reader.next_chapter.set_neq(result.next);
                    reader.prev_chapter.set_neq(result.prev);

                    let len = result.pages.len();
                    reader.pages_len.set_neq(len);

                    reader.pages.lock_mut().replace_cloned(result.pages.iter().map(|page| (page.clone(), false)).collect());

                    reader.reader_settings.load_by_manga_id(result.manga.id);

                    let page;
                    match nav {
                        Nav::None => {
                            let current_page = reader.current_page.get();
                            page = match reader.reader_settings.reader_mode.get() {
                                ReaderMode::Continous => current_page,
                                ReaderMode::Paged => {
                                    match reader.reader_settings.display_mode.get().get() {
                                        // display_mode.get() shouldn't return auto, here to satisfy compiler
                                        DisplayMode::Single | DisplayMode::Auto => current_page,
                                        DisplayMode::Double => {
                                            if current_page % 2 == 0 {
                                                current_page
                                            } else {
                                                current_page - 1
                                            }
                                        }
                                    }
                                }
                            };
                        },
                        Nav::Prev => {
                            page = match reader.reader_settings.reader_mode.get() {
                                ReaderMode::Continous => len - 1,
                                ReaderMode::Paged => {
                                    match reader.reader_settings.display_mode.get().get() {
                                        // display_mode.get() shouldn't return auto, here to satisfy compiler
                                        DisplayMode::Single | DisplayMode::Auto => len - 1,
                                        DisplayMode::Double => {
                                            if len % 2 == 0 {
                                                len - 2
                                            } else {
                                                len - 1
                                            }
                                        }
                                    }
                                }
                            };
                        },
                        Nav::Next => {
                            page = 0;
                        },
                    }

                    reader.current_page.set_neq(page);

                    Self::replace_state_with_url(chapter_id, page + 1);
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }

            reader.spinner.set_active(false);
        }));
    }

    fn replace_state_with_url(chapter_id: i64, current_page: usize) {
        if let Err(e) = history().replace_state_with_url(
            &JsValue::null(),
            "",
            Some(format!("/chapter/{}/{}", chapter_id, current_page,).as_str()),
        ) {
            let message = if let Some(msg) = e.as_string() {
                msg
            } else {
                "unknown reason".to_string()
            };

            error!("error replace_state_with_url: {}", message);
            snackbar::show(format!("error replace_state_with_url: {:?}", message));
        }
    }

    fn update_page_read(reader: Rc<Self>, page: usize) {
        let chapter_id = reader.chapter_id.get();
        AsyncLoader::new().load(async move {
            match query::update_page_read_at(chapter_id, page as i64).await {
                Ok(_) => {}
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
            Self::replace_state_with_url(chapter_id, page + 1);
        });
    }

    pub fn render_topbar(reader: Rc<Self>) -> Dom {
        html!("div", {
            .class(["animate__animated", "topbar"])
            .class_signal("animate__slideInDown", reader.is_bar_visible.signal())
            .class_signal("animate__slideOutUp", reader.is_bar_visible.signal().map(|x| !x))
            .children(&mut [
                html!("button", {
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
                        })
                    ])
                    .event(|_: events::Click| {
                        if history().length().unwrap_throw() == 0 {
                            routing::go_to_url("/");
                        } else {
                            history().back().unwrap_throw();
                        }
                    })
                }),
                html!("div", {
                    .style("display", "flex")
                    .style("flex-direction", "column")
                    .style("min-width", "0")
                    .style("width", "100%")
                    .children(&mut [
                        html!("span", {
                            .style("flex", "1")
                            .style("overflow", "hidden")
                            .style("text-overflow", "ellipsis")
                            .style("white-space", "nowrap")
                            .text_signal(reader.manga_title.signal_cloned())
                        }),
                        html!("span", {
                            .style("flex", "1")
                            .style("overflow", "hidden")
                            .style("text-overflow", "ellipsis")
                            .style("white-space", "nowrap")
                            .style("font-size", "smaller")
                            .text_signal(reader.chapter_title.signal_cloned())
                        }),
                    ])
                }),
                html!("button", {
                    .children(&mut [
                        svg!("svg", {
                            .attribute("xmlns", "http://www.w3.org/2000/svg")
                            .attribute("viewBox", "0 0 24 24")
                            .attribute("stroke", "currentColor")
                            .attribute("fill", "none")
                            .class("icon")
                            .children(&mut [
                                svg!("path", {
                                    .attribute("stroke-linecap", "round")
                                    .attribute("stroke-linejoin", "round")
                                    .attribute("stroke-width", "1")
                                    .class("heroicon-ui")
                                    .attribute("d", "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z")
                                }),
                                svg!("path", {
                                    .attribute("stroke-linecap", "round")
                                    .attribute("stroke-linejoin", "round")
                                    .attribute("stroke-width", "1")
                                    .class("heroicon-ui")
                                    .attribute("d", "M15 12a3 3 0 11-6 0 3 3 0 016 0z")
                                })
                            ])
                        })
                    ])
                    .event(clone!(reader => move |_: events::Click| {
                        reader.reader_settings.toggle_show();
                    }))
                })
            ])
        })
    }

    pub fn render_bottombar(reader: Rc<Self>) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("justify-content", "space-between")
            .style("align-items", "center")
            .style("position", "fixed")
            .style("left", "0")
            .style("right", "0")
            .style("bottom", "0")
            .style("background-color", "var(--bottombar-background-color)")
            .style("border-top-width", "1px")
            .style("border-top-style", "solid")
            .style("border-top-color", "var(--background-color-100)")
            .style("align-content", "flex-end")
            .style("padding-top", "0.5rem")
            .style("padding-bottom", "calc(env(safe-area-inset-bottom) + 0.5rem)")
            .style("color", "var(--color)")
            .style("z-index", "40")
            .class("animate__animated")
            .class_signal("animate__slideInUp", reader.is_bar_visible.signal())
            .class_signal("animate__slideOutDown", reader.is_bar_visible.signal().map(|x| !x))
            .children(&mut [
                html!("button", {
                    .attribute_signal("disabled", reader.prev_chapter.signal().map(|prev_chapter| if prev_chapter.is_none() {Some("true")} else {None}))
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
                                    .attribute("d", "M11 17l-5-5m0 0l5-5m-5 5h12")
                                })
                            ])
                        })
                    ])
                    .event(clone!(reader => move |_: events::Click| {
                       if let Some(prev) = reader.prev_chapter.get() {
                           reader.chapter_id.set(prev);
                       }
                    }))
                }),
                html!("button", {
                    .attribute_signal("disabled", reader.next_chapter.signal().map(|next_chapter| if next_chapter.is_none() {Some("true")} else {None}))
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
                                    .attribute("d", "M13 7l5 5m0 0l-5 5m5-5H6")
                                })
                            ])
                        })
                    ])
                    .event(clone!(reader => move |_: events::Click| {
                       if let Some(next) = reader.next_chapter.get() {
                        reader.chapter_id.set(next);
                       }
                    }))
                })
            ])
        })
    }

    pub fn render_page_indicator(reader: Rc<Self>) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("justify-content", "center")
            .style("align-items", "center")
            .style("position", "fixed")
            .style("left", "50%")
            .style("right", "50%")
            .style("bottom", "0")
            .style("background-color", "transparent")
            .style("padding-top", "0.5rem")
            .style("padding-bottom", "calc(env(safe-area-inset-bottom) + 0.5rem)")
            .style("z-index", "50")
            .children(&mut [
                html!("div", {
                    .style("border-radius", "10%")
                    .style("padding", "0.5rem")
                    .style_signal("background-color", reader.is_bar_visible.signal().map(|visible| if visible {None} else {Some("rgba(0, 0, 0, 0.5)")}))
                    .style_signal("color", reader.is_bar_visible.signal().map(|visible| if visible {None} else {Some("white")}))
                    .children(&mut [
                        html!("span", {
                            .text_signal(reader.current_page.signal().map(|p| (p + 1).to_string()))
                        }),
                        html!("span", {
                            .text("/")
                        }),
                        html!("span", {
                            .text_signal(reader.pages_len.signal().map(|len| len.to_string()))
                        }),
                    ])
                }),
            ])
        })
    }

    fn render_navigation(reader: Rc<Self>) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("position", "fixed")
            .style("width", "100vw")
            .style("height", "100vh")
            .style("z-index", "10")
            .style("cursor", "pointer")
            .style_signal("flex-direction", reader.reader_settings.direction.signal_cloned().map(|x| match x {
                Direction::LeftToRight => "row-reverse",
                Direction::RightToLeft => "row",
            }))
            .children(&mut [
                html!("div", {
                    .style("height", "100%")
                    .style("width", "33.3333%")
                    .attribute("id", "next")
                    .event(clone!(reader => move |_: events::Click| {
                        if let Some(next_page) = reader.next_page.get() {
                            reader.current_page.set_neq(next_page);
                        } else if let Some(next_chapter) = reader.next_chapter.get() {
                            reader.chapter_id.set(next_chapter);
                        } else {
                            info!("no next_page or next_chapter");
                        }
                    }))
                }),
                html!("div", {
                    .style("height", "100%")
                    .style("width", "33.3333%")
                    .attribute("id", "hide-bar")
                    .event(clone!(reader => move |_: events::Click| {
                        reader.is_bar_visible.set_neq(!reader.is_bar_visible.get());
                    }))
                }),
                html!("div", {
                    .style("height", "100%")
                    .style("width", "33.3333%")
                    .attribute("id", "prev")
                    .event(clone!(reader => move |_: events::Click| {
                        if let Some(prev_page) = reader.prev_page.get() {
                            reader.current_page.set_neq(prev_page);
                        } else if let Some(prev_chapter) = reader.prev_chapter.get() {
                            reader.chapter_id.set(prev_chapter);
                        } else {
                            info!("no prev_page or prev_chapter");
                        }
                    }))
                })
            ])
        })
    }

    pub fn pages_signal(&self) -> impl SignalVec<Item = (usize, (String, bool))> {
        self.pages
            .signal_vec_cloned()
            .enumerate()
            .filter_map(|(index, page)| {
                if let Some(index) = index.get() {
                    Some((index, page))
                } else {
                    None
                }
            })
            .to_signal_cloned()
            .to_signal_vec()
    }

    pub fn render_vertical(reader: Rc<Self>) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("flex-direction", "column")
            .children(&mut [
                html!("button", {
                    .style("width", "100%")
                    .style("height", "5rem")
                    .style("border-width", "2px")
                    .style("border-style", "dashed")
                    .style("margin-top", "env(safe-area-inset-top)")
                    .attribute_signal("disabled", reader.prev_chapter.signal().map(|prev_chapter| if prev_chapter.is_some() { None } else { Some("true") }))
                    .text_signal(reader.prev_chapter.signal().map(|prev_chapter| if prev_chapter.is_some() { "Prev Chapter" } else { "No Prev Chapter" }))
                    .event(clone!(reader => move |_: events::Click| {
                        if let Some(prev_chapter) = reader.prev_chapter.get() {
                            reader.chapter_id.set(prev_chapter);
                        } else {
                            info!("no prev_page or prev_chapter");
                        }
                    }))
                })
            ])
            .children_signal_vec(reader.pages_signal().map(clone!(reader => move |(index, (page, error))|
                if !error {
                    html!("img" => HtmlImageElement, {
                        .style("margin-left", "auto")
                        .style("margin-right", "auto")
                        .attribute("id", format!("page-{}", index).as_str())
                        .attribute("src", &proxied_image_url(&page))
                        .style_signal("max-width", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Height => "none",
                            _ => "768px",
                        }))
                        .style_signal("object-fit", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::All => "contain",
                            _ => "initial",
                        }))
                        .style_signal("width", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Height =>"initial",
                            _ => "100vw"
                        }))
                        .style_signal("height", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Width => "initial",
                            _ => "100vh"
                        }))
                        .event(clone!(reader, page => move |_: events::Error| {
                            log::error!("error loading image");
                            let mut lock = reader.pages.lock_mut();
                            lock.set_cloned(index, (page.clone(), true));
                        }))
                        .event(clone!(reader => move |_: events::Click| {
                            reader.is_bar_visible.set_neq(!reader.is_bar_visible.get());
                        }))
                    })
                } else {
                    html!("div", {
                        .attribute("id", index.to_string().as_str())
                        .style("display", "flex")
                        .style("height", "calc(100vw * 1.59)")
                        .children(&mut [
                            html!("button", {
                                .style("margin", "auto")
                                .text("Retry")
                                .event(clone!(reader, page => move |_: events::Click| {
                                    log::info!("retry loading image");
                                    let mut lock = reader.pages.lock_mut();
                                    lock.set_cloned(index, (page.clone(), false));
                                }))
                            })
                        ])
                    })
                }
            )))
            .children(&mut [
                html!("button", {
                    .style("width", "100%")
                    .style("height", "5rem")
                    .style("border-width", "2px")
                    .style("border-style", "dashed")
                    .style("margin-bottom", "env(safe-area-inset-bottom)")
                    .attribute_signal("disabled", reader.next_chapter.signal().map(|next_chapter| if next_chapter.is_some() { None } else { Some("true") }))
                    .text_signal(reader.next_chapter.signal().map(|next_chapter| if next_chapter.is_some() { "Next Chapter" } else { "No Next Chapter" }))
                    .event(clone!(reader => move |_: events::Click| {
                        if let Some(next_chapter) = reader.next_chapter.get() {
                            reader.chapter_id.set(next_chapter);
                        } else {
                            info!("no next_page or next_chapter");
                        }
                    }))
                })
            ])
            .global_event(clone!(reader => move |_: events::Scroll| {
                let mut page_no = 0;
                let body_top = window().scroll_y().unwrap_throw();
                for i in 0..reader.pages_len.get() {
                    let page_top = document()
                        .get_element_by_id(format!("page-{}", i).as_str())
                        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                        .map(|el| el.offset_top() as f64)
                        .unwrap_or_default();
                    if page_top > body_top {
                        page_no = i;
                        break;
                    }
                }
                let is_last_page = reader.pages_len.get() == reader.current_page.get() + 1;
                info!("is_last_page: {} page_no: {}", is_last_page, page_no);
                if !(is_last_page && page_no == 0) {
                    reader.current_page.set_neq(page_no as usize);
                }
            }))
        })
    }

    pub fn render_single(reader: Rc<Self>) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("align-items", "center")
            .children_signal_vec(reader.pages_signal().map(clone!(reader => move |(index, (page, error))|
                if !error {
                    html!("img", {
                        .style("margin-left", "auto")
                        .style("margin-right", "auto")
                        .style_signal("max-width", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Height => "none",
                            _ => "100%",
                        }))
                        .style_signal("object-fit", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::All => "contain",
                            _ => "initial",
                        }))
                        .style_signal("width", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Height => "initial",
                            _ => "100vw"
                        }))
                        .style_signal("height", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Width => "initial",
                            _ => "100vh"
                        }))
                        .visible_signal(reader.current_page.signal_cloned().map(clone!(reader => move |x| {
                            reader.prev_page.set_neq(x.checked_sub(1));
                            if x + 1 < reader.pages_len.get() {
                                reader.next_page.set_neq(Some(x + 1));
                            }

                            x == index
                        })))
                        .attribute("src", &proxied_image_url(&page))
                        .event(clone!(reader, page => move |_: events::Error| {
                            log::error!("error loading image");
                            let mut lock = reader.pages.lock_mut();
                            lock.set_cloned(index, (page.clone(), true));
                        }))
                    })
                } else {
                    html!("div", {
                        .attribute("id", index.to_string().as_str())
                        .style("display", "flex")
                        .style("height", "100vh")
                        .style("width", "100vw")
                        .visible_signal(reader.current_page.signal_cloned().map(clone!(reader => move |x| {
                            reader.prev_page.set_neq(x.checked_sub(1));
                            if x + 1 < reader.pages_len.get() {
                                reader.next_page.set_neq(Some(x + 1));
                            }

                            x == index
                        })))
                        .children(&mut [
                            html!("button", {
                                .style("margin", "auto")
                                .style("z-index", "20")
                                .text("Retry")
                                .event(clone!(reader, page => move |_: events::Click| {
                                    log::info!("retry loading image");
                                    let mut lock = reader.pages.lock_mut();
                                    lock.set_cloned(index, (page.clone(), false));
                                }))
                            })
                        ])
                    })
                }
            )))
        })
    }

    pub fn render_double(reader: Rc<Self>) -> Dom {
        html!("div", {
            .attribute("id", "page-list")
            .style("display", "flex")
            .style("width", "100vw")
            .style("height", "100vh")
            .style("align-items", "center")
            .style_signal("flex-direction", reader.reader_settings.direction.signal_cloned().map(|x| match x {
                Direction::LeftToRight => "row",
                Direction::RightToLeft => "row-reverse",
            }))
            .children_signal_vec(reader.pages_signal().map(clone!(reader => move |(index, (page, error))|
                if !error {
                    html!("img" => HtmlImageElement, {
                        .class([
                            "mx-auto"
                        ])
                        .attribute("id", format!("page-{}", index).as_str())
                        .style_signal("max-width", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Height => "none",
                            _ => "100%",
                        }))
                        .style_signal("object-fit", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::All => "contain",
                            _ => "initial",
                        }))
                        .style_signal("height", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Width => "initial",
                            _ => "100%"
                        }))
                        .attribute("src", &proxied_image_url(&page))
                        .event(clone!(reader, page => move |_: events::Error| {
                            log::error!("error loading image");
                            let mut lock = reader.pages.lock_mut();
                            lock.set_cloned(index, (page.clone(), true));
                        }))
                        .with_node!(img => {
                            .style_signal("width", reader.current_page.signal_cloned().map(clone!(reader, index, img => move |current_page|
                                if (index == current_page && img.natural_width() > img.natural_height())
                                    || matches!(reader.reader_settings.fit.get(), crate::common::Fit::Height) {
                                    "initial"
                                } else {
                                    "50%"
                                }
                            )))
                            .visible_signal(reader.current_page.signal_cloned().map(clone!(reader, index, img => move |current_page| {
                                let mut hidden = true;
                                if index == current_page {
                                    hidden = false;
                                    if current_page > 0 {
                                        let is_prev_img_landscape = if let Some(prev_img) = document().get_element_by_id(format!("page-{}", current_page - 1).as_str()).and_then(|el| el.dyn_into::<web_sys::HtmlImageElement>().ok()) {
                                            prev_img.natural_width() > prev_img.natural_height()
                                        } else {
                                            false
                                        };
                                        let sub = if is_prev_img_landscape || current_page == 1 {
                                            1
                                        } else {
                                            2
                                        };
                                        reader.prev_page.set_neq(current_page.checked_sub(sub));
                                    }
                                } else if index == current_page + 1 {
                                    let is_prev_img_portrait = if let Some(prev_img) = document().get_element_by_id(format!("page-{}", current_page).as_str()).and_then(|el| el.dyn_into::<web_sys::HtmlImageElement>().ok()) {
                                        prev_img.natural_width() <= prev_img.natural_height()
                                    } else {
                                        true
                                    };

                                    if img.natural_width() < img.natural_height() && is_prev_img_portrait {
                                        hidden = false;
                                        if current_page + 2 < reader.pages_len.get() {
                                            reader.next_page.set_neq(Some(current_page + 2));
                                        } else {
                                            reader.next_page.set_neq(None);
                                        }
                                    } else if current_page + 1 < reader.pages_len.get() {
                                        reader.next_page.set_neq(Some(current_page + 1));
                                    } else {
                                        reader.next_page.set_neq(None);
                                    }
                                }

                                !hidden
                            })))
                            .event(clone!(reader, index => move |_: events::Load| {
                                let current_page = reader.current_page.get();
                                if index == current_page || index == current_page + 1 {
                                    reader.current_page.replace(current_page);
                                }
                            }))
                        })
                    })
                } else {
                    html!("div", {
                        .attribute("id", format!("page-{}", index).as_str())
                        .style("display", "flex")
                        .style_signal("width", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Height => "none",
                            _ => "100%",
                        }))
                        .style_signal("height", reader.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Width => "initial",
                            _ => "100%"
                        }))
                        .visible_signal(reader.current_page.signal_cloned().map(clone!(reader, index => move |current_page| {
                            let mut hidden = true;
                            if index == current_page {
                                hidden = false;
                                if current_page > 0 {
                                    let sub = if current_page == 1 {
                                        1
                                    } else {
                                        2
                                    };
                                    reader.prev_page.set_neq(current_page.checked_sub(sub));
                                }
                            } else if index == current_page + 1 {
                                hidden = false;
                                if current_page + 2 < reader.pages_len.get() {
                                    reader.next_page.set_neq(Some(current_page + 2));
                                } else {
                                    reader.next_page.set_neq(None);
                                }
                            }

                            !hidden
                        })))
                        .children(&mut [
                            html!("button", {
                                .style("margin", "auto")
                                .style("z-index", "20")
                                .text("Retry")
                                .event(clone!(reader, page => move |_: events::Click| {
                                    log::info!("retry loading image");
                                    let mut lock = reader.pages.lock_mut();
                                    lock.set_cloned(index, (page.clone(), false));
                                }))
                            })
                        ])
                    })
                }
            )))
        })
    }

    pub fn render(reader: Rc<Self>) -> Dom {
        html!("div", {
            .class("reader")
            .future(reader.current_page.signal().for_each(clone!(reader => move |page| {
                // just opening a chapter shouldn't be considered as reading
                if page > 0 {
                    let page = match reader.reader_settings.reader_mode.get() {
                        ReaderMode::Continous => page,
                        ReaderMode::Paged => {
                            match reader.reader_settings.display_mode.get().get() {
                                // display_mode.get() shouldn't return auto, here to satisfy compiler
                                DisplayMode::Single | DisplayMode::Auto => page,
                                DisplayMode::Double => {
                                    if page + 2 == reader.pages_len.get() {
                                        page + 1
                                    } else {
                                        page
                                    }
                                }
                            }
                        }
                    };

                    Self::update_page_read(reader.clone(), page);
                }

                if page == 0 {
                    reader.prev_page.set(None);
                } else if page + 1 == reader.pages_len.get() {
                    reader.next_page.set(None);
                }

                async {}
            })))
            .future(reader.chapter_id.signal().for_each(clone!(reader => move |chapter_id| {
                let nav = match chapter_id {
                    _ if Some(chapter_id) == reader.prev_chapter.get() => Nav::Prev,
                    _ if Some(chapter_id) == reader.next_chapter.get() => Nav::Next,
                    _ => Nav::None,
                };

                Self::fetch_detail(reader.clone(), chapter_id, nav);

                async {}
            })))
            .global_event(clone!(reader => move |_:events::Resize| reader.reader_settings.display_mode.set(reader.reader_settings.display_mode.get())))
            .class_signal("dark", reader.reader_settings.background.signal_cloned().map(|x| match x {
                Background::White => false,
                Background::Black => true,
            }))
            .children(&mut [
                Self::render_topbar(reader.clone()),
            ])
            .child_signal(reader.reader_settings.reader_mode.signal_cloned().map(clone!(reader => move |x| match x {
                ReaderMode::Continous => Some(Self::render_vertical(reader.clone())),
                ReaderMode::Paged => Some(html!("div", {
                    .children(&mut [
                        Self::render_navigation(reader.clone())
                    ])
                    .child_signal(reader.reader_settings.display_mode.signal_cloned().map(clone!(reader => move |x| match x.get() {
                        DisplayMode::Single => Some(Self::render_single(reader.clone())),
                        DisplayMode::Double => Some(Self::render_double(reader.clone())),
                        DisplayMode::Auto => None // shouldn't return this
                    })))
                }))
            })))
            .children(&mut [
                Self::render_page_indicator(reader.clone()),
                Self::render_bottombar(reader.clone()),
                ReaderSettings::render(reader.reader_settings.clone()),
                Spinner::render(&reader.spinner)
            ])
        })
    }
}
