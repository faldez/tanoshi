use yew::{html, Bridge, Bridged, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_router::prelude::{Route, RouteAgent};
use yew_router::{router::Router, Switch};

use super::catalogue::Catalogue;
use super::component::NavigationBar;
use super::detail::Detail;
use super::history::History;
use super::home::Home;
use super::settings::Settings;
use crate::app::catalogue::CatalogueRoute;
use crate::app::settings::SettingRoute;

#[derive(Switch, Debug, Clone)]
pub enum BrowseRoute {
    #[to = "/manga/{manga_id}"]
    Detail(i32),
    #[to = "/catalogue{*:rest}"]
    Catalogue(CatalogueRoute),
    #[to = "/updates"]
    Updates,
    #[to = "/history"]
    History,
    #[to = "/settings{*:rest}"]
    Settings(SettingRoute),
    #[to = "/"]
    Home,
}
#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub source: Option<String>,
    pub title: Option<String>,
}

impl Into<Props> for BrowseRoute {
    fn into(self) -> Props {
        Props {
            source: None,
            title: None,
        }
    }
}

pub struct Browse {
    props: Props,
    #[allow(dead_code)]
    link: ComponentLink<Self>,
    #[allow(dead_code)]
    router: Box<dyn Bridge<RouteAgent>>,
    route: String,
}

pub enum Msg {
    RouterCallback(Route),
}

impl Component for Browse {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|route| Msg::RouterCallback(route));

        Browse {
            props,
            link,
            router: RouteAgent::bridge(callback),
            route: "/".to_string(),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            return true;
        }
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RouterCallback(route) => {
                self.route = route.route;
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
                <div class="w-full pl-auto xl:pl-48">
                    <div class="block fixed inset-x-0 top-0 z-10 safe-top"></div>
                    <NavigationBar />
                    <Router<BrowseRoute>
                    render = Router::render(|switch: BrowseRoute| {
                    match switch {
                        BrowseRoute::Detail(manga_id) => html!{<Detail manga_id=manga_id/>},
                        BrowseRoute::Catalogue(catalogue_route) => html!{<Catalogue route=catalogue_route/>},
                        BrowseRoute::Updates => html!{<History/>},
                        BrowseRoute::History => html!{<History/>},
                        BrowseRoute::Home => html!{<Home/>},
                        BrowseRoute::Settings(setting_page) => html!{<Settings setting_page=setting_page />},
                    }}) / >
            </div>
        }
    }
}
