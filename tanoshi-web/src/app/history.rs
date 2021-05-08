use crate::app::AppRoute;
use std::collections::BTreeMap;
use web_sys::HtmlElement;
use yew::format::{Json, Nothing};
use yew::services::fetch::{FetchTask, Request, Response};
use yew::services::storage::Area;
use yew::services::{FetchService, StorageService};
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use yew_router::components::RouterAnchor;
use yew_router::service::RouteService;

use super::component::{Spinner, TopBar};
use tanoshi_lib::manga::{History as HistoryModel, Update as UpdateModel};
use tanoshi_lib::rest::{HistoryResponse, UpdatesResponse};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Node;
use yew::utils::{document, window};
use yew::virtual_dom::VNode;

#[derive(Debug, Eq, PartialEq)]
pub enum PageType {
    History,
    Updates,
}

impl Into<PageType> for String {
    fn into(self) -> PageType {
        match self.as_str() {
            "/updates" => PageType::Updates,
            "/history" => PageType::History,
            _ => PageType::Updates,
        }
    }
}

#[derive(Clone, Properties)]
pub struct Props {}

pub struct History {
    fetch_task: Option<FetchTask>,
    link: ComponentLink<Self>,
    history: BTreeMap<i64, Vec<Row>>,
    updates: BTreeMap<i64, Vec<Row>>,
    token: String,
    is_fetching: bool,
    closure: Closure<dyn Fn()>,
    page: i32,
    should_fetch: bool,
    page_type: PageType,
    route_service: RouteService<()>,
}

pub enum Msg {
    HistoryReady(HistoryResponse),
    UpdatesReady(UpdatesResponse),
    ScrolledDown,
    Noop,
}

struct Row {
    pub title: String,
    pub thumbnail_url: String,
    pub chapter: String,
    pub route: AppRoute,
}

impl From<HistoryModel> for Row {
    fn from(h: HistoryModel) -> Self {
        Row {
            title: h.title,
            thumbnail_url: h.thumbnail_url.unwrap_or("".to_string()),
            chapter: h.chapter,
            route: AppRoute::Reader(h.chapter_id, (h.read + 1) as usize),

        }
    }
}

impl From<UpdateModel> for Row {
    fn from(u: UpdateModel) -> Self {
        Row {
            title: u.title,
            thumbnail_url: u.thumbnail_url,
            chapter: u.number,
            route: AppRoute::Reader(u.chapter_id, 1),

        }
    }
}

impl Component for History {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let token = {
            if let Ok(token) = storage.restore("token") {
                token
            } else {
                "".to_string()
            }
        };
        let tmp_link = link.clone();
        let closure = Closure::wrap(Box::new(move || {
            let current_scroll = window().scroll_y().expect("error get scroll y")
                + window().inner_height().unwrap().as_f64().unwrap();
            let height = document()
                .get_element_by_id("updates")
                .expect("should have updates")
                .dyn_ref::<HtmlElement>()
                .unwrap()
                .offset_height() as f64;

            if current_scroll >= height {
                tmp_link.send_message(Msg::ScrolledDown);
            }
        }) as Box<dyn Fn()>);

        let route_service: RouteService<()> = RouteService::new();
        let page_type: PageType = route_service.get_path().into();

