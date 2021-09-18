use crate::{common::{AppearanceSettings, ChapterSettings, Login, Profile, ReaderSettings, Route, SettingCategory, Source, Spinner, User, events, snackbar}, query, utils::{AsyncLoader, window}};
use dominator::svg;
use dominator::{clone, html, link, routing, Dom};
use futures_signals::{signal::{Mutable, SignalExt}, signal_vec::{MutableSignalVec, MutableVec}, signal_vec::SignalVecExt};
use std::rc::Rc;

pub struct Settings {
    server_version: String,
    page: Mutable<SettingCategory>,
    installed_sources: MutableVec<Source>,
    available_sources: MutableVec<Source>,
    me: Mutable<Option<User>>,
    users: MutableVec<User>,
    appearance_settings: Rc<AppearanceSettings>,
    reader_settings: Rc<ReaderSettings>,
    chapter_settings: Rc<ChapterSettings>,
    loader: AsyncLoader,
}

impl Settings {
    pub fn new(server_version: String) -> Rc<Self> {
        Rc::new(Settings {
            server_version,
            page: Mutable::new(SettingCategory::None),
            installed_sources: MutableVec::new(),
            available_sources: MutableVec::new(),
            me: Mutable::new(None),
            users: MutableVec::new(),
            appearance_settings: AppearanceSettings::new(),
            reader_settings: ReaderSettings::new(true, false),
            chapter_settings: ChapterSettings::new(true, false),
            loader: AsyncLoader::new(),
        })
    }

