use super::component::{Filter, Manga, MangaList, Spinner, TopBar, WeakComponentLink};
use web_sys::HtmlElement;
use yew::prelude::*;
use yew::services::fetch::FetchTask;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew::worker::{Bridge, Bridged};
use tanoshi_lib::manga::{Manga as MangaModel, Params, SortByParam, SortOrderParam, SourceLogin, SourceLoginResult};
use tanoshi_lib::rest::GetMangasResponse;
use yew_router::{agent::RouteRequest, prelude::Route, prelude::RouteAgent};

use crate::app::api;
use crate::app::worker::{Worker, Request as WorkerRequest, Response as WorkerResponse};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::format::Json;

#[derive(Clone, Properties)]
pub struct Props {
    pub source_name: String,
}

pub struct Source {
    link: ComponentLink<Self>,
    source_name: String,
    page: i32,
    mangas: Vec<MangaModel>,
    fetch_task: Option<FetchTask>,
    is_fetching: bool,
    closure: Option<Closure<dyn Fn()>>,
    keyword: String,
    is_login_page: bool,
    login: SourceLogin,
    show_filter: bool,
    sort_by: SortByParam,
    sort_order: SortOrderParam,
    catalogue_ref: NodeRef,
    scroll_position: i32,
    router: Box<dyn Bridge<RouteAgent>>,
    worker: Box<dyn Bridge<Worker>>,
}

pub enum Msg {
    MangasReady(GetMangasResponse),
    ScrolledDown,
    KeywordChanged(InputData),
    Search(FocusEvent),
    SourceLogin,
    LoginReady(SourceLoginResult),
    Submit(FocusEvent),
    UsernameChange(InputData),
    PasswordChange(InputData),
    RememberMeChange(InputData),
    TwoFactorChange(InputData),
    Filter,
    FilterClosed,
    FilterCancel,
    SortByChange(SortByParam),
    SortOrderChange(SortOrderParam),
    OnLongTap(usize),
    Favorited(usize),
    Unfavorited(usize),
    GoToDetail(i32),
    Noop,
}

impl Component for Source {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut storage = super::api::get_local_storage().unwrap();
        let mangas = if let Ok(data) =
            storage.restore(format!("source-{}-mangas", props.source_name.clone()).as_str())
        {
            let val = serde_json::from_str(&data).unwrap();
            storage.remove(format!("source-{}-mangas", props.source_name.clone()).as_str());
            val
        } else {
            vec![]
        };
        let page = if let Ok(data) =
            storage.restore(format!("source-{}-page", props.source_name.clone()).as_str())
        {
            let val = data.parse().unwrap();
            storage.remove(format!("source-{}-page", props.source_name.clone()).as_str());
            val
        } else {
            0
        };

        let scroll_position = if let Ok(data) =
            storage.restore(format!("source-{}-pos", props.source_name.clone()).as_str())
        {
            let val = data.parse().unwrap();
            storage.remove(format!("source-{}-pos", props.source_name.clone()).as_str());
            val
        } else {
            0
        };

        let callback = link.callback(|_| Msg::Noop);
        let router = RouteAgent::bridge(callback);

        let worker_callback = link.callback(|res| match res {
            WorkerResponse::Favorited(index, favorite) => {
                if favorite { 
                    Msg::Favorited(index) 
                } else { 
                    Msg::Unfavorited(index)
                }
            }
            _ => Msg::Noop,
        });
        let worker = Worker::bridge(worker_callback);

