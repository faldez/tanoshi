use js_sys;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlElement};
use yew::prelude::*;
use yew::services::fetch::FetchTask;

use yew::{html, Component, ComponentLink, Html, InputData, Properties, ShouldRender};
use yew_router::{agent::RouteRequest, prelude::*};

use super::component::model::{BackgroundColor, PageRendering, SettingParams};
use super::component::{Pager, ReaderSeekbar, ReaderToolbar, Spinner, WeakComponentLink, Webtoon};

use std::collections::HashMap;
use tanoshi_lib::manga::{Chapter as ChapterModel, Manga as MangaModel};
use tanoshi_lib::rest::{HistoryRequest, ReadResponse};
use yew::format::Json;

#[derive(Clone, Properties)]
pub struct Props {
    pub chapter_id: i32,
    pub page: usize,
}

pub struct Reader {
    link: ComponentLink<Self>,
    router: Box<dyn Bridge<RouteAgent>>,
    fetch_task_map: HashMap<&'static str, FetchTask>,
    manga: MangaModel,
    chapter: ChapterModel,
    current_chapter_id: i32,
    current_page: usize,
    chapters: Vec<ChapterModel>,
    pages: Vec<String>,
    is_fetching: bool,
    is_bar_visible: bool,
    settings: SettingParams,
    should_fetch: bool,
}

pub enum Msg {
    ReadReady(ReadResponse),
    PageChange(usize),
    NextChapter,
    PrevChapter,
    ToggleBar,
    PageSliderChange(usize),
    RouterCallback,
    Refresh,
    Noop,
}

impl Component for Reader {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|_| Msg::RouterCallback);
        let router = RouteAgent::bridge(callback);
        let settings = SettingParams::parse_from_local_storage();

        let _ = window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .class_list()
            .remove_2("bg-white", "dark:bg-black");
        match settings.background_color.clone() {
            BackgroundColor::Black => {
                let _ = window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .class_list()
                    .add_1("bg-black");
            }
            BackgroundColor::White => {
                let _ = window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .class_list()
                    .add_1("bg-white");
            }
        };

        Reader {
            link,
            router,
            fetch_task_map: HashMap::new(),
            manga: MangaModel::default(),
            current_chapter_id: props.chapter_id,
            chapter: ChapterModel::default(),
            current_page: props.page.checked_sub(1).unwrap_or(0),
            chapters: vec![],
            pages: vec![],
            is_fetching: false,
            is_bar_visible: true,
            settings,
            should_fetch: true,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ReadReady(data) => {
                self.manga = data.manga;
                self.chapters = data.chapters;
                self.chapter = data.chapter;
                self.pages = data.pages;
                self.is_fetching = false;
            }
            Msg::PageChange(page) => {
                self.current_page = page;
                self.set_history();
                let route_string = format!(
                    "/chapter/{}/page/{}",
                    self.current_chapter_id,
                    self.current_page + 1,
                );
                let route = Route::from(route_string);
                self.router
                    .send(RouteRequest::ReplaceRouteNoBroadcast(route));
            }
            Msg::NextChapter => {
                self.next_chapter();
            }
            Msg::PrevChapter => {
                self.prev_chapter();
            }
            Msg::PageSliderChange(page) => {
                let page =
                    if self.settings.page_rendering == PageRendering::DoublePage && page % 2 != 0 {
                        page.checked_sub(1).unwrap_or(0)
                    } else {
                        page
                    };
                self.current_page = page;
                self.set_history();
            }
            Msg::ToggleBar => {
                self.is_bar_visible = !self.is_bar_visible;
            }
            Msg::RouterCallback => {
                self.read(false);
            }
            Msg::Refresh => {
                self.read(true);
            }
            Msg::Noop => {
                return false;
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.current_chapter_id != props.chapter_id
            || self.current_page != props.page.checked_sub(1).unwrap_or(0)
        {
            self.current_chapter_id = props.chapter_id;
            self.current_page = props.page.checked_sub(1).unwrap_or(0);
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let reader_link = &WeakComponentLink::<Reader>::default();
        *reader_link.borrow_mut() = Some(self.link.clone());
        return html! {
            <>
                <ReaderToolbar
                    visible={self.is_bar_visible}
                    manga_id={self.manga.id}
                    title={self.manga.title.clone()}
                    chapter={self.get_current_volume_and_chapter()}
                    on_refresh={self.link.callback(|_| Msg::Refresh)} />
                {
                    match self.settings.page_rendering {
                        PageRendering::LongStrip => html!{
                            <Webtoon
                                current_page=self.current_page
                                pages=&self.pages
                                reader_link=reader_link
                                on_page_change=self.link.callback(|page| Msg::PageChange(page))/>
                        },
                        _ => html!{
                            <Pager
                                current_page=self.current_page
                                pages=&self.pages
                                settings=self.settings.clone()
                                reader_link=reader_link
                                on_page_change=self.link.callback(|page| Msg::PageChange(page))
                                on_next_chapter=self.link.callback(|page| Msg::NextChapter)
                                on_prev_chapter=self.link.callback(|page| Msg::PrevChapter)/>
                        }
                    }
                }

                <Spinner is_active=self.is_fetching is_fullscreen=true />
                <ReaderSeekbar
                    visible={self.is_bar_visible}
                    disabled={self.settings.page_rendering == PageRendering::LongStrip}
                    current_page={self.current_page}
                    length={self.pages.len()}
                    on_next_chapter=self.link.callback(|page| Msg::NextChapter)
                    on_prev_chapter=self.link.callback(|page| Msg::PrevChapter)
                    on_input={self.link.callback(|e: InputData| Msg::PageSliderChange(e.value.parse::<usize>().unwrap()))} />
            </>
        };
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            if self.settings.page_rendering == PageRendering::DoublePage
                && self.current_page % 2 != 0
            {
                let route_string = format!(
                    "/chapter/{}/page/{}",
                    self.current_chapter_id, self.current_page
                );

                let route = Route::from(route_string);
                self.router.send(RouteRequest::ChangeRoute(route));
            }
        }
        if self.should_fetch {
            self.should_fetch = false;
            self.read(false);
        }

        window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("manga-reader")
            .expect("should have manga reader")
            .dyn_ref::<HtmlElement>()
            .expect("should load HtmlElement")
            .focus()
            .unwrap();
    }

    fn destroy(&mut self) {
        let _ = window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .class_list()
            .add_2("bg-white", "dark:bg-black");
        match self.settings.background_color.clone() {
            BackgroundColor::Black => {
                let _ = window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .class_list()
                    .remove_1("bg-black");
            }
            BackgroundColor::White => {
                let _ = window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .class_list()
                    .remove_1("bg-white");
            }
        };
    }
}

