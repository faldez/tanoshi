use yew::{html, Bridge, Bridged, Component, ComponentLink, Html, ShouldRender};
use yew_router::agent::RouteRequest;
use yew_router::prelude::{Route, RouteAgent};
use yew_router::{router::Router, Switch};

use super::browse::{self, Browse, BrowseRoute};
use super::login::Login;
use super::logout::Logout;
use super::reader::Reader;
use crate::app::component::model::SettingParams;
use web_sys::window;
use yew::services::fetch::FetchTask;

#[derive(Switch, Debug, Clone)]
pub enum AppRoute {
    #[to = "/chapter/{chapter_id}/page/{page}"]
    Reader(i32, usize),
    #[to = "/login"]
    Login,
    #[to = "/logout"]
    Logout,
    #[to = "{*:path}"]
    Browse(BrowseRoute),
}

pub struct App {
    #[allow(dead_code)]
    link: ComponentLink<Self>,
    router: Box<dyn Bridge<RouteAgent>>,
    route: String,
    fetch_task: Option<FetchTask>,
}

pub enum Msg {
    RouterCallback(Route),
    TokenInvalidOrExpired,
    Noop,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let _ = window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .class_list()
            .add_2("bg-white", "dark:bg-black");

        let settings = SettingParams::parse_from_local_storage();
        if settings.dark_mode {
            let _ = window()
                .unwrap()
                .document()
                .unwrap()
                .document_element()
                .unwrap()
                .class_list()
                .add_1("dark");
        }

        let callback = link.callback(|route| Msg::RouterCallback(route));
        let router = RouteAgent::bridge(callback);
        App {
            link,
            router,
            route: "/".to_string(),
            fetch_task: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RouterCallback(route) => {
                self.route = route.route;
            }
            Msg::TokenInvalidOrExpired => {
                self.router
                    .send(RouteRequest::ChangeRoute(Route::from("/login".to_string())));
            }
            Msg::Noop => {
                return false;
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="w-full h-screen">
                <Router<AppRoute, ()>
                render = Router::render(|switch: AppRoute| {
                match switch {
                    AppRoute::Reader(chapter_id, page) => html!{<Reader chapter_id=chapter_id page=page/>},
                    AppRoute::Login => html!{<Login />},
                    AppRoute::Logout => html!{<Logout />},
                    AppRoute::Browse(route) => {
                        let route: browse::Props = route.into();
                        html!{<Browse with route/>}
                    },
                }}) />
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            if let Ok(task) = super::api::validate_token(self.link.callback(
                move |response: super::api::FetchTextResponse| {
                    let (meta, _res) = response.into_parts();
                    let status = meta.status;
                    if status == http::StatusCode::UNAUTHORIZED {
                        return Msg::TokenInvalidOrExpired;
                    }
                    Msg::Noop
                },
            )) {
                self.fetch_task = Some(task);
            } else {
                self.link.send_message(Msg::TokenInvalidOrExpired);
            }
        }
    }
}