        Source {
            link,
            source_name: props.source_name,
            page,
            mangas,
            fetch_task: None,
            is_fetching: false,
            closure: None,
            keyword: "".to_string(),
            is_login_page: false,
            login: SourceLogin::default(),
            show_filter: false,
            sort_by: SortByParam::Views,
            sort_order: SortOrderParam::Desc,
            catalogue_ref: NodeRef::default(),
            scroll_position,
            router,
            worker,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::MangasReady(data) => {
                self.is_fetching = false;
                self.page += 1;
                let mut mangas = data.mangas;
                if self.page == 1 {
                    self.mangas = mangas;
                } else {
                    self.mangas.append(&mut mangas);
                }
            }
            Msg::ScrolledDown => {
                if !self.is_fetching {
                    self.fetch_mangas();
                }
            }
            Msg::KeywordChanged(e) => {
                self.keyword = e.value;
            }
            Msg::Search(e) => {
                e.prevent_default();
                self.mangas.clear();
                self.page = 1;
                self.fetch_mangas();
            }
            Msg::SourceLogin => {
                self.is_login_page = !self.is_login_page;
                self.is_fetching = false;
            }
            Msg::LoginReady(data) => {
                if let Ok(mut storage) = api::get_local_storage() {
                    storage.store(
                        format!("source-token-{}", &data.clone().source_name).as_str(),
                        Ok(data.clone().value),
                    );
                }

                self.is_login_page = false;
                self.fetch_mangas();
            }
            Msg::Submit(e) => {
                e.prevent_default();
                self.login();
            }
            Msg::UsernameChange(e) => {
                self.login.username = e.value;
            }
            Msg::PasswordChange(e) => {
                self.login.password = e.value;
            }
            Msg::RememberMeChange(_e) => {
                self.login.remember_me = Some(!self.login.remember_me.unwrap_or(false));
            }
            Msg::TwoFactorChange(e) => {
                self.login.two_factor = Some(e.value);
            }
            Msg::Filter => {
                if !self.show_filter {
                    self.show_filter = true;
                } else {
                    return false;
                }
            }
            Msg::FilterClosed => {
                if self.show_filter {
                    self.page = 1;
                    self.mangas.clear();
                    self.fetch_mangas();
                    self.show_filter = false;
                } else {
                    return false;
                }
            }
            Msg::FilterCancel => {
                self.show_filter = false;
            }
            Msg::SortByChange(sort_by) => {
                self.sort_by = sort_by;
            }
            Msg::SortOrderChange(sort_order) => {
                self.sort_order = sort_order;
            }
            Msg::OnLongTap(index) => {
                if self.mangas[index].is_favorite {
                    self.unfavorite(index);
                } else {
                    self.favorite(index);
                }
            }
            Msg::Favorited(index) => {
                self.mangas[index].is_favorite = true;
                self.get_manga_info(self.mangas[index].id);
                self.get_chapters(self.mangas[index].id, true);
            }
            Msg::Unfavorited(index) => {
                self.mangas[index].is_favorite = false;
            }
            Msg::GoToDetail(manga_id) => {
                if let Some(div) = self.catalogue_ref.cast::<HtmlElement>() {
                    self.scroll_position = div.scroll_top();
                }
                if let Ok(mut storage) = super::api::get_local_storage() {
                    storage.store(
                        format!("source-{}-mangas", self.source_name).as_str(),
                        Ok(serde_json::to_string(&self.mangas).unwrap()),
                    );
                    storage.store(
                        format!("source-{}-page", self.source_name).as_str(),
                        Ok(self.page.to_string()),
                    );
                    storage.store(
                        format!("source-{}-pos", self.source_name).as_str(),
                        Ok(self.scroll_position.to_string()),
                    );
                }
                self.to_detail(manga_id);
                return false;
            }
            Msg::Noop => {
                return false;
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.source_name != props.clone().source_name {
            self.source_name = props.source_name;
            return true;
        }
        return false;
    }

    fn view(&self) -> Html {
        return html! {
            <div ref={self.catalogue_ref.clone()} id="catalogue" class="pb-20 px-2 overflow-scroll max-h-screen">
                <TopBar>
                    <button onclick=self.link.callback(|_| Msg::SourceLogin)
                        class="hover:text-accent-darker focus:text-accent-darker focus:outline-none rounded flex-none">
                        <svg viewBox="0 0 20 20" fill="currentColor" class="login w-6 h-6 mx-2 self-center flex-none">
                            <path fill-rule="evenodd" d="M3 3a1 1 0 011 1v12a1 1 0 11-2 0V4a1 1 0 011-1zm7.707 3.293a1 1 0 010 1.414L9.414 9H17a1 1 0 110 2H9.414l1.293 1.293a1 1 0 01-1.414 1.414l-3-3a1 1 0 010-1.414l3-3a1 1 0 011.414 0z" clip-rule="evenodd"></path>
                        </svg>
                    </button>
                    <form class="mx-2 flex-grow" onsubmit=self.link.callback(|e| Msg::Search(e))>
                        <input type="search"
                            class="w-full px-3 py-2 focus:outline-none text-sm leading-tight text-white xl:text-black bg-gray-800 xl:bg-gray-100 rounded appearance-none focus:outline-none"
                            placeholder={"Search"}
                            value={self.keyword.clone()}
                            oninput=self.link.callback(|e| Msg::KeywordChanged(e))/>
                    </form>
                    <button onclick=self.link.callback(|_| Msg::Filter) class="hover:text-accent-darker focus:text-accent-darker focus:outline-none rounded flex-none">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                    </button>
                </TopBar>
                {if !self.is_login_page{self.view_mangas()} else {self.view_login_page()}}
                <Filter
                    show={self.show_filter}
                    onsortbychange={self.link.callback(|data| Msg::SortByChange(data))}
                    onsortorderchange={self.link.callback(|data| Msg::SortOrderChange(data))}
                    onclose={self.link.callback(|_| Msg::FilterClosed)}
                    oncancel={self.link.callback(|_| Msg::FilterCancel)}
                    sort_by={&self.sort_by}
                    sort_order={&self.sort_order}
                />
            </div>
        };
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            if self.page == 0 {
                self.fetch_mangas();
            }

            let tmp_link = self.link.clone();
            if let Some(div) = self.catalogue_ref.cast::<HtmlElement>() {
                let cloned_div = div.clone();
                self.closure = Some(Closure::wrap(Box::new(move || {
                    if (cloned_div.scroll_height() - cloned_div.scroll_top())
                        == cloned_div.client_height()
                    {
                        tmp_link.send_message(Msg::ScrolledDown);
                    }
                }) as Box<dyn Fn()>));

                div.set_onscroll(Some(
                    self.closure.as_ref().unwrap().as_ref().unchecked_ref(),
                ));

                if self.scroll_position > 0 {
                    div.scroll_by_with_x_and_y(0.0, self.scroll_position as f64);
                }
            }
        }
    }