        History {
            fetch_task: None,
            link,
            history: BTreeMap::new(),
            updates: BTreeMap::new(),
            token,
            is_fetching: false,
            closure,
            page: 1,
            should_fetch: true,
            page_type,
            route_service,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        let page_type: PageType = self.route_service.get_path().into();
        if self.page_type != page_type {
            self.page_type = page_type;
            self.history.clear();
            self.updates.clear();
            self.page = 1;
            self.should_fetch = true;
            true
        } else {
            false
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        if self.should_fetch {
            window().set_onscroll(Some(self.closure.as_ref().unchecked_ref()));
            match self.page_type {
                PageType::History => self.fetch_history(),
                PageType::Updates => self.fetch_updates(),
            }
            self.should_fetch = false;
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::HistoryReady(data) => {
                let mut history = data.history;
                if history.is_empty() {
                    window().set_onscroll(None);
                } else {
                    let mut prev_days: i64 = -1;
                    for his in history.iter_mut() {
                        let days = self.calculate_days(his.at);
                        if prev_days != days {
                            prev_days = days;
                            his.days = Some(days);
                            his.show_sep = Some(true);
                        }
                        self.history.entry(days).and_modify(|rows| rows.push(his.clone().into())).or_insert(vec![his.clone().into()]);
                    }
                }
                self.is_fetching = false;
            }
            Msg::ScrolledDown => {
                if !self.is_fetching {
                    self.page += 1;
                    match self.page_type {
                        PageType::History => self.fetch_history(),
                        PageType::Updates => self.fetch_updates(),
                    }
                }
            }
            Msg::UpdatesReady(data) => {
                let mut updates = data.updates;
                let mut prev_days: i64 = -1;
                for update in updates.iter_mut() {
                    let days = self.calculate_days(update.uploaded);
                    if prev_days != days {
                        prev_days = days;
                        update.days = Some(days);
                        update.show_sep = Some(true);
                    }
                    self.updates.entry(days).and_modify(|rows| rows.push(update.clone().into())).or_insert(vec![update.clone().into()]);
                }
                self.is_fetching = false;
            }
            Msg::Noop => {
                return false;
            }
        };
        true
    }

    fn view(&self) -> Html {
        html! {
           <div class="mx-auto pb-20 max-h-screen overflow-scroll pt-12" style="margin-top:env(safe-area-inset-top)">
                <TopBar>
                    <span class="w-full text-center">{
                        match self.page_type {
                            PageType::History => "History",
                            PageType::Updates => "Updates",
                        }
                    }</span>
                </TopBar>
                <div class="flex flex-col" id="updates">
                {self.updates_or_history_cards()}
                </div>
                {
                    match self.is_fetching {
                        false => html!{
                            <div class="flex justify-center">
                                <button class="w-full block text-gray-700 dark:text-gray-300 hover:text-accent my-2 focus:outline-none" onclick=self.link.callback(|_| Msg::ScrolledDown)>{"Load More"}</button>
                            </div>
                        },
                        true => html!{<Spinner is_active=self.is_fetching is_fullscreen=false />}
                    }
                }
            </div>
        }
    }
    fn destroy(&mut self) {
        window().set_onscroll(None);
    }
}

impl History {
    fn calculate_days(&self, at: chrono::NaiveDateTime) -> i64 {
        let timestamp = js_sys::Date::now();
        let secs: i64 = (timestamp / 1000.0).floor() as i64;
        let nanoes: u32 = (timestamp as u32 % 1000) * 1_000_000;
        let today = chrono::NaiveDateTime::from_timestamp(secs, nanoes);
        today.date().signed_duration_since(at.date()).num_days()
    }

    fn subheader_days(&self, days: &i64) -> Html {
        html!{
            <div class="mx-auto w-full xl:w-1/2">
                <span class="w-full font-bold text-gray-900 dark:text-gray-100 text-xl">{
                    match days {
                        0 => "Today".to_string(),
                        1 => "Yesterday".to_string(),
                        _ => format!("{} Days Ago", days)
                    }
                }
                </span>
            </div>
        }
    }

    fn chapter_row(&self, route: &AppRoute, thumbnail_url: &String, title: &String, chapter: &String) -> Html {
        html!{
            <RouterAnchor<AppRoute>
                classes="w-full flex inline-flex content-center hover:bg-gray-200 dark:hover:bg-gray-700 p-2"
                route={route}>
                <div class="mr-4 my-2 h-16 w-16 flex-none object-fit object-center bg-center bg-cover rounded-lg" style={format!("background-image: url({})", thumbnail_url.clone())}/>
                <div class="flex flex-col my-auto text-gray-700 dark:text-gray-300">
                    {self.title(title.clone())}
                    <span class="text-md text-gray-700 dark:text-gray-300">{format!("Chapter {}", chapter.clone())}</span>
                </div>
            </RouterAnchor<AppRoute>>
        }
    }

    fn updates_or_history_cards(&self) -> Html {
        let rows = match self.page_type {
            PageType::History => {
                &self.history
            },
            PageType::Updates => {
                &self.updates
            }
        };

        rows.iter().map(|(days, rows)| {
            html!{
                <div class="flex flex-col w-full justify-center p-2">
                    {self.subheader_days(days)}
                    <div class="rounded-lg bg-white dark:bg-gray-900 mb-2">
                        <div class="w-full xl:w-1/2 divide-y divide-gray-300 dark:divide-gray-800 mx-auto">
                        {
                            for rows.iter().map(|h| {
                                self.chapter_row(&h.route, &h.thumbnail_url, &h.title, &h.chapter)
                            })
                        }
                        </div>
                    </div>
                </div>
            }
        }).collect()
    }

    fn title(&self, title: String) -> Html {
        let div = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("span")
            .unwrap();
        let _ = div.class_list().add_2("text-lg", "font-semibold");
        // TODO: proper sanitazion
        let _ = div.set_inner_html(&title);

        let node = Node::from(div);
        let vnode = VNode::VRef(node);
        vnode
    }

    fn fetch_history(&mut self) {
        let req = Request::get(format!("/api/history?page={}", self.page))
            .header("Authorization", self.token.to_string())
            .body(Nothing)
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(
                |response: Response<Json<Result<HistoryResponse, anyhow::Error>>>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::HistoryReady(data);
                        }
                    }
                    Msg::Noop
                },
            ),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
            self.is_fetching = true;
        }
    }
    
    fn fetch_updates(&mut self) {
        let req = Request::get(format!("/api/updates?page={}", self.page))
            .header("Authorization", self.token.to_string())
            .body(Nothing)
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(
                |response: Response<Json<Result<UpdatesResponse, anyhow::Error>>>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::UpdatesReady(data);
                        }
                    }
                    Msg::Noop
                },
            ),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
            self.is_fetching = true;
        }
    }
}
