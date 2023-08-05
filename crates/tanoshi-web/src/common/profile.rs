use std::rc::Rc;

use dominator::{clone, html, routing, Dom};
use dominator::{with_node, EventOptions};
use futures_signals::signal::Mutable;
use futures_signals::signal::SignalExt;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlInputElement, Notification, NotificationPermission};

use crate::common::{events, snackbar, Route};
use crate::query;
use crate::utils::{local_storage, AsyncLoader};

pub struct Profile {
    old_password: Mutable<String>,
    new_password: Mutable<String>,
    confirm_password: Mutable<String>,
    telegram_chat_id: Mutable<Option<String>>,
    pushover_user_key: Mutable<Option<String>>,
    gotify_token: Mutable<Option<String>>,
    myanimelist_status: Mutable<bool>,
    anilist_status: Mutable<bool>,
    notification_cb: Closure<dyn FnMut(JsValue) -> ()>,
    pub loader: AsyncLoader,
}

impl Profile {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            old_password: Mutable::new("".to_string()),
            new_password: Mutable::new("".to_string()),
            confirm_password: Mutable::new("".to_string()),
            telegram_chat_id: Mutable::new(None),
            pushover_user_key: Mutable::new(None),
            gotify_token: Mutable::new(None),
            myanimelist_status: Mutable::new(false),
            anilist_status: Mutable::new(false),
            notification_cb: Closure::wrap(Box::new(|value| {
                let permission = NotificationPermission::from_js_value(&value)
                    .unwrap_or(NotificationPermission::Default);

                debug!("notification: {permission:?}");

                if matches!(permission, NotificationPermission::Granted) {
                    let _notification = Notification::new("Notification enabled");
                }
            })),
            loader: AsyncLoader::new(),
        })
    }

    fn fetch_me(profile: Rc<Self>) {
        profile.loader.load(clone!(profile => async move {
            match query::fetch_me().await {
                Ok(result) => {
                    profile.telegram_chat_id.set(result.telegram_chat_id.map(|id| id.to_string()));
                    profile.pushover_user_key.set(result.pushover_user_key);
                    profile.gotify_token.set(result.gotify_token);
                    profile.myanimelist_status.set(result.myanimelist_status);
                    profile.anilist_status.set(result.anilist_status);
                },
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

    fn test_pushover(profile: Rc<Self>) {
        if let Some(user_key) = profile.pushover_user_key.get_cloned() {
            profile.loader.load(async move {
                match query::test_pushover(&user_key).await {
                    Ok(_) => {}
                    Err(err) => {
                        snackbar::show(format!("{}", err));
                    }
                }
            });
        }
    }

    fn test_gotify(profile: Rc<Self>) {
        if let Some(token) = profile.gotify_token.get_cloned() {
            profile.loader.load(async move {
                match query::test_gotify(&token).await {
                    Ok(_) => {}
                    Err(err) => {
                        snackbar::show(format!("{}", err));
                    }
                }
            });
        }
    }

    fn test_browser_notification(profile: Rc<Self>) {
        profile.loader.load({
            let profile = profile.clone();
            async move {
                match Notification::permission() {
                    NotificationPermission::Denied => {
                        snackbar::show("Notification perimission is not granted".to_string());
                    }
                    NotificationPermission::Granted => {
                        let _notification = Notification::new("Notification enabled");
                    }
                    _ => {
                        let promise = Notification::request_permission()
                            .unwrap_throw()
                            .then(&profile.notification_cb);

                        JsFuture::from(promise).await.unwrap_throw();
                    }
                }
            }
        });
    }

    fn tracker_logout(profile: Rc<Self>, tracker: String) {
        profile.loader.load(clone!(profile => async move {
            match query::tracker_logout(tracker).await {
                Ok(_) => Self::fetch_me(profile),
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }))
    }

    fn change_password(profile: Rc<Self>) {
        profile.loader.load(clone!(profile => async move {
            let old_password = profile.old_password.get_cloned();
            let new_password = profile.new_password.get_cloned();
            match query::change_password(old_password, new_password).await {
                Ok(_) => {
                    // routing::go_to_url(Route::Settings(SettingCategory::None).url().as_str());
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
            let pushover_user_key = profile.pushover_user_key.get_cloned();
            let gotify_token = profile.gotify_token.get_cloned();
            match query::update_profile(telegram_chat_id, pushover_user_key, gotify_token).await {
                Ok(_) => {
                    // routing::go_to_url(Route::Settings(SettingCategory::None).url().as_str());
                },
                Err(e) => {
                    snackbar::show(format!("update profile error: {e}"));
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
            .style("margin-bottom", "0.5rem")
            .style("padding", "0.5rem")
            .style("border-radius", "0.5rem")
            .style("border", "var(--list-group-border)")
            .children(&mut [
                html!("span", {
                    .style("margin-left", "0.25rem")
                    .style("margin-bottom", "0.5rem")
                    .text("Change Password")
                }),
                html!("input" => HtmlInputElement, {
                    .attr("type", "password")
                    .attr("placeholder", "Current Password")
                    .attr("autocomplete", "current-password")
                    .prop_signal("value", profile.old_password.signal_cloned())
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
                    .attr("type", "password")
                    .attr("placeholder", "New Password")
                    .attr("autocomplete", "new-password")
                    .prop_signal("value", profile.new_password.signal_cloned())
                    .with_node!(input => {
                        .event(clone!(profile => move |_: events::Input| {
                            profile.new_password.set(input.value());
                        }))
                    })
                }),
                html!("input" => HtmlInputElement, {
                    .class_signal("error", profile.confirm_password.signal_cloned().map(clone!(profile => move |x| x != profile.new_password.get_cloned())))
                    .attr("type", "password")
                    .attr("placeholder", "Confirm Password")
                    .attr("autocomplete", "new-password")
                    .prop_signal("value", profile.confirm_password.signal_cloned())
                    .with_node!(input => {
                        .event(clone!(profile => move |_: events::Input| {
                            profile.confirm_password.set(input.value());
                        }))
                    })
                }),
                html!("div", {
                    .style("display", "flex")
                    .style("justify-content", "flex-end")
                    .style("margin-top", "0.5rem")
                    .children(&mut [
                        html!("input", {
                            .attr("type", "submit")
                            .attr_signal("disabled", profile.confirm_password.signal_cloned().map(clone!(profile => move |x| {
                                if x != profile.new_password.get_cloned() || x.len() < 8 {
                                    Some("true")
                                } else {
                                    None
                                }
                            })))
                            .text("Submit")
                            .event_with_options(&EventOptions::preventable(), clone!(profile => move |e: events::Click| {
                                e.prevent_default();
                                Self::change_password(profile.clone());
                            }))
                        })
                    ])
                })
            ])
        })
    }

    pub fn render_notification_setting(profile: Rc<Self>) -> Dom {
        html!("form", {
            .class("content")
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("max-width", "1024px")
            .style("margin-left", "auto")
            .style("margin-right", "auto")
            .style("margin-bottom", "0.5rem")
            .style("padding", "0.5rem")
            .style("border-radius", "0.5rem")
            .style("border", "var(--list-group-border)")
            .children(&mut [
                html!("span", {
                    .style("margin-left", "0.25rem")
                    .style("margin-bottom", "0.5rem")
                    .text("Notification")
                }),
                // Browser
                html!("div", {
                    .style("display", "flex")
                    .style("justify-content", "flex-end")
                    .style("margin-right", "0.5rem")
                    .style("margin-top", "0.5rem")
                    .children(&mut [
                        html!("input", {
                            .attr("type", "button")
                            .attr("value", "Test Browser Notification")
                            .text("Test Browser Notification")
                            .event_with_options(&EventOptions::preventable(), clone!(profile => move |e: events::Click| {
                                e.prevent_default();
                                Self::test_browser_notification(profile.clone());
                            }))
                        }),
                    ])
                }),
                // Telegram
                html!("div", {
                    .style("display", "flex")
                    .children(&mut [
                        html!("input" => HtmlInputElement, {
                            .style("width", "100%")
                            .attr("type", "text")
                            .attr("placeholder", "Telegram chat id, get from telegram bot")
                            .prop_signal("value", profile.telegram_chat_id.signal_cloned().map(|id| id.unwrap_or_else(|| "".to_string())))
                            .with_node!(input => {
                                .event(clone!(profile => move |_: events::Input| {
                                    profile.telegram_chat_id.set(Some(input.value()));
                                }))
                            })
                        }),
                        html!("input", {
                            .attr("type", "button")
                            .attr("value", "Test")
                            .text("Test Telegram")
                            .event_with_options(&EventOptions::preventable(), clone!(profile => move |e: events::Click| {
                                e.prevent_default();
                                Self::test_telegram(profile.clone());
                            }))
                        }),
                    ])
                }),
                // Pushover
                html!("div", {
                    .style("display", "flex")
                    .children(&mut [
                        html!("input" => HtmlInputElement, {
                            .style("width", "100%")
                            .attr("type", "text")
                            .attr("placeholder", "Pushover user key, get from pushover dashboard")
                            .prop_signal("value", profile.pushover_user_key.signal_cloned().map(|id| id.unwrap_or_else(|| "".to_string())))
                            .with_node!(input => {
                                .event(clone!(profile => move |_: events::Input| {
                                    profile.pushover_user_key.set(Some(input.value()));
                                }))
                            })
                        }),
                        html!("input", {
                            .attr("type", "button")
                            .attr("value", "Test")
                            .text("Test Pushover")
                            .event_with_options(&EventOptions::preventable(), clone!(profile => move |e: events::Click| {
                                e.prevent_default();
                                Self::test_pushover(profile.clone());
                            }))
                        }),
                    ])
                }),
                // Gotify
                html!("div", {
                    .style("display", "flex")
                    .children(&mut [
                        html!("input" => HtmlInputElement, {
                            .style("width", "100%")
                            .attr("type", "text")
                            .attr("placeholder", "Gotify token, get from Gotify dashboard")
                            .prop_signal("value", profile.gotify_token.signal_cloned().map(|id| id.unwrap_or_else(|| "".to_string())))
                            .with_node!(input => {
                                .event(clone!(profile => move |_: events::Input| {
                                    profile.gotify_token.set(Some(input.value()));
                                }))
                            })
                        }),
                        html!("input", {
                            .attr("type", "button")
                            .attr("value", "Test")
                            .text("Test Gotify")
                            .event_with_options(&EventOptions::preventable(), clone!(profile => move |e: events::Click| {
                                e.prevent_default();
                                Self::test_gotify(profile.clone());
                            }))
                        }),
                    ])
                }),
                html!("div", {
                    .style("display", "flex")
                    .style("justify-content", "flex-end")
                    .style("margin-top", "0.5rem")
                    .children(&mut [
                        html!("input", {
                            .attr("type", "submit")
                            .text("Submit")
                            .event_with_options(&EventOptions::preventable(), clone!(profile => move |e: events::Click| {
                                e.prevent_default();
                                Self::update_profile(profile.clone());
                            }))
                        })
                    ])
                })
            ])
        })
    }

    fn render_tracker_setting(profile: Rc<Self>) -> Dom {
        html!("form", {
            .class("content")
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("max-width", "1024px")
            .style("margin-left", "auto")
            .style("margin-right", "auto")
            .style("margin-bottom", "0.5rem")
            .style("padding", "0.5rem")
            .style("border-radius", "0.5rem")
            .style("border", "var(--list-group-border)")
            .children(&mut [
                html!("span", {
                    .style("margin-left", "0.25rem")
                    .style("margin-bottom", "0.5rem")
                    .text("Tracker")
                }),
                html!("div", {
                    .attr("id", "myanimelist")
                    .style("display", "flex")
                    .style("margin-bottom", "0.5rem")
                    .children(&mut [
                        html!("div", {
                            .style("display", "flex")
                            .style("align-items", "center")
                            .style("width", "100%")
                            .children(&mut [
                                html!("img", {
                                    .style("height", "20px")
                                    .style("width", "20px")
                                    .style("margin-right", "0.5rem")
                                    .attr("src", "https://myanimelist.net/img/common/pwa/launcher-icon-0-75x.png")
                                }),
                                html!("span", {
                                    .text("MyAnimeList")
                                })
                            ])
                        }),
                    ])
                    .child_signal(profile.myanimelist_status.signal_cloned().map(clone!(profile => move |status| if status {
                        Some(html!("button", {
                            .style("color", "red")
                            .text("Logout")
                            .event_with_options(&EventOptions::preventable(), clone!(profile => move |e: events::Click| {
                                e.prevent_default();
                                Self::tracker_logout(profile.clone(), "myanimelist".to_string());
                            }))
                        }))
                    } else {
                        Some(html!("a", {
                            .class("button")
                            .attr("href", &Route::TrackerLogin("myanimelist".to_string()).url())
                            .attr("target", "_blank")
                            .text("Login")
                        }))
                    })))
                }),
                html!("div", {
                    .attr("id", "anilist")
                    .style("display", "flex")
                    .style("margin-bottom", "0.5rem")
                    .children(&mut [
                        html!("div", {
                            .style("display", "flex")
                            .style("align-items", "center")
                            .style("width", "100%")
                            .children(&mut [
                                html!("img", {
                                    .style("height", "20px")
                                    .style("width", "20px")
                                    .style("margin-right", "0.5rem")
                                    .attr("src", "https://upload.wikimedia.org/wikipedia/commons/6/61/AniList_logo.svg")
                                }),
                                html!("span", {
                                    .text("AniList")
                                })
                            ])
                        }),
                    ])
                    .child_signal(profile.anilist_status.signal_cloned().map(clone!(profile => move |status| if status {
                        Some(html!("button", {
                            .style("color", "red")
                            .text("Logout")
                            .event_with_options(&EventOptions::preventable(), clone!(profile => move |e: events::Click| {
                                e.prevent_default();
                                Self::tracker_logout(profile.clone(), "anilist".to_string());
                            }))
                        }))
                    } else {
                        Some(html!("a", {
                            .class("button")
                            .attr("href", &Route::TrackerLogin("anilist".to_string()).url())
                            .attr("target", "_blank")
                            .text("Login")
                        }))
                    })))
                }),
            ])
        })
    }

    pub fn render(profile: Rc<Self>) -> Dom {
        Self::fetch_me(profile.clone());

        html!("div", {
            .children(&mut [
                Self::render_change_password(profile.clone()),
                Self::render_notification_setting(profile.clone()),
                Self::render_tracker_setting(profile),
                html!("div", {
                    .style("max-width", "1024px")
                    .style("margin-left", "auto")
                    .style("margin-right", "auto")
                    .event(|_: events::Click| {
                        local_storage().delete("token").unwrap_throw();
                        routing::go_to_url("/login");
                    })
                    .children(&mut [
                        html!("button", {
                            .class("uninstall-btn")
                            .children(&mut [
                                html!("span", {
                                    .text("Logout")
                                })
                            ])
                        })
                    ])
                })
            ])
        })
    }
}