    fn destroy(&mut self) {}
}

impl Source {
    fn view_mangas(&self) -> Html {
        let list_link = &WeakComponentLink::<MangaList>::default();
        html! {
            <>
                <MangaList weak_link=list_link style="margin-top: calc(env(safe-area-inset-top) + .5rem)">
                { for self.mangas.iter().enumerate().map(|(idx, manga)| {
                    let id = manga.id;
                    html_nested!{
                    <Manga
                        key=id
                        id=id
                        title=&manga.title
                        thumbnail=&manga.thumbnail_url
                        is_favorite=&manga.is_favorite
                        on_tap=self.link.callback(move |_| Msg::GoToDetail(id))
                        on_long_tap=self.link.callback(move |_| Msg::OnLongTap(idx))/>
                    }})
                }
                </MangaList>
                {
                    match self.is_fetching {
                        true => html!{<Spinner is_active=true is_fullscreen=false />},
                        false => html!{<button class="w-full dark:text-gray-200 dark:text-grey-800 my-2 focus:outline-none" onclick=self.link.callback(|_| Msg::ScrolledDown)>{"Load More"}</button>}
                    }
                }
            </>
        }
    }

    fn view_login_page(&self) -> Html {
        html! {
            <div class="flex justify-center px-2 my-12" style="margin-top: calc(env(safe-area-inset-top) + .5rem)">
                <div class="w-full xl:w-3/4 xl:w-11/12 flex">
                    <div class="w-full p-5">
                        <form class="px-8 pt-6 pb-8 mb-4 rounded" onsubmit=self.link.callback(|e| Msg::Submit(e))>
                            <div class="mb-4">
                                <label class="block mb-2 text-sm font-bold text-gray-700 dark:text-gray-300" for="username">
                                    {"Username"}
                                </label>
                                <input
                                    class="w-full px-3 py-2 text-sm leading-tight bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-700 rounded shadow appearance-none focus:outline-none focus:ring"
                                    id="username"
                                    type="text"
                                    value=self.login.username.to_owned()
                                    oninput=self.link.callback(|e| Msg::UsernameChange(e))
                                />
                            </div>
                            <div class="mb-4">
                                <label class="block mb-2 text-sm font-bold text-gray-700 dark:text-gray-300" for="password">
                                    {"Password"}
                                </label>
                                <input
                                    class="w-full px-3 py-2 mb-3 text-sm bg-white dark:bg-gray-800 leading-tight text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-700 rounded shadow appearance-none focus:outline-none focus:ring"
                                    id="password"
                                    type="password"
                                    value=self.login.password.clone()
                                    oninput=self.link.callback(|e| Msg::PasswordChange(e))
                                />
                                <label class="block mb-2 text-sm font-bold text-gray-700 dark:text-gray-300" for="remember-me">
                                    {"Remember Me"}
                                </label>
                                <input
                                    id="remember-me"
                                    type="checkbox"
                                    value=self.login.remember_me.unwrap_or(false).to_string()
                                    checked=self.login.remember_me.unwrap_or(false)
                                    oninput=self.link.callback(|e| Msg::RememberMeChange(e))
                                />
                                <label class="block mb-2 text-sm font-bold text-gray-700 dark:text-gray-300" for="token">
                                    {"2FA Code"}
                                </label>
                                <input
                                    class="w-full px-3 py-2 mb-3 text-sm leading-tight bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-700 rounded shadow appearance-none focus:outline-none focus:ring"
                                    type="text"
                                    name="token"
                                    id="token"
                                    inputmode="numeric"
                                    value=self.login.two_factor.clone().unwrap_or("".to_string())
                                    oninput=self.link.callback(|e| Msg::TwoFactorChange(e))
                                />
                            </div>
                            <div class="mb-6 text-center">
                            {
                                match self.is_fetching {
                                    true => html!{<Spinner is_active=true is_fullscreen=false />},
                                    false => html!{
                                        <button
                                        class="w-full px-4 py-2 font-bold text-white bg-blue-500 rounded-full hover:bg-blue-700 focus:outline-none focus:ring"
                                        type="submit">
                                        {"Sign In"}
                                    </button>
                                    }
                                }
                            }
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        }
    }

