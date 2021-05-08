use serde::{Deserialize, Serialize};

use yew::format::Json;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::{storage::Area, StorageService};
use yew::{
    html, Bridge, Bridged, Component, ComponentLink, Html, InputData, Properties, ShouldRender,
};

use yew_router::{agent::RouteRequest, prelude::*};

use web_sys::FocusEvent;

use super::component::model::{Claims, User};
use super::component::{TopBar};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserResponse {
    pub claim: Option<Claims>,
    pub token: Option<String>,
    pub status: String,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {}

pub struct Login {
    fetch_task: Option<FetchTask>,
    link: ComponentLink<Self>,
    router: Box<dyn Bridge<RouteAgent>>,
    storage: StorageService,
    user: User,
}

pub enum Msg {
    UsernameChange(InputData),
    PasswordChange(InputData),
    Submit(FocusEvent),
    LoggedIn(UserResponse),
    Noop,
}

impl Component for Login {
    type Message = Msg;
    type Properties = Props;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let callback = link.callback(|_| Msg::Noop);
        let router = RouteAgent::bridge(callback);
        Login {
            fetch_task: None,
            link,
            router,
            storage,
            user: User {
                username: "".to_string(),
                password: None,
                role: "".to_string(),
                telegram_chat_id: None,
            },
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UsernameChange(e) => {
                self.user.username = e.value;
            }
            Msg::PasswordChange(e) => {
                self.user.password = Some(e.value);
            }
            Msg::Submit(e) => {
                e.prevent_default();
                self.login();
            }
            Msg::LoggedIn(res) => match res.token {
                Some(token) => {
                    self.storage.store("token", Ok(token));
                    self.router
                        .send(RouteRequest::ChangeRoute(Route::from("/".to_string())));
                }
                None => {
                    return false;
                }
            },
            Msg::Noop => {
                return false;
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="pb-20 overflow-scroll max-h-screen" style="margin-top: calc(env(safe-area-inset-top) + 3rem)">
                <TopBar>
                    <span class="mx-auto my-1">{"Login"}</span>
                </TopBar>
                <div class="w-full xl:w-1/2 p-2">
                    <form class="mx-auto" onsubmit=self.link.callback(|e| Msg::Submit(e))>
                        <label class="mb-2 text-sm font-bold text-gray-700 dark:text-gray-300" for="username">
                            {"Username"}
                        </label>
                        <input
                            class="w-full bg-white dark:bg-gray-800 px-3 py-2 mb-3 text-sm leading-tight text-gray-700 dark:text-gray-300 rounded border border-gray-300 dark:border-gray-700 appearance-none focus:outline-none focus:ring"
                            id="username"
                            type="text"
                            value=self.user.username.to_owned()
                            oninput=self.link.callback(|e| Msg::UsernameChange(e))
                        />
                        <label class="mb-2 text-sm font-bold text-gray-700 dark:text-gray-300" for="password">
                            {"Password"}
                        </label>
                        <input
                            class="w-full bg-white dark:bg-gray-800 px-3 py-2 mb-3 text-sm leading-tight text-gray-700 dark:text-gray-300 rounded border border-gray-300 dark:border-gray-700 appearance-none focus:outline-none focus:ring"
                            id="password"
                            type="password"
                            value=self.user.password.clone().unwrap_or("".to_string()).to_owned()
                            oninput=self.link.callback(|e| Msg::PasswordChange(e))
                        />
                        <button
                            class="w-full px-4 py-2 font-bold text-white bg-blue-500 rounded-full hover:bg-blue-700 focus:outline-none focus:ring"
                            type="submit"
                        >
                            {"Sign In"}
                        </button>
                    </form>
                </div>
            </div>
        }
    }
}

impl Login {
    fn login(&mut self) {
        let req = Request::post("/api/login")
            .header("Content-Type", "application/json")
            .body(Json(&self.user))
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(
                |response: Response<Json<Result<UserResponse, anyhow::Error>>>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::LoggedIn(data);
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
