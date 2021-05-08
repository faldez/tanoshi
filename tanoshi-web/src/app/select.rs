use http::{Request, Response};
use yew::format::{Json, Nothing, Text};
use yew::services::fetch::{FetchService, FetchTask};
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_router::components::RouterAnchor;

use tanoshi_lib::manga::SourceIndex;
use tanoshi_lib::rest::{ErrorResponse, GetSourceIndexResponse};

use super::browse::BrowseRoute;
use super::catalogue::CatalogueRoute;
use super::component::{Spinner, Toast, ToastType, TopBar};

use std::collections::HashMap;

#[derive(Clone, Properties)]
pub struct Props {}

pub struct Select {
    fetch_task: Option<FetchTask>,
    link: ComponentLink<Self>,
    sources: HashMap<bool, Vec<SourceIndex>>,
    is_fetching: bool,
    show_toast: bool,
    toast_message: String,
}

pub enum Msg {
    SourceReady(GetSourceIndexResponse),
    InstallExtension(usize),
    ExtensionInstalled,
    ExtensionError(ErrorResponse),
    Noop,
}

impl Component for Select {
    type Message = Msg;
    type Properties = Props;

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Select {
            fetch_task: None,
            link,
            sources: HashMap::new(),
            is_fetching: false,
            show_toast: false,
            toast_message: "".to_string(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SourceReady(data) => {
                for source in data.sources.iter() {
                    self.sources
                        .entry(source.installed)
                        .and_modify(|s| s.push(source.clone()))
                        .or_insert(vec![source.clone()]);
                }
                self.is_fetching = false;
            }
            Msg::InstallExtension(index) => {
                self.install_source(self.sources[&false][index].name.clone());
            }
            Msg::ExtensionInstalled => {
                self.fetch_sources();
            }
            Msg::ExtensionError(res) => {
                self.show_toast = true;
                self.toast_message = res.message;
                self.is_fetching = false;
            }
            Msg::Noop => {}
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="m-2 pb-20 pt-12">
                <TopBar>
                    <span class="w-full text-center">{"Catalogue"}</span>
                </TopBar>
                <Spinner is_active=self.is_fetching is_fullscreen=true />
                {self.installed_view()}
                {self.available_view()}
                <Toast visible={self.show_toast} toast_type={ToastType::Error} message={&self.toast_message} />
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.fetch_sources();
        }
    }
}

impl Select {
    fn installed_view(&self) -> Html {
        if !self.sources.contains_key(&true) {
            return html! {};
        }
        return html! {
            <>
            <div class="w-full xl:w-1/2 flex justify-start xl:justify-center m-2"><span class="text-black dark:text-white">{"Installed"}</span></div>
            <div class="flex flex-col rounded bg-white dark:bg-gray-900 divide-y divide-gray-100 dark:divide-gray-800" style="margin-top:env(safe-area-inset-top)">
            {
                for self.sources[&true].iter().map(|source| html!{
                    <RouterAnchor<BrowseRoute>
                        classes="flex inline-flex justify-center p-2 content-center hover:bg-gray-100 dark:hover:bg-gray-800"
                        route=BrowseRoute::Catalogue(CatalogueRoute::Source(source.name.clone()))>
                        <div class="w-full xl:w-1/2 flex justify-between items-center text-gray-900 dark:text-gray-300">
                            <div class="flex flex-col">
                                <span class="text-md font-semibold">{source.name.to_owned()}</span>
                                <span class="text-sm">{source.installed_version.to_owned()}</span>
                            </div>
                            <svg viewBox="0 0 20 20" fill="currentColor" class="chevron-right w-6 h-6"><path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"></path></svg>
                        </div>
                    </RouterAnchor<BrowseRoute>>
                })
            }
            </div>
            </>
        };
    }

    fn available_view(&self) -> Html {
        if !self.sources.contains_key(&false) {
            return html! {};
        }
        return html! {
            <>
            <div class="w-full xl:w-1/2 flex justify-center m-2"><span class="text-black dark:text-white">{"Available"}</span></div>
            <div class="flex flex-col bg-white dark:bg-gray-900 divide-y divide-gray-300 dark:divide-gray-700 border-t border-b border-gray-300 dark:border-gray-700" style="margin-top: calc(env(safe-area-inset-top) + .5rem)">
            {
                for (0..self.sources[&false].len()).map(|i| html!{
                    <div class="flex inline-flex justify-center p-2 content-center">
                    <div class="w-full xl:w-1/2 flex justify-between items-center text-gray-900 dark:text-gray-300">
                        <div class="flex flex-col">
                            <span class="text-md font-semibold text-gray-900 dark:text-gray-300">{self.sources[&false][i].name.clone()}</span>
                            <span class="text-sm">{self.sources[&false][i].version.clone()}</span>
                        </div>
                        <button class="bg-tachiyomi-blue text-white p-2 rounded-full focus:outline-none"
                            disabled={!self.sources[&false][i].update && self.sources[&false][i].installed}
                            onclick={self.link.callback(move |_| Msg::InstallExtension(i))}>
                            {
                                if self.sources[&false][i].update {
                                    "Update"
                                } else if !self.sources[&false][i].update && !self.sources[&false][i].installed{
                                    "Install"
                                } else if !self.sources[&false][i].update && self.sources[&false][i].installed {
                                    "Installed"
                                } else {
                                    ""
                                }
                            }
                        </button>
                    </div>
                    </div>
                })
            }
            </div>
            </>
        };
    }

    fn fetch_sources(&mut self) {
        let req = Request::get("/api/source")
            .body(Nothing)
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(
                |response: Response<Json<Result<GetSourceIndexResponse, anyhow::Error>>>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::SourceReady(data);
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

    fn install_source(&mut self, name: String) {
        let req = Request::post(format!("/api/source/install/{}", &name))
            .body(Nothing)
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(|response: Response<Text>| {
                let (meta, body) = response.into_parts();
                if meta.status.is_success() {
                    return Msg::ExtensionInstalled;
                } else {
                    let res: ErrorResponse = serde_json::from_str(&body.unwrap()).unwrap();
                    return Msg::ExtensionError(res);
                }
            }),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
            self.is_fetching = true;
        }
    }
}
