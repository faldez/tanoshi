use std::rc::Rc;

use dominator::{clone, html, Dom};
use dominator::{routing, with_node};
use futures_signals::signal::Mutable;
use futures_signals::signal::SignalExt;
use web_sys::HtmlInputElement;

use crate::common::{events, snackbar, Route, SettingCategory};
use crate::query;
use crate::utils::AsyncLoader;

pub struct Profile {
    old_password: Mutable<String>,
    new_password: Mutable<String>,
    confirm_password: Mutable<String>,
    telegram_chat_id: Mutable<Option<String>>,
    pub loader: AsyncLoader,
}

impl Profile {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            old_password: Mutable::new("".to_string()),
            new_password: Mutable::new("".to_string()),
            confirm_password: Mutable::new("".to_string()),
            telegram_chat_id: Mutable::new(None),
            loader: AsyncLoader::new(),
        })
    }

    fn fetch_me(profile: Rc<Self>) {
        profile.loader.load(clone!(profile => async move {
            match query::fetch_me().await {
                Ok(result) => profile.telegram_chat_id.set(result.settings.telegram_chat_id.map(|id| id.to_string())),
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    fn test_telegram(profile: Rc<Self>) {
        if let Some(chat_id) = profile
            .telegram_chat_id
            .get_cloned()
            .map(|id| id.parse().unwrap_or_default())
        {
            profile.loader.load(async move {
                match query::test_telegram(chat_id).await {
                    Ok(_) => {}
                    Err(err) => {
                        snackbar::show(format!("{}", err));
                    }
                }
            });
        }
    }

    fn change_password(profile: Rc<Self>) {
        profile.loader.load(clone!(profile => async move {
            let old_password = profile.old_password.get_cloned();
            let new_password = profile.new_password.get_cloned();
            match query::change_password(old_password, new_password).await {
                Ok(_) => {
                    routing::go_to_url(Route::Settings(SettingCategory::None).url().as_str());
                },
                Err(e) => {
                    snackbar::show(format!("change password error: {}", e));
                }
            };
        }));
    }

    fn update_profile(profile: Rc<Self>) {
        profile.loader.load(clone!(profile => async move {
            let telegram_chat_id = profile.telegram_chat_id.get_cloned().and_then(|telegram_chat_id| telegram_chat_id.parse().ok());
            match query::update_profile(telegram_chat_id).await {
                Ok(_) => {
                    routing::go_to_url(Route::Settings(SettingCategory::None).url().as_str());
                },
                Err(e) => {
                    snackbar::show(format!("change password error: {}", e));
                }
            };
        }));
    }

    pub fn render_change_password(profile: Rc<Self>) -> Dom {
        html!("form", {
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("max-width", "1024px")
            .style("margin-left", "auto")
            .style("margin-right", "auto")
            .style("border-radius", "0.5rem")
            .children(&mut [
                html!("input" => HtmlInputElement, {
                    .attribute("type", "password")
                    .attribute("placeholder", "Current Password")
                    .property_signal("value", profile.old_password.signal_cloned())
                    .with_node!(input => {
                        .event(clone!(profile => move |_: events::Input| {
                            profile.old_password.set(input.value());
                        }))
                    })
                }),
                html!("span", {
                    .visible_signal(profile.confirm_password.signal_cloned().map(clone!(profile => move |x| x != profile.new_password.get_cloned())))
                    .style("margin-left", "0.5rem")
                    .style("margin-right", "0.5rem")
                    .style("color", "red")
                    .text("Password do not match")
                }),

                html!("span", {
                    .visible_signal(profile.new_password.signal_cloned().map(|x| x.len() < 8))
                    .style("margin-left", "0.5rem")
                    .style("margin-right", "0.5rem")
                    .style("color", "red")
                    .text("Minimum password length is 8")
                }),
                html!("input" => HtmlInputElement, {
                    .class_signal("error", profile.confirm_password.signal_cloned().map(clone!(profile => move |x| x != profile.new_password.get_cloned())))
                    .attribute("type", "password")
                    .attribute("placeholder", "New Password")
                    .property_signal("value", profile.new_password.signal_cloned())
                    .with_node!(input => {
                        .event(clone!(profile => move |_: events::Input| {
                            profile.new_password.set(input.value());
                        }))
                    })
                }),
                html!("input" => HtmlInputElement, {
                    .class_signal("error", profile.confirm_password.signal_cloned().map(clone!(profile => move |x| x != profile.new_password.get_cloned())))
                    .attribute("type", "password")
                    .attribute("placeholder", "Confirm Password")
                    .property_signal("value", profile.confirm_password.signal_cloned())
                    .with_node!(input => {
                        .event(clone!(profile => move |_: events::Input| {
                            profile.confirm_password.set(input.value());
                        }))
                    })
                }),
                html!("div", {
                    .style("display", "flex")
                    .style("justify-content", "flex-end")
                    .style("margin", "0.5rem")
                    .children(&mut [
                        html!("input", {
                            .attribute("type", "submit")
                            .attribute_signal("disabled", profile.confirm_password.signal_cloned().map(clone!(profile => move |x| {
                                if x != profile.new_password.get_cloned() || x.len() < 8 {
                                    Some("true")
                                } else {
                                    None
                                }
                            })))
                            .text("Submit")
                            .event_preventable(clone!(profile => move |e: events::Click| {
                                e.prevent_default();
                                Self::change_password(profile.clone());
                            }))
                        })
                    ])
                })
            ])
        })
    }

    pub fn render_telegram_setting(profile: Rc<Self>) -> Dom {
        Self::fetch_me(profile.clone());

        html!("form", {
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("max-width", "1024px")
            .style("margin-left", "auto")
            .style("margin-right", "auto")
            .style("border-radius", "0.5rem")
            .children(&mut [
                html!("input" => HtmlInputElement, {
                    .attribute("type", "text")
                    .attribute("placeholder", "Telegram chat id, get from telegram bot")
                    .property_signal("value", profile.telegram_chat_id.signal_cloned().map(|id| id.unwrap_or_else(|| "".to_string())))
                    .with_node!(input => {
                        .event(clone!(profile => move |_: events::Input| {
                            profile.telegram_chat_id.set(Some(input.value()));
                        }))
                    })
                }),
                html!("div", {
                    .style("display", "flex")
                    .style("justify-content", "flex-end")
                    .style("margin", "0.5rem")
                    .children(&mut [
                        html!("input", {
                            .attribute("type", "button")
                            .attribute("value", "Test")
                            .text("Test")
                            .event_preventable(clone!(profile => move |e: events::Click| {
                                e.prevent_default();
                                Self::test_telegram(profile.clone());
                            }))
                        }),
                        html!("input", {
                            .attribute("type", "submit")
                            .text("Submit")
                            .event_preventable(clone!(profile => move |e: events::Click| {
                                e.prevent_default();
                                Self::update_profile(profile.clone());
                            }))
                        })
                    ])
                })
            ])
        })
    }

    pub fn render(profile: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                Self::render_change_password(profile.clone()),
                Self::render_telegram_setting(profile)
            ])
        })
    }
}
