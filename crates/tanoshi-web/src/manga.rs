use crate::{
    common::{
        ChapterSettings, ChapterSort, Filter,  Order, Route, Sort, Spinner, snackbar, SelectCategoryModal, SelectTrackMangaModal, TrackerStatus
    }, 
    query, 
    utils::{AsyncLoader, proxied_image_url, window}
};
use chrono::NaiveDateTime;
use dominator::{Dom, EventOptions, clone, events, html, routing, svg, with_node};
use futures_signals::{signal::{self, Mutable, SignalExt}, signal_vec::{MutableVec, SignalVecExt}};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

#[derive(Clone)]
struct ReadProgress {
    pub at: NaiveDateTime,
    pub last_page: i64,
    pub is_complete: bool,
}

#[derive(Clone)]
struct Chapter {
    pub id: i64,
    pub title: String,
    pub number: f64,
    pub scanlator: String,
    pub uploaded: NaiveDateTime,
    pub read_progress: Option<ReadProgress>,
    pub selected: Mutable<bool>,
    pub downloaded_path: Option<String>
}

impl Default for Chapter {
    fn default() -> Self {
        Self {
            id: Default::default(),
            title: Default::default(),
            number: Default::default(),
            scanlator: Default::default(),
            uploaded: NaiveDateTime::from_timestamp(0, 0),
            read_progress: Default::default(),
            selected: Default::default(),
            downloaded_path: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
enum SelectState {
    None,
    Category,
    Tracker
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
    link: Mutable<Option<String>>,
    status: Mutable<Option<String>>,
    is_favorite: Mutable<bool>,
    next_chapter: Mutable<Option<Chapter>>,
    chapters: MutableVec<Rc<Chapter>>,
    is_edit_chapter: Mutable<bool>,
    is_tracker_available: Mutable<bool>,
    num_tracked: Mutable<i64>,
    trackers: MutableVec<TrackerStatus>,
    chapter_settings: Rc<ChapterSettings>,
    select_state: Mutable<SelectState>,
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
            link: Mutable::new(None),
            status: Mutable::new(None),
            is_favorite: Mutable::new(false),
            next_chapter: Mutable::new(None),
            chapters: MutableVec::new(),
            is_edit_chapter: Mutable::new(false),
            is_tracker_available: Mutable::new(false),
            num_tracked: Mutable::new(0),
            trackers: MutableVec::new(),
            chapter_settings: ChapterSettings::new(false, true),
            select_state: Mutable::new(SelectState::None),
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
                    manga.link.set_neq(Some(result.link));
                    manga.status.set_neq(result.status);
                    manga.is_favorite.set_neq(result.is_favorite);
                    manga.next_chapter.set(result.next_chapter.map(|chapter| Chapter {
                        id: chapter.id,
                        read_progress: chapter.read_progress.as_ref().map(|progress| ReadProgress {
                            at: NaiveDateTime::parse_from_str(&progress.at, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse read at date"),
                            last_page: progress.last_page,
                            is_complete: progress.is_complete,
                        }),
                        ..Default::default()
                    }));
                    manga.is_tracker_available.set(!result.trackers.is_empty());
                    manga.num_tracked.set(result.trackers.iter().map(|tracker| if tracker.tracker_manga_id.is_some() { 1 } else { 0 }).sum());
                    manga.trackers.lock_mut().replace_cloned(result.trackers.iter().map(|t| TrackerStatus{
                        tracker: t.tracker.clone(), 
                        tracker_manga_id: Mutable::new(t.tracker_manga_id.clone()),
                        ..Default::default()
                    }).collect());
                    manga.chapters.lock_mut().replace_cloned(result.chapters.iter().map(|chapter| Rc::new(Chapter{
                        id: chapter.id,
                        title: chapter.title.clone(),
                        number: chapter.number,
                        scanlator: chapter.scanlator.clone(),
                        uploaded: NaiveDateTime::parse_from_str(&chapter.uploaded, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse uploaded date"),
                        read_progress: chapter.read_progress.as_ref().map(|progress| ReadProgress {
                            at: NaiveDateTime::parse_from_str(&progress.at, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse read at date"),
                            last_page: progress.last_page,
                            is_complete: progress.is_complete,
                        }),
                        selected: Mutable::new(false),
                        downloaded_path: chapter.downloaded_path.clone(),
                    })).collect());

                    manga.chapter_settings.load_by_manga_id(manga.id.get());                    
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
                    manga.link.set_neq(Some(result.link));
                    manga.status.set_neq(result.status);
                    manga.is_favorite.set_neq(result.is_favorite);
                    manga.next_chapter.set(result.next_chapter.map(|chapter| Chapter {
                        id: chapter.id,
                        read_progress: chapter.read_progress.as_ref().map(|progress| ReadProgress {
                            at: NaiveDateTime::parse_from_str(&progress.at, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse read at date"),
                            last_page: progress.last_page,
                            is_complete: progress.is_complete,
                        }),
                        ..Default::default()
                    }));
                    manga.is_tracker_available.set(!result.trackers.is_empty());
                    manga.num_tracked.set(result.trackers.iter().map(|tracker| if tracker.tracker_manga_id.is_some() { 1 } else { 0 }).sum());
                    manga.trackers.lock_mut().replace_cloned(result.trackers.iter().map(|t| TrackerStatus{
                        tracker: t.tracker.clone(), 
                        tracker_manga_id: Mutable::new(t.tracker_manga_id.clone()),
                        ..Default::default()
                    }).collect());
                    manga.chapters.lock_mut().replace_cloned(result.chapters.iter().map(|chapter| Rc::new(Chapter{
                        id: chapter.id,
                        title: chapter.title.clone(),
                        number: chapter.number,
                        scanlator: chapter.scanlator.clone(),
                        uploaded: NaiveDateTime::parse_from_str(&chapter.uploaded, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse uploaded date"),
                        read_progress: chapter.read_progress.as_ref().map(|progress| ReadProgress {
                            at: NaiveDateTime::parse_from_str(&progress.at, "%Y-%m-%dT%H:%M:%S%.f").expect_throw("failed to parse read at date"),
                            last_page: progress.last_page,
                            is_complete: progress.is_complete,
                        }),
                        selected: Mutable::new(false),
                        downloaded_path: chapter.downloaded_path.clone(),
                    })).collect());

                    manga.chapter_settings.load_by_manga_id(manga.id.get());
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

    fn download_chapters(manga: Rc<Self>) {
        let mut selected_chapter_id: Vec<i64> = vec![];
        for chapter in manga.chapters.lock_ref().to_vec() {
            if chapter.selected.get() {
                selected_chapter_id.push(chapter.id);
            }
        }

        manga.loader.load(clone!(manga => async move {
            match query::download_chapters(&selected_chapter_id).await {
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

    

    fn remove_download_chapters(manga: Rc<Self>) {
        let mut selected_chapter_id: Vec<i64> = vec![];
        for chapter in manga.chapters.lock_ref().to_vec() {
            if chapter.selected.get() {
                selected_chapter_id.push(chapter.id);
            }
        }

        manga.loader.load(clone!(manga => async move {
            match query::remove_downloaded_chapters(&selected_chapter_id).await {
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

    

    pub fn add_to_library(manga: Rc<Self>, category_ids: Vec<i64>)  {
        if manga.id.get() == 0 {
            return;
        }

        manga.loader.load(clone!(manga => async move {
            match query::add_to_library(manga.id.get(), category_ids).await {
                Ok(_) => {
                    manga.is_favorite.set_neq(true);
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }))
    }

    fn remove_from_library(manga: Rc<Self>) {
        if manga.id.get() == 0 {
            return;
        }
        manga.loader.load(clone!(manga => async move {
            match query::delete_from_library(manga.id.get()).await {
                Ok(_) => {
                    manga.is_favorite.set_neq(false);
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
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
                    .event(|_: events::Click| {
                        let history = window().history().unwrap();
                        if history.length().unwrap() > 1 {
                            let _ = history.back();
                        } else {
                            routing::go_to_url("/");
                        }
                    })
                }),
                html!("span", {
                    .style("overflow", "hidden")
                    .style("text-overflow", "ellipsis")
                    .style("white-space", "nowrap")
                    .style("margin-left", "0.5rem")
                    .style("margin-right", "0.5rem")
                    .text_signal(manga.title.signal_cloned().map(|x| x.unwrap_or_else(|| "".to_string())))
                }),
                html!("div", {
                    .style("min-width", "4.5rem")
                    .children(&mut [
                        html!("button", {
                            .style("padding", "0.25rem")
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
                                            .attribute("d", "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15")
                                        })
                                    ])
                                }),
                            ])
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
                        html!("a", {
                            .class("button")
                            .attribute_signal("href", manga.link.signal_cloned().map(|ext_link| ext_link.unwrap_or_else(|| "".to_string())))
                            .attribute_signal("disabled", manga.link.signal_cloned().map(|ext_link| ext_link.map(|_| "")))
                            .attribute("target", "_blank")
                            .style("padding", "0.5rem")
                            .children(&mut [
                                svg!("svg", {
                                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                                    .attribute("fill", "currentColor")
                                    .attribute("viewBox", "0 0 20 20")
                                    .class("icon")
                                    .children(&mut [
                                        svg!("path", {
                                            .attribute("d", "M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z")
                                        }),
                                        svg!("path", {
                                            .attribute("d", "M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z")
                                        })
                                    ])
                                })
                            ])
                        })
                    ])
                })
            ])
        })
    }

    pub fn render_header(manga: Rc<Self>) -> Dom {
        html!("div", {
            .class("manga-detail-header")
            .attribute("id", "detail")
            .style("margin-top", "0.5rem")
            .style("margin-left", "0.5rem")
            .children(&mut [
                html!("div", {
                    .style("display", "flex")
                    // .class_signal("animate-pulse", manga.loader.is_loading())
                    .children(&mut [
                        html!("div", {
                            .child_signal(manga.cover_url.signal_cloned().map(|x| {
                                if let Some(cover_url) = x {
                                    Some(html!("img", {
                                        .style("border-radius", "0.375rem")
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
                                    .style("font-size", "larger")
                                    .style("font-weight", "bold")
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
                    .class("action-button")
                    .style("display", "flex")
                    .style("padding", "0.5rem")
                    .style("margin-left", "0.5rem")
                    .style("margin-top", "0.5rem")
                    .style("margin-bottom", "0.5rem")
                    .style("align-items", "center")
                    .style_important_signal("background-color", manga.is_favorite.signal().map(|x| x.then(|| "var(--primary-color)")))
                    .style_important_signal("color", manga.is_favorite.signal().map(|x| x.then(|| "white")))
                    .children(&mut [
                        svg!("svg", {
                            .attribute("xmlns", "http://www.w3.org/2000/svg")
                            .attribute("fill", "currentColor")
                            .attribute("viewBox", "0 0 20 20")
                            .class("icon-sm")
                            .children(&mut [
                                svg!("path", {
                                    .attribute("fill-rule", "evenodd")
                                    .attribute("d", "M3 5a2 2 0 012-2h10a2 2 0 012 2v10a2 2 0 01-2 2H5a2 2 0 01-2-2V5zm11 1H6v8l4-2 4 2V6z")
                                    .attribute("clip-rule", "evenodd")
                                })
                            ])
                        }),
                        html!("span", {
                            .style("margin-left", "0.5rem")
                            .text_signal(manga.is_favorite.signal().map(|x| if x { "In Library" } else { "Add to Library" }))
                        })
                    ])
                    .event(clone!(manga => move |_: events::Click| {
                        if !manga.is_favorite.get() {
                            manga.select_state.set(SelectState::Category);
                        } else {
                            Self::remove_from_library(manga.clone());
                        }
                    }))
                }),
            ])
            .child_signal(manga.next_chapter.signal_cloned().map(|next_chapter| next_chapter.map(|chapter| html!("button", {
                .class("action-button")
                .style("display", "flex")
                .style("padding", "0.5rem")
                .style("margin-left", "0.5rem")
                .style("margin-top", "0.5rem")
                .style("margin-bottom", "0.5rem")
                .style("align-items", "center")
                .children(&mut [
                    svg!("svg", {
                        .attribute("xmlns", "http://www.w3.org/2000/svg")
                        .attribute("fill", "currentColor")
                        .attribute("viewBox", "0 0 20 20")
                        .class("icon-sm")
                        .children(&mut [
                            svg!("path", {
                                .attribute("d", "M9 4.804A7.968 7.968 0 005.5 4c-1.255 0-2.443.29-3.5.804v10A7.969 7.969 0 015.5 14c1.669 0 3.218.51 4.5 1.385A7.962 7.962 0 0114.5 14c1.255 0 2.443.29 3.5.804v-10A7.968 7.968 0 0014.5 4c-1.255 0-2.443.29-3.5.804V12a1 1 0 11-2 0V4.804z")
                            })
                        ])
                    }),
                    html!("span", {
                        .style("margin-left", "0.5rem")
                        .text("Read")
                    })
                ])
                .event(clone!(chapter => move |_: events::Click| {
                    routing::go_to_url(Route::Chapter(chapter.id, chapter.read_progress.as_ref().map(|progress| progress.last_page).unwrap_or(0)).url().as_str());
                }))
            }))))
            .child_signal(manga.is_tracker_available.signal_cloned().map(clone!(manga => move |is_tracker_available| is_tracker_available.then(|| 
                html!("button", {
                    .class("action-button")
                    .style("display", "flex")
                    .style("padding", "0.5rem")
                    .style("margin-left", "0.5rem")
                    .style("margin-top", "0.5rem")
                    .style("margin-bottom", "0.5rem")
                    .style("align-items", "center")
                    .children(&mut [
                        svg!("svg", {
                            .attribute("xmlns", "http://www.w3.org/2000/svg")
                            .attribute("fill", "currentColor")
                            .attribute("viewBox", "0 0 20 20")
                            .class("icon-sm")
                            .children(&mut [
                                svg!("path", {
                                    .attribute("d", "M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z")
                                })
                            ])
                        }),
                        html!("span", {
                            .style("margin-left", "0.5rem")
                            .text_signal(manga.num_tracked.signal_cloned().map(|num_tracked| if num_tracked > 0 {
                                format!("{num_tracked} Tracked")
                            } else {
                                "Track".to_string()
                            }))
                        })
                    ])
                    .event(clone!(manga => move |_: events::Click| {
                        manga.select_state.set(SelectState::Tracker);
                    }))
                })
            ))))
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
                        .style("white-space", "pre-wrap")
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
        let filter = manga.chapter_settings.filter.clone();
        html!("div", {
            .class("chapter-list")
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
                                        manga.chapter_settings.toggle_show()
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
                                                    .attribute("d", "M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z")
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
                            Filter::Read => chapter.read_progress.as_ref().map(|progress| progress.is_complete).unwrap_or(false),
                            Filter::Unread => chapter.read_progress.as_ref().map(|progress| !progress.is_complete).unwrap_or(true),
                        })))
                        .child_signal(is_edit_chapter.signal().map(clone!(chapter => move |is_edit_chapter| if is_edit_chapter {
                            Some(html!("input" => HtmlInputElement, {
                                .attribute("type", "checkbox")
                                .style("height", "0.75rem")
                                .style("width", "0.75rem")
                                .style("margin-left", "0.5rem")
                                .style("margin-right", "0.5rem")
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
                                .event_with_options(&EventOptions::preventable(), clone!(is_edit_chapter, chapter => move |e: events::Click| {
                                    e.prevent_default();
                                    if is_edit_chapter.get() {
                                        chapter.selected.set(!chapter.selected.get());
                                    }  else {
                                        routing::go_to_url(Route::Chapter(chapter.id, chapter.read_progress.as_ref().map(|progress| if progress.last_page < 0 { 0 } else { progress.last_page }).unwrap_or(0)).url().as_str());
                                    }
                                }))
                                .style("display", "inline-flex")
                                .style("padding", "0.5rem")
                                .style("width", "100%")
                                // .style_signal("opacity", chapter.read_progress.signal().map(|x| if x.is_some() {"0.5"} else {"1"}))
                                .style_signal("opacity", signal::always(chapter.read_progress.clone()).map(|x| x.map(|progress| progress.is_complete).map(|read| if read {"0.5"} else {"1"})))
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
                                                .child_signal(signal::always(chapter.read_progress.clone()).map(|x| x.map(|progress| html!("span", {
                                                    .style("font-size", "smaller")
                                                    .style("font-weight", "400")
                                                    .visible_signal(signal::always(progress.last_page).map(|page| page >= 0))
                                                    .text(format!("Page: {}", progress.last_page + 1).as_str())
                                                }))))
                                            }),
                                        ])
                                    })
                                ])
                            }),
                        ])
                        .child_signal(signal::always(chapter.downloaded_path.clone()).map(|downloaded_path| downloaded_path.is_some().then(|| html!("div", {
                            .style("align-self", "center")
                            .style("padding", "0.25rem")
                            .children(&mut [
                                svg!("svg", {
                                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                                    .attribute("viewBox", "0 0 20 20")
                                    .attribute("fill", "currentColor")
                                    .class("icon")
                                    .children(&mut [
                                        svg!("path", {
                                            .attribute("fill-rule", "evenodd")
                                            .attribute("clip-rule", "evenodd")
                                            .attribute("d", "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z")
                                        })
                                    ])
                                })
                            ])
                        }))))
                    }))))
                }),
                html!("div", {
                    .style("height", "env(safe-area-inset-bottom)")
                })
            ])
        })
    }

    pub fn render_main(manga_page: Rc<Self>) -> Dom {
        html!("div", {
            .class("content")
            .children(&mut [
                html!("div", {
                    .class("manga-detail")
                    .children(&mut [
                        Self::render_header(manga_page.clone()),
                        Self::render_action(manga_page.clone()),
                        Self::render_description(manga_page.clone()),
                        html!("div", {
                            .style("height", "2.5rem")
                        })
                    ])
                }),
                Self::render_chapters(manga_page.clone()),
                html!("div", {
                    .visible_signal(manga_page.is_edit_chapter.signal())
                    .class("edit-action-spacing")
                }),
                ChapterSettings::render(manga_page.chapter_settings.clone()),
            ])
            .child_signal(manga_page.select_state.signal_cloned().map(clone!(manga_page => move |select_state| {
                match select_state {
                    SelectState::Tracker => {
                        Some(SelectTrackMangaModal::new(manga_page.id.get(), manga_page.title.get_cloned().unwrap()).render(clone!(manga_page => move || {
                            Self::fetch_detail(manga_page.clone(), false);
                            manga_page.select_state.set(SelectState::None);                          
                        })))
                    }
                    SelectState::Category => {
                        Some(SelectCategoryModal::new().render(clone!(manga_page => move |category_ids: Vec<i64>| {
                            Self::add_to_library(manga_page.clone(), category_ids);
                            manga_page.select_state.set(SelectState::None);    
                        })))
                    }
                    _ => None
                }
            })))
            .child_signal(manga_page.is_edit_chapter.signal().map(clone!(manga_page => move |is_edit| if is_edit {
                Some(html!("div",{
                    .class("edit-action")
                    .children(&mut [
                        html!("button", {
                            .attribute("id", "select-all")
                            .style("margin", "auto")
                            .children(&mut [
                                svg!("svg", {
                                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                                    .attribute("fill", "currentColor")
                                    .attribute("viewBox", "0 0 20 20")
                                    .class("icon-sm")
                                    .children(&mut [
                                        svg!("path", {
                                            .attribute("fill-rule", "evenodd")
                                            .attribute("d", "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z")
                                            .attribute("clip-rule", "evenodd")
                                        })
                                    ])
                                }),
                            ])
                            .event(clone!(manga_page => move |_:events::Click| {
                                let chapters = manga_page.chapters.lock_ref();
                                for chapter in chapters.iter() {
                                    chapter.selected.set(true);
                                }
                            }))
                        }),
                        html!("button", {
                            .attribute("id", "deselect-all")
                            .style("margin", "auto")
                            .children(&mut [
                                svg!("svg", {
                                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                                    .attribute("fill", "currentColor")
                                    .attribute("viewBox", "0 0 20 20")
                                    .class("icon-sm")
                                    .children(&mut [
                                        svg!("path", {
                                            .attribute("fill-rule", "evenodd")
                                            .attribute("d", "M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z")
                                            .attribute("clip-rule", "evenodd")
                                        })
                                    ])
                                }),
                            ])
                            .event(clone!(manga_page => move |_:events::Click| {
                                let chapters = manga_page.chapters.lock_ref();
                                for chapter in chapters.iter() {
                                    chapter.selected.set(false);
                                }
                            }))
                        }),
                        html!("button", {
                            .attribute("id", "mark-as-read")
                            .style("margin", "auto")
                            .children(&mut [
                                svg!("svg", {
                                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                                    .attribute("fill", "currentColor")
                                    .attribute("viewBox", "0 0 20 20")
                                    .class("icon-sm")
                                    .children(&mut [
                                        svg!("path", {
                                            .attribute("d", "M10 12a2 2 0 100-4 2 2 0 000 4z")
                                        }),
                                        svg!("path", {
                                            .attribute("fill-rule", "evenodd")
                                            .attribute("d", "M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z")
                                            .attribute("clip-rule", "evenodd")
                                        })
                                    ])
                                }),
                            ])
                            .event(clone!(manga_page => move |_:events::Click| {
                                Self::mark_chapter_as_read(manga_page.clone());
                            }))
                        }),
                        html!("button", {
                            .attribute("id", "mark-as-unread")
                            .style("margin", "auto")
                            .children(&mut [
                                svg!("svg", {
                                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                                    .attribute("fill", "currentColor")
                                    .attribute("viewBox", "0 0 20 20")
                                    .class("icon-sm")
                                    .children(&mut [
                                        svg!("path", {
                                            .attribute("d", "M12.454 16.697L9.75 13.992a4 4 0 01-3.742-3.741L2.335 6.578A9.98 9.98 0 00.458 10c1.274 4.057 5.065 7 9.542 7 .847 0 1.669-.105 2.454-.303z")
                                        }),
                                        svg!("path", {
                                            .attribute("fill-rule", "evenodd")
                                            .attribute("d", "M3.707 2.293a1 1 0 00-1.414 1.414l14 14a1 1 0 001.414-1.414l-1.473-1.473A10.014 10.014 0 0019.542 10C18.268 5.943 14.478 3 10 3a9.958 9.958 0 00-4.512 1.074l-1.78-1.781zm4.261 4.26l1.514 1.515a2.003 2.003 0 012.45 2.45l1.514 1.514a4 4 0 00-5.478-5.478z")
                                            .attribute("clip-rule", "evenodd")
                                        })
                                    ])
                                }),
                            ])
                            .event(clone!(manga_page => move |_:events::Click| {
                                Self::mark_chapter_as_unread(manga_page.clone());
                            }))
                        }),
                        html!("button", {
                            .attribute("id", "download")
                            .style("margin", "auto")
                            .children(&mut [
                                svg!("svg", {
                                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                                    .attribute("fill", "currentColor")
                                    .attribute("viewBox", "0 0 20 20")
                                    .class("icon-sm")
                                    .children(&mut [
                                        svg!("path", {
                                            .attribute("fill-rule", "evenodd")
                                            .attribute("d", "M2 9.5A3.5 3.5 0 005.5 13H9v2.586l-1.293-1.293a1 1 0 00-1.414 1.414l3 3a1 1 0 001.414 0l3-3a1 1 0 00-1.414-1.414L11 15.586V13h2.5a4.5 4.5 0 10-.616-8.958 4.002 4.002 0 10-7.753 1.977A3.5 3.5 0 002 9.5zm9 3.5H9V8a1 1 0 012 0v5z")
                                            .attribute("clip-rule", "evenodd")
                                        })
                                    ])
                                }),
                            ])
                            .event(clone!(manga_page => move |_:events::Click| {
                                Self::download_chapters(manga_page.clone());
                            }))
                        }),
                        html!("button", {
                            .attribute("id", "remove-download")
                            .style("margin", "auto")
                            .children(&mut [
                                svg!("svg", {
                                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                                    .attribute("fill", "currentColor")
                                    .attribute("viewBox", "0 0 20 20")
                                    .class("icon-sm")
                                    .children(&mut [
                                        svg!("path", {
                                            .attribute("fill-rule", "evenodd")
                                            .attribute("d", "M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z")
                                            .attribute("clip-rule", "evenodd")
                                        })
                                    ])
                                }),
                            ])
                            .event(clone!(manga_page => move |_:events::Click| {
                                Self::remove_download_chapters(manga_page.clone());
                            }))
                        }),
                    ])
                }))
            } else {
                None
            })))
        })
    }

    pub fn render(manga_page: Rc<Self>) -> Dom {
        if manga_page.id.get() != 0 {
            Self::fetch_detail(manga_page.clone(), false);
        } else if manga_page.source_id.get() != 0 && manga_page.path.get_cloned() != "" {
            Self::fetch_detail_by_source_path(manga_page.clone());
        }

        html!("div", {
            .future(manga_page.chapter_settings.sort.signal_cloned().for_each(clone!(manga_page => move |sort| {
                let mut chapters = manga_page.chapters.lock_ref().to_vec();
                chapters.sort_by(|a, b| match sort {
                    ChapterSort { by: Sort::Number, order: Order::Asc} => a.number.partial_cmp(&b.number).unwrap_or(std::cmp::Ordering::Equal),
                    ChapterSort { by: Sort::Number, order: Order::Desc} => b.number.partial_cmp(&a.number).unwrap_or(std::cmp::Ordering::Equal),
                    ChapterSort { by: Sort::ReadAt, order: Order::Asc} => {
                        let a = a.read_progress.as_ref().map(|progress| progress.at).unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));
                        let b = b.read_progress.as_ref().map(|progress| progress.at).unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));
                        a.cmp(&b)
                    },
                    ChapterSort { by: Sort::ReadAt, order: Order::Desc} => {
                        let a = a.read_progress.as_ref().map(|progress| progress.at).unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));
                        let b = b.read_progress.as_ref().map(|progress| progress.at).unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));
                        b.cmp(&a)
                    }
                });
                manga_page.chapters.lock_mut().replace_cloned(chapters);

                async {}
            })))
            .style("display", "flex")
            .style("flex-direction", "column")
            .child_signal(manga_page.loader.is_loading().map(|is_loading| is_loading.then(|| Spinner::render_spinner(true))))
            .children(&mut [
                Self::render_topbar(manga_page.clone()),
                html!("div", {
                   .class("topbar-spacing")
                }),
                Self::render_main(manga_page),
            ])
        })
    }
}
