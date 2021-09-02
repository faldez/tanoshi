use std::rc::Rc;

use dominator::routing;
use dominator::{clone, html, with_node, Dom};
use futures_signals::signal::{Mutable, SignalExt};
use web_sys::HtmlInputElement;

use crate::common::{events, snackbar, Route, SettingCategory};
use crate::query;
use crate::utils::AsyncLoader;

pub struct Login {
    username: Mutable<String>,
    password: Mutable<String>,
    is_admin: Mutable<bool>,
    pub loader: AsyncLoader,
}

impl Login {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            username: Mutable::new("".to_string()),
            password: Mutable::new("".to_string()),
            is_admin: Mutable::new(false),
            loader: AsyncLoader::new(),
        })
    }

    fn register(login: Rc<Self>) {
        login.loader.load(clone!(login => async move {
            let username = login.username.get_cloned();
            let password = login.password.get_cloned();
            let is_admin = login.is_admin.get();
            match query::user_register(username, password, is_admin).await {
                Ok(_) => {
                    login.username.set("".to_string());
                    login.password.set("".to_string());

                    routing::go_to_url(Route::Settings(SettingCategory::Users).url().as_str());
                }
                Err(e) => {
                    snackbar::show(format!("error register: {}", e));
                }
            }
        }));
    }

    pub fn render(login: Rc<Self>) -> Dom {
        html!("form", {
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("max-width", "1024px")
            .style("margin", "auto")
            // .class([
            //     "flex",
            //     "flex-col",
            //     "max-w-lg",
            //     "mx-auto",
            //     "mt-safe-top",
            //     "bg-white",
            //     "dark:bg-gray-800",
            //     "shadow",
            //     "dark:shadow-none",
            //     "rounded",
            //     "p-2"
            // ])
            .children(&mut [
                html!("input" => HtmlInputElement, {
                    .style("margin", "0.5rem")
                    .style("padding", "0.5rem")
                    .style("border-radius", "0.5rem")
                    // .class([
                    //     "m-2",
                    //     "p-1",
                    //     "focus:outline-none",
                    //     "rounded",
                    //     "bg-white",
                    //     "dark:bg-gray-900",
                    //     "text-black",
                    //     "dark:text-white"
                    // ])
                    .attribute("type", "username")
                    .attribute("placeholder", "Username")
                    .property_signal("value", login.username.signal_cloned())
                    .with_node!(input => {
                        .event(clone!(login => move |_: events::Input| {
                            login.username.set(input.value());
                        }))
                    })
                }),
                html!("input" => HtmlInputElement, {
                    .style("margin", "0.5rem")
                    .style("padding", "0.5rem")
                    .style("border-radius", "0.5rem")
                    // .class([
                    //     "m-2",
                    //     "p-1",
                    //     "focus:outline-none",
                    //     "rounded",
                    //     "bg-white",
                    //     "dark:bg-gray-900",
                    //     "text-black",
                    //     "dark:text-white"
                    // ])
                    .attribute("type", "password")
                    .attribute("placeholder", "Password")
                    .property_signal("value", login.password.signal_cloned())
                    .with_node!(input => {
                        .event(clone!(login => move |_: events::Input| {
                            login.password.set(input.value());
                        }))
                    })
                }),
                html!("label", {
                    .style("display", "flex")
                    .style("align-items", "center")
                    .style("padding", "0.5rem")
                    .children(&mut [
                        html!("input" => HtmlInputElement, {
                            .attribute("type", "checkbox")
                            .attribute_signal("checked", login.is_admin.signal_cloned().map(|x| if x {Some("checked")} else {None}))
                            .with_node!(element => {
                                .event(clone!(login => move |_: events::Change| {
                                    login.is_admin.set_neq(element.checked());
                                }))
                            })
                        })
                    ])
                    .text("Admin")
                }),
                html!("div", {
                    .style("display", "flex")
                    .style("align-items", "center")
                    .style("padding", "0.5rem")
                    .children(&mut [
                        html!("input" => HtmlInputElement, {
                            .attribute("type", "submit")
                            // .class([
                            //     "bg-accent",
                            //     "active:bg-accent-lighter",
                            //     "hover:bg-accent-lighter",
                            //     "focus:outline-none",
                            //     "text-white",
                            //     "px-2",
                            //     "py-1",
                            //     "rounded",
                            //     "focus:outline-none"
                            // ])
                            .text("Create")
                            .event_preventable(clone!(login => move |e: events::Click| {
                                e.prevent_default();
                                Self::register(login.clone());
                            }))
                        })
                    ])
                })
            ])
        })
    }
}