    fn fetch_sources(settings: Rc<Self>) {
        settings.loader.load(clone!(settings => async move {
            match query::fetch_all_sources().await {
                Ok(result) => {
                    settings.installed_sources.lock_mut().replace_cloned(result.installed_sources.iter().map(|s| Source {
                        id: s.id,
                        name: s.name.clone(),
                        version: s.version.clone(),
                        icon: s.icon.clone(),
                        need_login: s.need_login,
                        has_update: s.has_update,
                        installed: true,
                    }).collect());

                    settings.available_sources.lock_mut().replace_cloned(result.available_sources.iter().map(|s| Source {
                        id: s.id,
                        name: s.name.clone(),
                        version: s.version.clone(),
                        icon: s.icon.clone(),
                        need_login: s.need_login,
                        has_update: s.has_update,
                        installed: false,
                    }).collect());                    
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    fn fetch_user_list(settings: Rc<Self>) {
        settings.loader.load(clone!(settings => async move {
            match query::fetch_users().await {
                Ok(result) => {
                    settings.me.set(Some(User{
                        id: result.0.id,
                        username: result.0.username,
                        is_admin: result.0.is_admin,
                        telegram_chat_id: result.0.telegram_chat_id
                    }));

                    settings.users.lock_mut().replace_cloned(result.1.iter().map(|u| User{
                        id: u.id,
                        username: u.username.clone(),
                        is_admin: u.is_admin,
                        telegram_chat_id: None
                    }).collect());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    fn fetch_me(settings: Rc<Self>) {
        settings.loader.load(clone!(settings => async move {
            match query::fetch_me().await {
                Ok(result) => {
                    settings.me.set(Some(User{
                        id: result.id,
                        username: result.username,
                        is_admin: result.is_admin,
                        telegram_chat_id: result.telegram_chat_id
                    }))
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    fn install_source(settings: Rc<Self>, id: i64) {
        settings.loader.load(clone!(settings => async move {
            match query::install_source(id).await {
                Ok(_) => {},
                Err(err) => {
                    snackbar::show(format!("{}", err));
                    return;
                }
            }

            match query::fetch_all_sources().await {
                Ok(result) => {
                    settings.installed_sources.lock_mut().replace_cloned(result.installed_sources.iter().map(|s| Source {
                        id: s.id,
                        name: s.name.clone(),
                        version: s.version.clone(),
                        icon: s.icon.clone(),
                        need_login: s.need_login,
                        has_update: s.has_update,
                        installed: true,
                    }).collect());

                    settings.available_sources.lock_mut().replace_cloned(result.available_sources.iter().map(|s| Source {
                        id: s.id,
                        name: s.name.clone(),
                        version: s.version.clone(),
                        icon: s.icon.clone(),
                        need_login: s.need_login,
                        has_update: s.has_update,
                        installed: false,
                    }).collect());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    fn update_source(settings: Rc<Self>, id: i64) {
        settings.loader.load(clone!(settings => async move {
            match query::update_source(id).await {
                Ok(_) => {},
                Err(err) => {
                    snackbar::show(format!("{}", err));
                    return;
                }
            }

            match query::fetch_all_sources().await {
                Ok(result) => {
                    settings.installed_sources.lock_mut().replace_cloned(result.installed_sources.iter().map(|s| Source {
                        id: s.id,
                        name: s.name.clone(),
                        version: s.version.clone(),
                        icon: s.icon.clone(),
                        need_login: s.need_login,
                        has_update: s.has_update,
                        installed: true,
                    }).collect());

                    settings.available_sources.lock_mut().replace_cloned(result.available_sources.iter().map(|s| Source {
                        id: s.id,
                        name: s.name.clone(),
                        version: s.version.clone(),
                        icon: s.icon.clone(),
                        need_login: s.need_login,
                        has_update: s.has_update,
                        installed: false,
                    }).collect());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    fn uninstall_source(settings: Rc<Self>, id: i64) {
        settings.loader.load(async move {
            match query::uninstall_source(id).await {
                Ok(_) => {
                    routing::go_to_url(&Route::Settings(SettingCategory::Source(0)).url());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        });
    }

    pub fn render_topbar(settings: Rc<Self>) -> Dom {
        html!("div", {
            .class("topbar")
            .children(&mut [
                html!("button", {
                    .style("justify-self", "start")
                    .style_signal("visibility", settings.page.signal_cloned().map(|x|
                        match x {
                            SettingCategory::None => "hidden",
                            _ => "visible"
                        }
                    ))
                    .child_signal(settings.page.signal_cloned().map(|x|
                        match x {
                            SettingCategory::None => None,
                            _ => Some(html!("div", {
                                .style("display", "flex")
                                .style("align-items", "center")
                                .children(&mut [
                                    svg!("svg", {
                                        .class("icon")
                                        .attribute("xmlns", "http://www.w3.org/2000/svg")
                                        .attribute("fill", "none")
                                        .attribute("viewBox", "0 0 24 24")
                                        .attribute("stroke", "currentColor")
                                        .children(&mut [
                                            svg!("path", {
                                                .attribute("stroke-linecap", "round")
                                                .attribute("stroke-linejoin", "round")
                                                .attribute("stroke-width", "2")
                                                .attribute("d", "M15 19l-7-7 7-7")
                                            })
                                        ])
                                    }),
                                    html!("span", {
                                        .text("Settings")
                                    })
                                ])
                            }))
                        }
                    ))
                    .event(|_: events::Click| {
                        let history = window().history().unwrap();
                        if history.length().unwrap() > 1 {
                            let _ = history.back();
                        } else {
                            routing::go_to_url("/");
                        }
                    })
                }),
                html!("span", {
                    .style("overflow", "hidden")
                    .style("text-overflow", "ellipsis")
                    .text_signal(settings.page.signal_cloned().map(|x|
                        match x {
                            SettingCategory::None => "Settings",
                            SettingCategory::Appearance => "Appearance",
                            SettingCategory::General => "General",
                            SettingCategory::Reader => "Reader",
                            SettingCategory::Source(_) => "Sources",
                            SettingCategory::Users => "Users",
                            SettingCategory::CreateUser => "Create User",
                            SettingCategory::User => "User",
                        }
                    ))
                }),
                html!("button", {
                    .style("justify-self", "end")
                    .child_signal(settings.page.signal_cloned().map(move |page| {
                        match page {
                            SettingCategory::Appearance => {
                                Some(AppearanceSettings::render_apply_button(settings.appearance_settings.clone()))
                            }
                            SettingCategory::Users => {
                                Some(link!(Route::Settings(SettingCategory::CreateUser).url(), {
                                    .text("Create User")
                                }))
                            }
                            _ => {
                                None
                            }
                        }
                    }))
                }),
            ])
        })
    }

    pub fn render_categories(settings: Rc<Self>) -> Dom {
        html!("ul", {
            .class(["list", "group"])
            .children(&mut [
                link!(Route::Settings(SettingCategory::Appearance).url(), {
                    .class("list-item")
                    .text("Appearance")
                }),
                link!(Route::Settings(SettingCategory::General).url(), {
                    .class("list-item")
                    .text("General")
                }),
                link!(Route::Settings(SettingCategory::Reader).url(), {
                    .class("list-item")
                    .text("Reader")
                }),
                link!(Route::Settings(SettingCategory::Source(0)).url(), {
                    .class("list-item")
                    .text("Source")
                })
            ])
            .child_signal(settings.me.signal_cloned().map(|me| {
                if let Some(me) = me {
                    if me.is_admin {
                        Some(link!(Route::Settings(SettingCategory::Users).url(), {
                            .class("list-item")
                            .text("Users")
                        }))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }))
        })
    }

    fn render_source_list(title: &str, settings: Rc<Self>, sources_signal: MutableSignalVec<Source>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("h1", {
                    .text(title)
                }),
                html!("ul", {
                    .class([
                        "list",
                        "group"
                    ])
                    .children_signal_vec(sources_signal.map(clone!(settings => move |x|
                        html!("li", {
                            .class("list-item")
                            .children(&mut [
                                link!(Route::Settings(SettingCategory::Source(x.id)).url(), {
                                    .style("display", "flex")
                                    .style("width", "100%")
                                    .children(&mut [
                                        html!("img", {
                                            .style("width", "2.5rem")
                                            .style("height", "2.5rem")
                                            .style("margin-right", "0.5rem")
                                            .attribute("src", &x.icon)
                                        }),
                                        html!("div", {
                                            .style("display", "flex")
                                            .style("flex-direction", "column")
                                            .children(&mut [
                                                html!("span", {
                                                    .text(&x.name)
                                                }),
                                                html!("span", {
                                                    .text(&x.version)
                                                })
                                            ])
                                        })
                                    ])
                                }),
                                if x.installed && x.has_update {
                                    html!("button", {
                                        .text("Update")
                                        .event(clone!(settings => move |_: events::Click| {
                                            Self::update_source(settings.clone(), x.id);
                                        }))
                                    })
                                } else if !x.installed {
                                    html!("button", {
                                        .text("Install")
                                        .event(clone!(settings => move |_: events::Click| {
                                            Self::install_source(settings.clone(), x.id);
                                        }))
                                    })
                                } else {
                                    html!("div", {})
                                }
                            ])
                        })
                    )))
                })
            ])
        })
    }

    pub fn render_source_settings(settings: Rc<Self>, source_id: i64) -> Dom {
        if source_id == 0 {
            html!("div", {
                .children(&mut [
                    Self::render_source_list("Installed", settings.clone(), settings.installed_sources.signal_vec_cloned()),
                    Self::render_source_list("Available", settings.clone(), settings.available_sources.signal_vec_cloned()),
                ])
            })
        } else {
            html!("div", {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("align-items", "center")
                .children_signal_vec(settings.installed_sources.signal_vec_cloned().map(move |source| if source.id == source_id {
                    html!("div", {
                        .style("display", "flex")
                        .style("flex-direction", "column")
                        .style("align-items", "center")
                        .children(&mut [
                            html!("img", {
                                .style("width", "3rem")
                                .style("height", "3rem")
                                .attribute("src", &source.icon)
                            }),
                            html!("span", {
                                .text(&source.name)
                            }),
                            html!("span", {
                                .text(&source.version)
                            })
                        ])
                    })
                } else {
                    html!("div", {})
                }))
                .children(&mut [
                    html!("button", {
                        .class("uninstall-btn")
                        .children(&mut [
                            html!("span", {
                                .text("Uninstall")
                                .event(clone!(settings => move |_: events::Click| {
                                    Self::uninstall_source(settings.clone(), source_id);
                                }))
                            })
                        ])
                    })
                ])
            })
        }
    }

    pub fn render_users_management(settings: Rc<Self>) -> Dom {
        html!("ul", {
            .class(["list", "group"])
            .visible_signal(settings.me.signal_cloned().map(|me| me.map(|me| me.is_admin).unwrap_or(false)))
            .children_signal_vec(settings.users.signal_vec_cloned().map(|x|
                html!("li", {
                    .class("list-item")
                    .style("display", "flex")
                    .style("justify-content", "space-between")
                    .children(&mut [
                        html!("span", {
                            .text(&x.username)
                        }),
                        html!("span", {
                            .text(if x.is_admin { "Admin" } else { "" })
                        })
                    ])
                })
            ))
        })
    }

    pub fn render_user(settings: Rc<Self>) -> Dom {
        link!(Route::Settings(SettingCategory::User).url(), {
            .class("me")
            .children(&mut [
                svg!("svg", {
                    .class("me-icon")
                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                    .attribute("viewBox", "0 0 24 24")
                    .attribute("stroke", "currentColor")
                    .attribute("fill", "none")
                    .children(&mut [
                        svg!("path", {
                            .attribute("stroke-linecap", "round")
                            .attribute("stroke-linejoin", "round")
                            .attribute("stroke-width", "1")
                            .class("heroicon-ui")
                            .attribute("d", "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z")
                        })
                    ])
                }),
            ])
            .child_signal(settings.me.signal_cloned().map(|me| {
                if let Some(me) = me {
                    Some(html!("div", {
                        .class("me-detail")
                        .children(&mut [
                            html!("span",{
                                .class("me-name")
                                .text(&me.username)
                            }),
                            html!("span", {
                                .class("me-role")
                                .text(if me.is_admin { "Admin" } else { "" })
                            })
                        ])
                    }))
                } else {
                    None
                }
            }))
            .children(&mut [
                svg!("svg", {
                    .class("icon")
                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                    .attribute("fill", "none")
                    .attribute("viewBox", "0 0 24 24")
                    .attribute("stroke", "currentColor")
                    .children(&mut [
                        svg!("path", {
                            .attribute("stroke-linecap", "round")
                            .attribute("stroke-linejoin", "round")
                            .attribute("stroke-width", "2")
                            .attribute("d", "M9 5l7 7-7 7")
                        })
                    ])
                })
            ])
        })
    }

    pub fn render(settings: Rc<Self>, category: SettingCategory) -> Dom {
        settings.page.set(category.clone());
        match category {
            SettingCategory::None => Self::fetch_me(settings.clone()),
            SettingCategory::Source(_) => Self::fetch_sources(settings.clone()),
            SettingCategory::Users => Self::fetch_user_list(settings.clone()),
            _ => {}
        }
        html!("div", {
            .class("page")
            .children(&mut [
                Self::render_topbar(settings.clone()),
                html!("div", {
                    .class("topbar-spacing")
                }),
            ])
            .child_signal(settings.loader.is_loading().map(|x| if x {
                Some(Spinner::render_spinner(true))
            } else {
                None
            }))
            .child_signal(settings.page.signal_cloned().map(clone!(settings => move |x|
                match x {
                    SettingCategory::None => Some(html!("div", {
                        .children(&mut [
                            Self::render_user(settings.clone()),
                            Self::render_categories(settings.clone()),
                            html!("text", {
                                .style("font-size", "small")
                                .text(format!("v{}", settings.server_version).as_str())
                            })
                        ])
                    })),
                    SettingCategory::Appearance =>  Some(AppearanceSettings::render(settings.appearance_settings.clone())),
                    SettingCategory::General => Some(html!("div", {
                        .children(&mut [
                            ChapterSettings::render(settings.chapter_settings.clone())
                        ])
                    })),
                    SettingCategory::Reader => Some(ReaderSettings::render(settings.reader_settings.clone())),
                    SettingCategory::Source(source_id) => Some(Self::render_source_settings(settings.clone(), source_id)),
                    SettingCategory::Users => Some(Self::render_users_management(settings.clone())),
                    SettingCategory::User => Some(Profile::render(Profile::new())),
                    SettingCategory::CreateUser => Some(Login::render(Login::new())),
                }
            )))                
        })
    }
}
