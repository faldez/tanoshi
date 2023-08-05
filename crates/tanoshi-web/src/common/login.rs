use std::rc::Rc;

use dominator::{clone, html, with_node, Dom};
use dominator::{routing, EventOptions};
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
            .children(&mut [
                html!("input" => HtmlInputElement, {
                    .style("margin", "0.5rem")
                    .style("padding", "0.5rem")
                    .style("border-radius", "0.5rem")
                    .attr("type", "username")
                    .attr("placeholder", "Username")
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
                    .attr("type", "password")
                    .attr("placeholder", "Password")
                    .with_node!(input => {
                        .event(clone!(login => move |_: events::Input| {
                            login.password.set_neq(input.value());
                        }))
                    })
                }),
                html!("label", {
                    .style("display", "flex")
                    .style("align-items", "center")
                    .style("padding", "0.5rem")
                    .children(&mut [
                        html!("input" => HtmlInputElement, {
                            .attr("type", "checkbox")
                            .attr_signal("checked", login.is_admin.signal_cloned().map(|x| if x {Some("checked")} else {None}))
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
                            .attr("type", "submit")
                            .text("Create")
                            .event_with_options(&EventOptions::preventable(), clone!(login => move |e: events::Click| {
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
