use web_sys::HtmlElement;
use yew::format::{Json};
use yew::prelude::*;
use yew::services::fetch::{FetchTask};
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_router::components::RouterAnchor;

use super::component::{Spinner, TopBar};
use crate::app::AppRoute;

use tanoshi_lib::manga::{Chapter as ChapterModel, Manga as MangaModel};
use tanoshi_lib::rest::{AddFavoritesResponse, GetChaptersResponse, GetMangaResponse};

#[derive(Clone, Properties)]
pub struct Props {
    pub manga_id: i32,
}

pub struct Detail {
    fetch_task: Option<FetchTask>,
    link: ComponentLink<Self>,
    manga_id: i32,
    manga: MangaModel,
    chapters: Vec<ChapterModel>,
    is_fetching: bool,
    should_fetch: bool,
    title_ref: NodeRef,
    desc_ref: NodeRef,
}

pub enum Msg {
    MangaReady(GetMangaResponse),
    ChapterReady(GetChaptersResponse),
    Refresh,
    FavoriteEvent,
    Favorited(AddFavoritesResponse),
    Unfavorited(AddFavoritesResponse),
    Noop,
}

impl Component for Detail {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Detail {
            fetch_task: None,
            link,
            manga_id: props.manga_id,
            manga: MangaModel::default(),
            chapters: vec![],
            is_fetching: true,
            should_fetch: true,
            title_ref: NodeRef::default(),
            desc_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::MangaReady(data) => {
                self.manga = data.manga;
                self.get_chapters(false);
                self.is_fetching = false;

                if let Some(title) = self.title_ref.cast::<HtmlElement>() {
                    let _ = title.set_inner_html(self.manga.title.as_str());
                }

                if let Some(desc) = self.desc_ref.cast::<HtmlElement>() {
                    let _ = desc.set_inner_html(
                        self.manga
                            .description
                            .as_ref()
                            .unwrap_or(&"N/A".to_string()),
                    );
                }

                return false;
            }
            Msg::ChapterReady(data) => {
                self.chapters = data.chapters;
                self.is_fetching = false;
            }
            Msg::Refresh => {
                self.get_chapters(true);
            }
            Msg::FavoriteEvent => {
                if self.manga.is_favorite {
                    self.unfavorite();
                } else {
                    self.favorite();
                }
            }
            Msg::Favorited(data) => {
                if data.status == "success" {
                    self.manga.is_favorite = true;
                }
            }
            Msg::Unfavorited(data) => {
                if data.status == "success" {
                    self.manga.is_favorite = false;
                }
            }
            Msg::Noop => {
                return false;
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.manga_id != props.manga_id {
            self.manga_id = props.manga_id;
            self.should_fetch = true;
            return true;
        }
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="pb-20 overflow-scroll max-h-screen" style="margin-top: calc(env(safe-area-inset-top) + 3rem)">
            <TopBar>
                <span ref=self.title_ref.clone() class="mx-auto my-1"></span>
                // <button
                // onclick=self.link.callback(|_| Msg::FavoriteEvent)
                // class="hover:bg-accent-darker rounded flex-none">
                //     <svg class="mx-2 my-auto self-center" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" fill="currentColor">
                //     {
                //         if !self.manga.is_favorite {
                //             html!{<path class="heroicon-ui" d="M6.1 21.98a1 1 0 0 1-1.45-1.06l1.03-6.03-4.38-4.26a1 1 0 0 1 .56-1.71l6.05-.88 2.7-5.48a1 1 0 0 1 1.8 0l2.7 5.48 6.06.88a1 1 0 0 1 .55 1.7l-4.38 4.27 1.04 6.03a1 1 0 0 1-1.46 1.06l-5.4-2.85-5.42 2.85zm4.95-4.87a1 1 0 0 1 .93 0l4.08 2.15-.78-4.55a1 1 0 0 1 .29-.88l3.3-3.22-4.56-.67a1 1 0 0 1-.76-.54l-2.04-4.14L9.47 9.4a1 1 0 0 1-.75.54l-4.57.67 3.3 3.22a1 1 0 0 1 .3.88l-.79 4.55 4.09-2.15z"/>}
                //         } else {
                //             html!{<path class="heroicon-ui" d="m6.1,21.98a1,1 0 0 1 -1.45,-1.059999l1.03,-6.03l-4.38,-4.26a1,1 0 0 1 0.56,-1.71l6.05,-0.88l2.7,-5.48a1,1 0 0 1 1.8,0l2.7,5.48l6.06,0.88a1,1 0 0 1 0.549999,1.7l-4.379999,4.27l1.039999,6.03a1,1 0 0 1 -1.459999,1.059999l-5.4,-2.85l-5.420001,2.85z"/>}
                //         }
                //     }
                //     </svg>
                // </button>
                // <RouterAnchor<AppRoute>
                // classes="hover:bg-accent-darker dark:hover:bg-gray-800 rounded flex-grow text-white text-center my-1 mx-2 px-3 w-full"
                // route=AppRoute::Reader(self.manga.last_read.unwrap_or(self.chapters.last().unwrap_or(&ChapterModel::default()).id), (self.manga.last_page.as_ref().unwrap_or(&0) + 1) as usize)>
                //     //<svg class="fill-current mx-2 my-auto self-center" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path class="heroicon-ui" d="M7 5H5v14h14V5h-2v10a1 1 0 0 1-1.45.9L12 14.11l-3.55 1.77A1 1 0 0 1 7 15V5zM5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5c0-1.1.9-2 2-2zm4 2v8.38l2.55-1.27a1 1 0 0 1 .9 0L15 13.38V5H9z"/></svg>
                //     {"Read"}
                // </RouterAnchor<AppRoute>>
                // <button
                // onclick=self.link.callback(|_| Msg::Refresh)
                // class="hover:bg-accent-darker rounded flex-none">
                //     <svg class="mx-2 my-auto self-center" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" fill="currentColor">
                //         <path class="heroicon-ui" d="M6 18.7V21a1 1 0 0 1-2 0v-5a1 1 0 0 1 1-1h5a1 1 0 1 1 0 2H7.1A7 7 0 0 0 19 12a1 1 0 1 1 2 0 9 9 0 0 1-15 6.7zM18 5.3V3a1 1 0 0 1 2 0v5a1 1 0 0 1-1 1h-5a1 1 0 0 1 0-2h2.9A7 7 0 0 0 5 12a1 1 0 1 1-2 0 9 9 0 0 1 15-6.7z"/>
                //     </svg>
                // </button>
            </TopBar>
            <Spinner is_active={self.is_fetching} is_fullscreen=true />
            <div id="detail" class="flex justify-center p-2 mb-2">
                <div class="w-full xl:w-1/2 flex flex-col">
                    <div class="flex">
                        <div class="pb-7/6">
                            <img class="w-32 md:40 object-cover mr-2 rounded-lg" src=self.manga.thumbnail_url />
                        </div>
                        <div class="flex flex-col">
                            <span class="md:text-xl sm:text-sm text-gray-900 dark:text-gray-300">{"Author"}</span>
                            <span class="md:text-xl sm:text-sm font-semibold text-gray-900 dark:text-gray-300">{self.manga.author.join(", ").to_owned()}</span>
                            <span class="md:text-xl sm:text-sm text-gray-900 dark:text-gray-300">{"Status"}</span>
                            <span class="md:text-xl sm:text-sm font-semibold text-gray-900 dark:text-gray-300">{self.manga.status.as_ref().unwrap_or(&"N/A".to_string()).to_owned()}</span>
                            //<p class="md:text-xl sm:text-sm font-medium break-normal text-gray-900 dark:text-gray-300">{self.manga.genre.join(", ").to_owned()}</p>
                        </div>
                    </div>
                </div>
            </div>
            <div id="detail-description" class="flex flex-col justify-center p-2 mb-2 rounded-t-lg">
                <div class="w-full xl:w-1/2 flex flex-col mx-auto">
                    <div class="inline-flex">
                        <button
                            onclick=self.link.callback(|_| Msg::FavoriteEvent)
                            class={"mr-2 my-2 p-2 self-center rounded text-accent-darker focus:outline-none shadow hover:shadow-lg dark:bg-gray-800 dark:hover:bg-gray-700 focus:outline-none"}>
                            <svg class={format!("h-6 h-6 {}", if self.manga.is_favorite{"fill-current"}else{"stroke-current"})} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" fill={if self.manga.is_favorite{"currentColor"}else{"none"}} />
                            </svg>
                        </button>
                        <button
                            // onclick=self.link.callback(|_| Msg::FavoriteEvent)
                            class={"mr-2 my-2 p-2 self-center rounded text-accent-darker focus:outline-none shadow hover:shadow-lg dark:bg-gray-800 dark:hover:bg-gray-700 focus:outline-none"}>
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
                            </svg>
                        </button>
                        // <button
                        //     onclick=self.link.callback(|_| Msg::FavoriteEvent)
                        //     class={"mr-2 my-2 p-2 self-center rounded text-accent-darker focus:outline-none shadow hover:shadow-lg dark:bg-gray-800 dark:hover:bg-gray-700"}>
                        //     <svg class="h-6 h-6" fill="none" stroke="currentColor" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                        //         <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                        //         <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        //     </svg>
                        // </button>
                    </div>
                    <p ref=self.desc_ref.clone() class="break-normal md:text-base sm:text-xs text-gray-900 dark:text-gray-300"></p>
                    <div class="flex flex-wrap">
                    {
                        for self.manga.genre.iter().map(|g| html!{
                            <div class="mr-1 mt-1 px-2 rounded-full border border-gray-500 dark:border-gray-700 text-black dark:text-gray-300">
                                {g}
                            </div>
                        })
                    }
                    </div>
                </div>
                <span class="w-full xl:w-1/2 mx-auto my-2 text-lg text-gray-700 dark:text-gray-300">{"Chapters"}</span>
                <div class="w-full xl:w-1/2 mx-auto flex flex-col divide-y divide-gray-300 dark:divide-gray-800">
                    {
                        for self.chapters.iter().map(|(chapter)| html!{
                            <RouterAnchor<AppRoute> classes=format!("w-full flex inline-flex justify-between p-2 content-center hover:bg-gray-200 dark:hover:bg-gray-800 {}", if chapter.read.unwrap_or(0) == 0 {"text-gray-900 dark:text-gray-300"} else {"text-gray-500"})
                                route=AppRoute::Reader(chapter.id, (chapter.read.unwrap_or(0) + 1) as usize)>
                                <div class="flex flex-col">
                                    <span class="text-md font-semibold">
                                    {
                                        format!("{}{}{}",
                                        if let Some(v) = &chapter.vol {
                                            format!("Vol. {} ", v)
                                        } else {
                                            "".to_string()
                                        },
                                        if let Some(c) = &chapter.no {
                                            format!("Ch. {} ", c)
                                        } else {
                                            "".to_string()
                                        },
                                        {
                                        let t = chapter.title.as_ref().unwrap();
                                        if !t.is_empty() {
                                            format!("{}", t)
                                        } else {
                                            "".to_string()
                                        }
                                        })
                                    }
                                    </span>
                                    <span class="text-sm">{chapter.uploaded.date()}</span>
                                </div>
                                <svg viewBox="0 0 20 20" fill="currentColor" class="chevron-right w-6 h-6"><path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"></path></svg>
                            </RouterAnchor<AppRoute>>
                        })
                    }
                </div>
            </div>
            </div>
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        if self.should_fetch {
            self.get_manga_info();
            self.should_fetch = false;
        }
    }
}

impl Detail {
    fn get_manga_info(&mut self) {
        if let Ok(task) = super::api::fetch_manga(
            self.manga_id,
            self.link.callback(
                move |response: super::api::FetchJsonResponse<GetMangaResponse>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::MangaReady(data);
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

    fn get_chapters(&mut self, refresh: bool) {
        if let Ok(task) = super::api::fetch_chapters(
            self.manga_id,
            refresh,
            self.link.callback(
                move |response: super::api::FetchJsonResponse<GetChaptersResponse>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::ChapterReady(data);
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

    fn favorite(&mut self) {
        if let Ok(task) = super::api::favorite(
            self.manga_id,
            self.link.callback(
                |response: super::api::FetchJsonResponse<AddFavoritesResponse>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::Favorited(data);
                        }
                    }
                    Msg::Noop
                },
            ),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
        }
    }

    fn unfavorite(&mut self) {
        if let Ok(task) = super::api::unfavorite(
            self.manga_id,
            self.link.callback(
                |response: super::api::FetchJsonResponse<AddFavoritesResponse>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::Unfavorited(data);
                        }
                    }
                    Msg::Noop
                },
            ),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
        }
    }
}