impl Reader {
    fn read(&mut self, refresh: bool) {
        if let Ok(task) = super::api::fetch_read(
            self.current_chapter_id,
            refresh,
            self.link.callback(
                move |response: super::api::FetchJsonResponse<ReadResponse>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::ReadReady(data);
                        }
                    }
                    Msg::Noop
                },
            ),
        ) {
            self.fetch_task_map.insert("read", task);
            self.is_fetching = true;
        }
    }

    fn next_chapter(&mut self) {
        let current_chapter_idx = match self
            .chapters
            .iter()
            .position(|chapter| chapter.id == self.current_chapter_id)
        {
            Some(index) => index,
            None => 0,
        };

        let is_next = match current_chapter_idx.checked_sub(1) {
            Some(index) => {
                self.current_chapter_id = self.chapters[index].id;
                true
            }
            None => false,
        };

        if is_next {
            self.pages.clear();
            let route_string = format!("/chapter/{}/page/1", self.current_chapter_id);
            self.current_page = 0;

            let route = Route::from(route_string);
            self.router.send(RouteRequest::ReplaceRoute(route));
        }
    }

    fn prev_chapter(&mut self) {
        let current_chapter_idx = match self
            .chapters
            .iter()
            .position(|chapter| chapter.id == self.current_chapter_id)
        {
            Some(index) => index + 1,
            None => 0,
        };

        self.current_chapter_id = match self.chapters.get(current_chapter_idx) {
            Some(chapter) => chapter.id,
            None => self.current_chapter_id,
        };
        if current_chapter_idx < self.chapters.len() {
            self.pages.clear();
            let route_string = format!("/chapter/{}/page/1", self.current_chapter_id,);
            let route = Route::from(route_string);
            self.router.send(RouteRequest::ReplaceRoute(route));
        }
    }

    fn get_date(&self) -> chrono::NaiveDateTime {
        let timestamp = js_sys::Date::now();
        let secs: i64 = (timestamp / 1000.0).floor() as i64;
        let nanoes: u32 = (timestamp as u32 % 1000) * 1_000_000;
        chrono::NaiveDateTime::from_timestamp(secs, nanoes)
    }

    fn set_history(&mut self) {
        let h = HistoryRequest {
            chapter_id: self.current_chapter_id,
            read: self.current_page as i32,
            at: self.get_date(),
        };
        if let Ok(task) = super::api::post_history(h, self.link.callback(|_| Msg::Noop)) {
            self.fetch_task_map.insert("history", task);
        }
    }

    fn get_current_volume_and_chapter(&self) -> String {
        match (self.chapter.vol.as_ref(), self.chapter.no.as_ref()) {
            (Some(vol), Some(ch)) => format!("Volume {} Chapter {}", vol, ch),
            (Some(vol), None) => format!("Volume {}", vol),
            (None, Some(ch)) => format!("Chapter {}", ch),
            (None, None) => "".to_string(),
        }
    }
}
