use std::rc::Rc;

use crate::common::{Fit, ReaderSettings, Spinner, events, snackbar};
use crate::utils::{document, proxied_image_url, window, AsyncLoader, body};
use crate::{
    common::{Background, Direction, DisplayMode, ReaderMode},
    query,
    utils::history,
};
use dominator::{Dom, EventOptions, clone, html, routing, svg, with_node};
use futures_signals::map_ref;
use futures_signals::signal::{self, Mutable, Signal, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVec, SignalVecExt};
use gloo_timers::callback::Timeout;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlImageElement, HtmlInputElement};

#[derive(Debug)]
enum Nav {
    None,
    Prev,
    Next,
}

#[derive(Debug, Clone, Copy)]
enum PageStatus {
    Initial,
    Loaded,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContinousLoaded {
    Initial,
    Loaded,
    Scrolled
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
    pages: MutableVec<(String, PageStatus)>,
    pages_loaded: Mutable<ContinousLoaded>,
    reader_settings: Rc<ReaderSettings>,
    zoom: Mutable<f64>,
    is_bar_visible: Mutable<bool>,
    loader: Rc<AsyncLoader>,
    spinner: Rc<Spinner>,
    timeout: Mutable<Option<Timeout>>
}

impl Reader {
    pub fn new(chapter_id: i64, page: i64) -> Rc<Self> {
        let loader = Rc::new(AsyncLoader::new());
        let spinner = Spinner::new_with_fullscreen_and_callback(true, clone!(loader => move || {
            loader.cancel();
        }));

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
            pages_loaded: Mutable::new(ContinousLoaded::Initial),
            reader_settings: ReaderSettings::new(false, true),
            zoom: Mutable::new(1.0),
            is_bar_visible: Mutable::new(true),
            loader,
            spinner,
            timeout: Mutable::new(None),
        })
    }

    fn fetch_detail(this: Rc<Self>, chapter_id: i64, nav: Nav) {
        let current_page = this.current_page.get_cloned();
        this.spinner.set_active(true);
        this.loader.load(clone!(this => async move {
            match query::fetch_chapter(chapter_id).await {
                Ok(result) => {
                    this.manga_id.set_neq(result.manga.id);
                    this.manga_title.set_neq(result.manga.title);
                    this.chapter_title.set_neq(result.title);
                    this.next_chapter.set_neq(result.next);
                    this.prev_chapter.set_neq(result.prev);

                    this.reader_settings.load_by_manga_id(result.manga.id);

                    let page;
                    match nav {
                        Nav::None => {
                            info!("get current_page {}", current_page);
                            page = match this.reader_settings.reader_mode.get() {
                                ReaderMode::Continous => current_page,
                                ReaderMode::Paged => {
                                    match this.reader_settings.display_mode.get().get() {
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
                            let len = this.pages.lock_ref().len();
                            page = match this.reader_settings.reader_mode.get() {
                                ReaderMode::Continous => len - 1,
                                ReaderMode::Paged => {
                                    match this.reader_settings.display_mode.get().get() {
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

                    info!("set current_page to {} nav: {:?}", page, nav);
                    this.current_page.set_neq(page);

                    this.pages_loaded.set(ContinousLoaded::Initial);

                    let source_url = result.source.url;
                    let pages = result.pages.iter().map(|page| (format!("{}?referer={}", page, source_url), PageStatus::Initial)).collect();
                    this.pages.lock_mut().replace_cloned(pages);
                    
                    Self::replace_state_with_url(chapter_id, page + 1);
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
            
            this.spinner.set_active(false);
        }));
    }

    fn replace_state_with_url(chapter_id: i64, current_page: usize) {
        if let Err(e) = history().replace_state_with_url(
            &JsValue::null(),
            "",
            Some(format!("/chapter/{}#{}", chapter_id, current_page,).as_str()),
        ) {
            let message = if let Some(msg) = e.as_string() {
                msg
            } else {
                "unknown reason".to_string()
            };

            error!("error replace_state_with_url: {}", message);
        }
    }

    fn update_page_read(this: Rc<Self>, page: usize) {
        let chapter_id = this.chapter_id.get();
        
        let pages_len = this.pages.lock_ref().len();
        let page = if matches!(this.reader_settings.reader_mode.get(), ReaderMode::Paged) && matches!(this.reader_settings.display_mode.get().get(), DisplayMode::Double) && page + 2 == pages_len {
            page + 1
        } else {
            page
        };
        
        let is_complete = page + 1 == pages_len;

        Self::replace_state_with_url(chapter_id, page + 1);
        

        let timeout = Timeout::new(500, move || {
            spawn_local(async move {
                match query::update_page_read_at(chapter_id, page as i64, is_complete).await {
                    Ok(_) => {}
                    Err(err) => {
                        snackbar::show(format!("{}", err));
                    }
                }
            });
        });
            
        this.timeout.set(Some(timeout));
    }

    pub fn render_topbar(this: Rc<Self>) -> Dom {
        html!("div", {
            .class("topbar")
            .class("animate__animated")
            .class("animate__faster")
            .class_signal("animate__slideInDown", this.is_bar_visible.signal())
            .class_signal("animate__slideOutUp", this.is_bar_visible.signal().map(|x| !x))
            .children(&mut [
                html!("button", {
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
                            .text_signal(this.manga_title.signal_cloned())
                        }),
                        html!("span", {
                            .style("flex", "1")
                            .style("overflow", "hidden")
                            .style("text-overflow", "ellipsis")
                            .style("white-space", "nowrap")
                            .style("font-size", "smaller")
                            .text_signal(this.chapter_title.signal_cloned())
                        }),
                    ])
                }),
                html!("button", {
                    .children(&mut [
                        svg!("svg", {
                            .attr("xmlns", "http://www.w3.org/2000/svg")
                            .attr("viewBox", "0 0 24 24")
                            .attr("stroke", "currentColor")
                            .attr("fill", "none")
                            .class("icon")
                            .children(&mut [
                                svg!("path", {
                                    .attr("stroke-linecap", "round")
                                    .attr("stroke-linejoin", "round")
                                    .attr("stroke-width", "1")
                                    .class("heroicon-ui")
                                    .attr("d", "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z")
                                }),
                                svg!("path", {
                                    .attr("stroke-linecap", "round")
                                    .attr("stroke-linejoin", "round")
                                    .attr("stroke-width", "1")
                                    .class("heroicon-ui")
                                    .attr("d", "M15 12a3 3 0 11-6 0 3 3 0 016 0z")
                                })
                            ])
                        })
                    ])
                    .event(clone!(this => move |_: events::Click| {
                        this.reader_settings.toggle_show();
                    }))
                })
            ])
        })
    }

    pub fn render_bottombar(this: Rc<Self>) -> Dom {
        html!("div", {
            .style("position", "fixed")
            .style("left", "0")
            .style("right", "0")
            .style("bottom", "0")
            .style("z-index", "40")
            .class("animate__animated")
            .class("animate__faster")
            .class_signal("animate__slideInUp", this.is_bar_visible.signal())
            .class_signal("animate__slideOutDown", this.is_bar_visible.signal().map(|x| !x))
            .children(&mut [
                Self::render_page_slider(this.clone()),
                Self::render_action_bar(this)
            ])
        })
    }
    
    pub fn render_page_slider(this: Rc<Self>) -> Dom {
        html!("div", {
            .style("padding-left", "0.125rem")
            .style("padding-right", "0.125rem")
            .style("padding-bottom", "0.5rem")
            .children(&mut [
                html!("div", {
                    .style("display", "flex")
                    .style("height", "2.25rem")
                    .style("padding-top", "0.25rem")
                    .style("padding-bottom", "0.25rem")
                    .style("justify-content", "space-between")
                    .style("align-items", "center")
                    .style("color", "var(--color)")
                    .style("align-content", "flex-end")
                    .style("border-radius", "5rem")
                    .style("border-top-width", "1px")
                    .style("border-top-style", "solid")
                    .style("border-top-color", "var(--background-color-100)")
                    .style("border-bottom-width", "1px")
                    .style("border-bottom-style", "solid")
                    .style("border-bottom-color", "var(--background-color-100)")
                    .style("border-left-width", "1px")
                    .style("border-left-style", "solid")
                    .style("border-left-color", "var(--background-color-100)")
                    .style("border-right-width", "1px")
                    .style("border-right-style", "solid")
                    .style("border-right-color", "var(--background-color-100)")
                    .style("background-color", "var(--bottombar-background-color)")
                    .style_signal("direction", this.reader_settings.direction.signal().map(|direction| matches!(direction, Direction::RightToLeft).then(|| "rtl")))
                    .children(&mut [
                        html!("button", {
                            .attr("id", "prev-chapter-btn")
                            .attr_signal("disabled", this.prev_chapter.signal().map(|prev_chapter| if prev_chapter.is_none() {Some("true")} else {None}))
                            .child_signal(this.reader_settings.reader_direction_signal().map(|mode| {
                                match mode {
                                    (ReaderMode::Paged, Direction::RightToLeft) => Some(svg!("svg", {
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
                                                .attr("d", "M13 7l5 5m0 0l-5 5m5-5H6")
                                            })
                                        ])
                                    })),
                                    _ => Some(svg!("svg", {
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
                                                .attr("d", "M11 17l-5-5m0 0l5-5m-5 5h12")
                                            })
                                        ])
                                    }))
                                }
                            }))
                            .event(clone!(this => move |_: events::Click| {
                               if let Some(prev) = this.prev_chapter.get() {
                                   this.chapter_id.set(prev);
                               }
                            }))
                        }),
                        html!("span", {
                            .text_signal(this.current_page.signal().map(|p| (p + 1).to_string()))
                        }),
                        html!("div", {
                            .style("width", "100%")
                            .style("display", "flex")
                            .style("margin", "0.5rem")
                            .children(&mut [
                                html!("input" => HtmlInputElement, {
                                    .style("width", "100%")
                                    .attr("type", "range")
                                    .attr("min", "0")
                                    .attr_signal("max", this.pages.signal_vec_cloned().len().map(|len| (len.saturating_sub(1)).to_string()))
                                    .attr_signal("value", this.current_page.signal().map(|p| p.to_string()))
                                    .with_node!(input => {
                                        .event(clone!(this, input => move |_: events::Change| {
                                            let page = input.value().parse().unwrap_or(0);
                                            info!("page: {}", page);
                                            if matches!(this.reader_settings.reader_mode.get(), ReaderMode::Continous) {
                                                let page_top =  document()
                                                    .get_element_by_id(format!("{}", page - 1).as_str())
                                                    .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                                                    .map(|el| el.offset_top() as f64)
                                                    .unwrap_or_default();
        
                                                info!("scroll to {}", page_top);
                                                window().scroll_to_with_x_and_y(0.0_f64, page_top);
                                            }
                                            this.current_page.set(page);
                                        }))
                                    })
                                }),
                            ])
                        }),
                        html!("span", {
                            .text_signal(this.pages.signal_vec_cloned().len().map(|len| len.to_string()))
                        }),
                        html!("button", {
                            .attr("id", "next-chapter-btn")
                            .style("border-radius", "100%")
                            .attr_signal("disabled", this.next_chapter.signal().map(|next_chapter| if next_chapter.is_none() {Some("true")} else {None}))
                            .child_signal(this.reader_settings.reader_direction_signal().map(|mode| {
                                match mode {
                                    (ReaderMode::Paged, Direction::RightToLeft) => Some(svg!("svg", {
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
                                                .attr("d", "M11 17l-5-5m0 0l5-5m-5 5h12")
                                            })
                                        ])
                                    })),
                                    _ => Some(svg!("svg", {
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
                                                .attr("d", "M13 7l5 5m0 0l-5 5m5-5H6")
                                            })
                                        ])
                                    }))
                                }
                            }))
                            .event(clone!(this => move |_: events::Click| {
                               if let Some(next) = this.next_chapter.get() {
                                this.chapter_id.set(next);
                                if matches!(this.reader_settings.reader_mode.get(), ReaderMode::Continous) {
                                    window().scroll_to_with_x_and_y(0.0_f64, 0.0_f64);
                                }
                               }
                            }))
                        })
                    ])
                })
            ])
        })
    }

    pub fn render_action_bar(this: Rc<Self>) -> Dom {
        html!("div", {
            .style("left", "0")
            .style("right", "0")
            .style("bottom", "0")
            .style("z-index", "40")
            .style("display", "flex")
            .style("width", "100%")
            .style("justify-content", "space-around")
            .style("align-items", "center")
            .style("background-color", "var(--bottombar-background-color)")
            .style("color", "var(--color)")
            .style("border-top-width", "1px")
            .style("border-top-style", "solid")
            .style("border-top-color", "var(--background-color-100)")
            .style("align-content", "flex-end")
            .style("padding-top", "0.25rem")
            .style("padding-bottom", "calc(env(safe-area-inset-bottom) + 0.25rem)")
            .children(&mut [
                html!("button", {
                    .attr("id", "zoom-in")
                    .style("margin-top", "0.5rem")
                    .style("margin-bottom", "0.25rem")
                    .style("text-align", "center")
                    .event(clone!(this => move |_: events::Click| {
                        info!("zoom in");
                        this.zoom.set_neq(this.zoom.get() + 0.5);   
                    }))
                    .children(&mut [
                        svg!("svg", {
                            .attr("xmlns", "http://www.w3.org/2000/svg")
                            .attr("width", "20px")
                            .attr("height", "20px")
                            .attr("viewBox", "0 0 20 20")
                            .attr("fill", "currentColor")
                            .children(&mut [
                                svg!("path", {
                                    .attr("d", "M5 8a1 1 0 011-1h1V6a1 1 0 012 0v1h1a1 1 0 110 2H9v1a1 1 0 11-2 0V9H6a1 1 0 01-1-1z")
                                }),
                                svg!("path", {
                                    .attr("fill-rule", "evenodd")
                                    .attr("clip-rule", "evenodd")
                                    .attr("d", "M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z")
                                })
                            ])
                        })
                    ])
                }),
                html!("span", {
                    .style("margin-top", "0.25rem")
                    .style("margin-bottom", "0.25rem")
                    .style("font-size", "smaller")
                    .text_signal(this.zoom.signal().map(|zoom| format!("{}%", 100.0 * zoom)))
                }),
                html!("button", {
                    .attr("id", "zoom-out")
                    .style("margin-top", "0.25rem")
                    .style("margin-bottom", "0.5rem")
                    .style("text-align", "center")
                    .event(clone!(this => move |_: events::Click| {
                        info!("zoom out");
                        let zoom = this.zoom.get();
                        if zoom > 0.0 {
                            this.zoom.set_neq(this.zoom.get() - 0.5);   
                        }
                    }))
                    .children(&mut [
                        svg!("svg", {
                            .attr("xmlns", "http://www.w3.org/2000/svg")
                            .attr("width", "20px")
                            .attr("height", "20px")
                            .attr("viewBox", "0 0 20 20")
                            .attr("fill", "currentColor")
                            .children(&mut [
                                svg!("path", {
                                    .attr("fill-rule", "evenodd")
                                    .attr("clip-rule", "evenodd")
                                    .attr("d", "M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z")
                                }),
                                svg!("path", {
                                    .attr("fill-rule", "evenodd")
                                    .attr("clip-rule", "evenodd")
                                    .attr("d", "M5 8a1 1 0 011-1h4a1 1 0 110 2H6a1 1 0 01-1-1z")
                                })
                            ])
                        })
                    ])
                }),
            ])
        })
    }

    pub fn render_page_indicator(this: Rc<Self>) -> Dom {
        html!("div", {
            .visible_signal(this.is_bar_visible.signal().map(|visible| !visible))
            .style("display", "flex")
            .style("justify-content", "center")
            .style("align-items", "center")
            .style("position", "fixed")
            .style("left", "50%")
            .style("right", "50%")
            .style("bottom", "0")
            .style("background-color", "transparent")
            .style("z-index", "50")
            .style("padding-top", "0.5rem")
            .style("padding-bottom", "env(safe-area-inset-bottom)")
            .children(&mut [
                html!("div", {
                    .style("border-radius", "10%")
                    .style("color", "white")
                    .style("font-weight", "bold")
                    .style("-webkit-text-fill-color", "white")
                    .style("-webkit-text-stroke-width", "1px")
                    .style("-webkit-text-stroke-color", "black")
                    .children(&mut [
                        html!("span", {
                            .text_signal(this.current_page.signal().map(|p| (p + 1).to_string()))
                        }),
                        html!("span", {
                            .text("/")
                        }),
                        html!("span", {
                            .text_signal(this.pages.signal_vec_cloned().len().map(|len| len.to_string()))
                        }),
                    ])
                }),
            ])
        })
    }

    fn go_to_next_page(&self) {
        if let Some(next_page) = self.next_page.get() {
            self.current_page.set_neq(next_page);
        } else if let Some(next_chapter) = self.next_chapter.get() {
            self.chapter_id.set(next_chapter);
        } else {
            info!("no next_page or next_chapter");
        }
    }

    fn go_to_prev_page(&self) {
        if let Some(prev_page) = self.prev_page.get() {
            self.current_page.set_neq(prev_page);
        } else if let Some(prev_chapter) = self.prev_chapter.get() {
            self.chapter_id.set(prev_chapter);
        } else {
            info!("no prev_page or prev_chapter");
        }
    }

    fn render_navigation(this: Rc<Self>) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("position", "fixed")
            .style("width", "100vw")
            .style("height", "100vh")
            .style("z-index", "10")
            .style("cursor", "pointer")
            .style_signal("flex-direction", this.reader_settings.direction.signal_cloned().map(|x| match x {
                Direction::LeftToRight => "row-reverse",
                Direction::RightToLeft => "row",
            }))
            .global_event(clone!(this => move |e: events::KeyDown| {
                let direction = this.reader_settings.direction.get();
                if e.key() == "ArrowLeft" {
                    match direction {
                        Direction::LeftToRight => this.go_to_prev_page(),
                        Direction::RightToLeft => this.go_to_next_page(),
                    }
                } else if e.key() == " " {
                    this.is_bar_visible.set_neq(!this.is_bar_visible.get());
                } else if e.key() == "ArrowRight" {
                    match direction {
                        Direction::LeftToRight => this.go_to_next_page(),
                        Direction::RightToLeft => this.go_to_prev_page(),
                    }
                }
            }))
            .children(&mut [
                html!("div", {
                    .style("height", "100%")
                    .style("width", "33.3333%")
                    .attr("id", "next")
                    .event(clone!(this => move |_: events::Click| {
                        this.go_to_next_page();
                    }))
                }),
                html!("div", {
                    .style("height", "100%")
                    .style("width", "33.3333%")
                    .attr("id", "hide-bar")
                    .event(clone!(this => move |_: events::Click| {
                        this.is_bar_visible.set_neq(!this.is_bar_visible.get());
                    }))
                }),
                html!("div", {
                    .style("height", "100%")
                    .style("width", "33.3333%")
                    .attr("id", "prev")
                    .event(clone!(this => move |_: events::Click| {
                        this.go_to_prev_page();
                    }))
                })
            ])
        })
    }

    fn pages_signal(&self) -> impl SignalVec<Item = (usize, String, PageStatus)> {
        self.pages
            .signal_vec_cloned()
            .enumerate()
            .filter_map(move |(index, (page, status))| index.get().map(|index| (index, page, status)))
            .to_signal_cloned()
            .to_signal_vec()
    }

    fn image_src_signal(&self, index: usize, preload_prev: usize, preload_next: usize, page: String, status: PageStatus)-> impl Signal<Item = Option<String>> {
        self.current_page.signal_cloned().map(move |current_page| {
            if (index >= current_page.saturating_sub(preload_prev) && index <= current_page + preload_next) || matches!(status, PageStatus::Loaded) {
                Some(proxied_image_url(&page))
            } else {
                None
            }
        })
    }

    fn fit_signal(&self)-> impl Signal<Item = (Fit, f64)> {
        map_ref!{
            let fit = self.reader_settings.fit.signal(),
            let zoom = self.zoom.signal() => {
            
                (*fit, *zoom)
        }
    }
    }

    fn render_vertical(this: Rc<Self>) -> Dom {
        html!("div", {
            .attr("id", "page-list")
            .style("display", "flex")
            .style("flex-direction", "column")
            .future(this.pages_loaded.signal_cloned().for_each(clone!(this => move |loaded| {
                let page = this.current_page.get();
                info!("page: {} loaded: {:?}", page, loaded);
                if page > 0 && matches!(loaded, ContinousLoaded::Loaded) {
                    let page_top =  document()
                        .get_element_by_id(format!("{}", page - 1).as_str())
                        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                        .map(|el| el.offset_top() as f64)
                        .unwrap_or_default();

                    info!("scroll to {}", page_top);
                    window().scroll_to_with_x_and_y(0.0_f64, page_top);
                    this.pages_loaded.set_neq(ContinousLoaded::Scrolled);
                }

                async {}
            })))
            .children(&mut [
                html!("button", {
                    .style("width", "100%")
                    .style("height", "5rem")
                    .style("border-width", "2px")
                    .style("border-style", "dashed")
                    .style("margin-top", "env(safe-area-inset-top)")
                    .attr_signal("disabled", this.prev_chapter.signal().map(|prev_chapter| if prev_chapter.is_some() { None } else { Some("true") }))
                    .text_signal(this.prev_chapter.signal().map(|prev_chapter| if prev_chapter.is_some() { "Prev Chapter" } else { "No Prev Chapter" }))
                    .event(clone!(this => move |_: events::Click| {
                        if let Some(prev_chapter) = this.prev_chapter.get() {
                            this.chapter_id.set(prev_chapter);
                        } else {
                            info!("no prev_page or prev_chapter");
                        }
                    }))
                })
            ])
            .children_signal_vec(this.pages_signal().map(clone!(this => move |(index, page, status)|
                if !matches!(status, PageStatus::Error) {
                    html!("img" => HtmlImageElement, {
                        .class_signal("continuous-image-loading", signal::always(status).map(|s| matches!(s, PageStatus::Initial)))
                        .style("margin-left", "auto")
                        .style("margin-right", "auto")
                        .style_signal("margin-top", this.reader_settings.padding.signal().map(|x| x.then(|| "0.25rem")))
                        .style_signal("margin-bottom", this.reader_settings.padding.signal().map(|x| x.then(|| "0.25rem")))
                        .attr("id", format!("{}", index).as_str())
                        .attr_signal("src", this.image_src_signal(index, 3, 4, page.clone(), status))
                        .style_signal("max-width", this.fit_signal().map(|(fit, zoom)| match fit {
                            crate::common::Fit::Height => "none".to_string(),
                            _ => format!("{}px", 768.0 * zoom),
                        }))
                        .style_signal("object-fit", this.reader_settings.fit.signal().map(|fit| match fit {
                            crate::common::Fit::All => "contain",
                            _ => "initial",
                        }))
                        .style_signal("width", this.fit_signal().map(|(fit, zoom)| match fit {
                            crate::common::Fit::Height =>"initial".to_string(),
                            _ => format!("{}vw", 100.0 * zoom)
                        }))
                        .style_signal("height", this.fit_signal().map(|(fit, zoom)| match fit {
                            crate::common::Fit::Width => "initial".to_string(),
                            _ => format!("{}vh", 100.0 * zoom)
                        }))
                        .event(clone!(this, page => move |_: events::Error| {
                            log::error!("error loading image");
                            let mut lock = this.pages.lock_mut();
                            lock.set_cloned(index, (page.clone(), PageStatus::Error));
                        }))
                        .event(clone!(this, page => move |_: events::Load| {
                            this.pages_loaded.set_if(ContinousLoaded::Loaded, |a, _| {
                                matches!(a, ContinousLoaded::Initial)
                            });
                            if !matches!(status, PageStatus::Loaded) {
                                let mut lock = this.pages.lock_mut();
                                lock.set_cloned(index, (page.clone(), PageStatus::Loaded));
                            }
                        }))
                        .event(clone!(this => move |_: events::Click| {
                            this.is_bar_visible.set_neq(!this.is_bar_visible.get());
                        }))
                    })
                } else {
                    html!("div", {
                        .attr("id", index.to_string().as_str())
                        .style("display", "flex")
                        .style("height", "calc(100vw * 1.59)")
                        .children(&mut [
                            html!("button", {
                                .style("margin", "auto")
                                .text("Retry")
                                .event(clone!(this, page => move |_: events::Click| {
                                    let mut lock = this.pages.lock_mut();
                                    lock.set_cloned(index, (page.clone(), PageStatus::Initial));
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
                    .attr_signal("disabled", this.next_chapter.signal().map(|next_chapter| if next_chapter.is_some() { None } else { Some("true") }))
                    .text_signal(this.next_chapter.signal().map(|next_chapter| if next_chapter.is_some() { "Next Chapter" } else { "No Next Chapter" }))
                    .event(clone!(this => move |_: events::Click| {
                        if let Some(next_chapter) = this.next_chapter.get() {
                            this.chapter_id.set(next_chapter);
                            window().scroll_to_with_x_and_y(0.0_f64, 0.0_f64);
                        } else {
                            info!("no next_page or next_chapter");
                        }
                    }))
                })
            ])
            .global_event_with_options(&EventOptions::preventable(), clone!(this => move |e: events::KeyDown| {
                if e.key() == " " {
                    e.prevent_default(); 
                    this.is_bar_visible.set_neq(!this.is_bar_visible.get());
                }
            }))
            .global_event(clone!(this => move |_: events::Scroll| {
                let mut page_no = 0;
                let window_height = body().offset_height();
                let client_height = document().document_element().unwrap_throw().client_height();
                let body_top = window().scroll_y().unwrap_throw().round() as i32;
                let pages_len = this.pages.lock_ref().len() ;
                for i in 0..pages_len {
                    let page_top = document()
                        .get_element_by_id(format!("{}", i).as_str())
                        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                        .map(|el| el.offset_top())
                        .unwrap_or_default();
                    if page_top > body_top {
                        page_no = i;
                        break;
                    }
                }
                if  body_top + client_height > window_height - 10 {
                    info!("window_height: {} body_top: {}", window_height, body_top + client_height);
                    page_no = pages_len - 1;
                }
                let is_last_page = pages_len == this.current_page.get() + 1;
                if !(is_last_page && page_no == 0) {
                    this.current_page.set_neq(page_no as usize);
                }
            }))
        })
    }

    fn render_single(this: Rc<Self>) -> Dom {
        html!("div", {
            .attr("id", "page-list")
            .style("display", "flex")
            .style("align-items", "center")
            .style("margin", "auto")
            .style_signal("width", this.zoom.signal().map(|zoom| format!("{}vw", 100.0 * zoom)))
            .style_signal("height", this.zoom.signal().map(|zoom| format!("{}vh", 100.0 * zoom)))
            .children_signal_vec(this.pages_signal().map(clone!(this => move |(index, page, status)|
                if !matches!(status, PageStatus::Error) {
                    html!("img", {
                        .style("margin-left", "auto")
                        .style("margin-right", "auto")
                        .style_signal("max-width", this.fit_signal().map(|(fit, zoom)| match fit {
                            crate::common::Fit::Height => "none".to_string(),
                            _ => format!("{}%", 100.0 * zoom),
                        }))
                        .style_signal("object-fit", this.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::All => "contain",
                            _ => "initial",
                        }))                        
                        .style_signal("width", this.fit_signal().map(|(fit, zoom)| match fit {
                            crate::common::Fit::Height => "initial".to_string(),
                            _ => format!("{}vw", 100.0 * zoom)
                        }))
                        .style_signal("height", this.fit_signal().map(|(fit, zoom)| match fit {
                            crate::common::Fit::Width => "initial".to_string(),
                            _ => format!("{}vh", 100.0 * zoom)
                        }))
                        .visible_signal(this.current_page.signal_cloned().map(clone!(this => move |x| {
                            this.prev_page.set_neq(x.checked_sub(1));
                            if x + 1 < this.pages.lock_ref().len() {
                                this.next_page.set_neq(Some(x + 1));
                            }

                            x == index
                        })))
                        .attr_signal("src", this.image_src_signal(index, 2, 3, page.clone(), status))
                        .event(clone!(this, page => move |_: events::Error| {
                            log::error!("error loading image");
                            let mut lock = this.pages.lock_mut();
                            lock.set_cloned(index, (page.clone(), PageStatus::Error));
                        }))
                        .event(clone!(this, page => move |_: events::Load| {
                            if !matches!(status, PageStatus::Loaded) {
                                let mut lock = this.pages.lock_mut();
                                lock.set_cloned(index, (page.clone(), PageStatus::Loaded));
                            }
                        }))
                    })
                } else {
                    html!("div", {
                        .attr("id", index.to_string().as_str())
                        .style("display", "flex")
                        .style("height", "100vh")
                        .style("width", "100vw")
                        .visible_signal(this.current_page.signal_cloned().map(clone!(this => move |x| {
                            this.prev_page.set_neq(x.checked_sub(1));
                            if x + 1 < this.pages.lock_ref().len() {
                                this.next_page.set_neq(Some(x + 1));
                            }

                            x == index
                        })))
                        .children(&mut [
                            html!("button", {
                                .style("margin", "auto")
                                .style("z-index", "20")
                                .text("Retry")
                                .event(clone!(this, page => move |_: events::Click| {
                                    let mut lock = this.pages.lock_mut();
                                    lock.set_cloned(index, (page.clone(), PageStatus::Initial));
                                }))
                            })
                        ])
                    })
                }
            )))
        })
    }

    fn render_double(this: Rc<Self>) -> Dom {
        html!("div", {
            .attr("id", "page-list")
            .style("display", "flex")
            .style("margin", "auto")
            .style_signal("width", this.zoom.signal().map(|zoom| format!("{}vw", 100.0 * zoom)))
            .style_signal("height", this.zoom.signal().map(|zoom| format!("{}vh", 100.0 * zoom)))
            .style("align-items", "center")
            .style_signal("flex-direction", this.reader_settings.direction.signal_cloned().map(|x| match x {
                Direction::LeftToRight => "row",
                Direction::RightToLeft => "row-reverse",
            }))
            .children_signal_vec(this.pages_signal().map(clone!(this => move |(index, page, status)|
                if !matches!(status, PageStatus::Error) {
                    html!("img" => HtmlImageElement, {
                        .style("margin-left", "auto")
                        .style("margin-right", "auto")
                        .attr("id", format!("{}", index).as_str())
                        .style_signal("max-width", this.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Height => "none",
                            _ => "100%",
                        }))
                        .style_signal("object-fit", this.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::All => "contain",
                            _ => "initial",
                        }))
                        .style_signal("height", this.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Width => "initial",
                            _ => "100%"
                        }))
                        .attr_signal("src", this.image_src_signal(index, 2, 4, page.clone(), status))
                        .event(clone!(this, page => move |_: events::Error| {
                            log::error!("error loading image");
                            let mut lock = this.pages.lock_mut();
                            lock.set_cloned(index, (page.clone(), PageStatus::Error));
                        }))
                        .event(clone!(this, page => move |_: events::Load| {
                            if !matches!(status, PageStatus::Loaded) {
                                let mut lock = this.pages.lock_mut();
                                lock.set_cloned(index, (page.clone(), PageStatus::Loaded));
                            }
                        }))
                        .with_node!(img => {
                            .style_signal("width", this.current_page.signal_cloned().map(clone!(this, index, img => move |current_page|
                                if (index == current_page && img.natural_width() > img.natural_height())
                                    || matches!(this.reader_settings.fit.get(), crate::common::Fit::Height) {
                                    "initial"
                                } else {
                                    "50%"
                                }
                            )))
                            .visible_signal(this.current_page.signal_cloned().map(clone!(this, index, img => move |current_page| {
                                let mut hidden = true;
                                if index == current_page {
                                    hidden = false;
                                    if current_page > 0 {
                                        let is_prev_img_landscape = if let Some(prev_img) = document().get_element_by_id(format!("{}", current_page - 1).as_str()).and_then(|el| el.dyn_into::<web_sys::HtmlImageElement>().ok()) {
                                            prev_img.natural_width() > prev_img.natural_height()
                                        } else {
                                            false
                                        };
                                        let sub = if is_prev_img_landscape || current_page == 1 {
                                            1
                                        } else {
                                            2
                                        };
                                        this.prev_page.set_neq(current_page.checked_sub(sub));
                                    }
                                } else if index == current_page + 1 {
                                    let is_prev_img_portrait = if let Some(prev_img) = document().get_element_by_id(format!("{}", current_page).as_str()).and_then(|el| el.dyn_into::<web_sys::HtmlImageElement>().ok()) {
                                        prev_img.natural_width() <= prev_img.natural_height()
                                    } else {
                                        true
                                    };

                                    let pages_len = this.pages.lock_ref().len();
                                    if img.natural_width() < img.natural_height() && is_prev_img_portrait {
                                        hidden = false;
                                        if current_page + 2 < pages_len {
                                            this.next_page.set_neq(Some(current_page + 2));
                                        } else {
                                            this.next_page.set_neq(None);
                                        }
                                    } else if current_page + 1 < pages_len {
                                        this.next_page.set_neq(Some(current_page + 1));
                                    } else {
                                        this.next_page.set_neq(None);
                                    }
                                }

                                !hidden
                            })))
                            .event(clone!(this, index => move |_: events::Load| {
                                let current_page = this.current_page.get();
                                if index == current_page || index == current_page + 1 {
                                    this.current_page.set_neq(current_page);
                                }
                            }))
                        })
                    })
                } else {
                    html!("div", {
                        .attr("id", format!("{}", index).as_str())
                        .style("display", "flex")
                        .style_signal("width", this.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Height => "none",
                            _ => "100%",
                        }))
                        .style_signal("height", this.reader_settings.fit.signal().map(|x| match x {
                            crate::common::Fit::Width => "initial",
                            _ => "100%"
                        }))
                        .visible_signal(this.current_page.signal_cloned().map(clone!(this, index => move |current_page| {
                            let mut hidden = true;
                            if index == current_page {
                                hidden = false;
                                if current_page > 0 {
                                    let sub = if current_page == 1 {
                                        1
                                    } else {
                                        2
                                    };
                                    this.prev_page.set_neq(current_page.checked_sub(sub));
                                }
                            } else if index == current_page + 1 {
                                hidden = false;
                                if current_page + 2 < this.pages.lock_ref().len() {
                                    this.next_page.set_neq(Some(current_page + 2));
                                } else {
                                    this.next_page.set_neq(None);
                                }
                            }

                            !hidden
                        })))
                        .children(&mut [
                            html!("button", {
                                .style("margin", "auto")
                                .style("z-index", "20")
                                .text("Retry")
                                .event(clone!(this, page => move |_: events::Click| {
                                    let mut lock = this.pages.lock_mut();
                                    lock.set_cloned(index, (page.clone(), PageStatus::Initial));
                                }))
                            })
                        ])
                    })
                }
            )))
        })
    }

    pub fn render(this: Rc<Self>) -> Dom {
        html!("div", {
            .attr("id", "this")
            .future(this.current_page.signal().for_each(clone!(this => move |page| {
                Self::update_page_read(this.clone(), page);

                this.is_bar_visible.set_neq(false);

                if page == 0 {
                    this.prev_page.set(None);
                } else if page + 1 == this.pages.lock_ref().len() {
                    this.next_page.set(None);
                }

                async {}
            })))
            .future(this.chapter_id.signal().for_each(clone!(this => move |chapter_id| {
                let nav = match chapter_id {
                    _ if Some(chapter_id) == this.prev_chapter.get() => Nav::Prev,
                    _ if Some(chapter_id) == this.next_chapter.get() => Nav::Next,
                    _ => Nav::None,
                };

                Self::fetch_detail(this.clone(), chapter_id, nav);

                async {}
            })))
            .future(this.reader_settings.background.signal_cloned().for_each(|x| {
                document().body().map(|body| body.style().set_property("background-color", match x {
                    Background::White => "white",
                    Background::Black => "black",
                }));

                async {}
            }))
            .global_event(clone!(this => move |_:events::Resize| this.reader_settings.display_mode.set(this.reader_settings.display_mode.get())))
            .children(&mut [
                Self::render_topbar(this.clone()),
            ])
            .child_signal(this.reader_settings.reader_mode.signal_cloned().map(clone!(this => move |x| match x {
                ReaderMode::Continous => Some(Self::render_vertical(this.clone())),
                ReaderMode::Paged => Some(html!("div", {
                    .children(&mut [
                        Self::render_navigation(this.clone())
                    ])
                    .child_signal(this.reader_settings.display_mode.signal_cloned().map(clone!(this => move |x| match x.get() {
                        DisplayMode::Single => Some(Self::render_single(this.clone())),
                        DisplayMode::Double => Some(Self::render_double(this.clone())),
                        DisplayMode::Auto => None // shouldn't return this
                    })))
                }))
            })))
            .children(&mut [
                Self::render_page_indicator(this.clone()),
                Self::render_bottombar(this.clone()),
                ReaderSettings::render(this.reader_settings.clone()),
                Spinner::render(this.spinner.clone())
            ])
        })
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        document().body().map(|body| body.style().set_property("background-color", "var(--background-color)"));
    }
}