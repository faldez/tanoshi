use serde::{Deserialize, Serialize};
use wasm_bindgen::__rt::core::time::Duration;
use yew::format::{Json, Nothing, Text};
use yew::services::fetch::{FetchTask, Request, Response};
use yew::services::storage::Area;
use yew::services::{FetchService, StorageService, Task, TimeoutService};
use yew::{html, Bridge, Bridged, Component, ComponentLink, Html, Properties, ShouldRender};

use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub is_active: bool,
    pub is_fullscreen: bool,
}

pub struct Spinner {
    is_active: bool,
    is_fullscreen: bool,
}

pub enum Msg {
    Noop,
}

impl Component for Spinner {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Spinner {
            is_active: props.is_active,
            is_fullscreen: props.is_fullscreen,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.is_active != props.is_active {
            self.is_active = props.is_active;
            return true;
        }
        false
    }
    //w-full h-full block m-auto bg-white opacity-75 z-10
    fn view(&self) -> Html {
        html! {
                <div class={
                    if !self.is_active {"hidden"}
                    else if self.is_fullscreen{"w-full h-full fixed flex top-0 left-0 bg-white opacity-75 z-10 justify-center"}
                    else {"w-full h-full block flex m-auto bg-white opacity-75 z-10 justify-center"}
                }>
                    <div class="loader self-center "></div>
                </div>
        }
    }
}