    fn login(&mut self) {
        if let Ok(task) = super::api::post_source_login(
            &self.source_name,
            self.login.clone(),
            self.link
                .callback(move |response: api::FetchJsonResponse<SourceLoginResult>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::LoginReady(data);
                        }
                    }
                    Msg::Noop
                }),
        ) {
            self.fetch_task = Some(task);
            self.is_fetching = true;
        }
    }

    fn fetch_mangas(&mut self) {
        let params = Params {
            keyword: Some(self.keyword.to_owned()),
            sort_by: Some(self.sort_by.clone()),
            sort_order: Some(self.sort_order.clone()),
            page: Some((self.page + 1).to_string()),
            genres: None,
            refresh: match self.page {
                1 => Some(true),
                _ => None,
            },
        };
        if let Ok(task) = super::api::fetch_mangas(
            &self.source_name,
            params,
            self.link.callback(
                move |response: super::api::FetchJsonResponse<GetMangasResponse>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::MangasReady(data);
                        }
                    }
                    Msg::Noop
                },
            ),
        ) {
            self.fetch_task = Some(task);
            self.is_fetching = true;
        }
    }

    fn get_manga_info(&mut self, manga_id: i32) {
        self.worker.send(WorkerRequest::FetchMangaDetail(manga_id))
    }

    fn get_chapters(&mut self, manga_id: i32, _refresh: bool) {
        self.worker.send(WorkerRequest::FetchMangaChapters(manga_id))
    }

    fn favorite(&mut self, index: usize) {
        let manga_id = self.mangas[index].id;
        self.worker.send(WorkerRequest::Favorite(manga_id, index, true));
    }

    fn unfavorite(&mut self, index: usize) {
        let manga_id = self.mangas[index].id;
        self.worker.send(WorkerRequest::Favorite(manga_id, index, false));
    }

    fn to_detail(&mut self, manga_id: i32) {
        self.router.send(RouteRequest::ChangeRoute(Route::from(format!(
                    "/manga/{}",
                    manga_id
                ))));
    }
}
