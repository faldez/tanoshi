use crate::{
    common::{
        AppearanceSettings, 
        ChapterSettings, 
        LibrarySettings,
        Login, 
        Profile, 
        ReaderSettings, 
        Route, 
        SettingCategory, 
        Source, 
        User, 
        events, 
        snackbar, Spinner
    }, 
    query, 
    settings_categories::SettingsCategories, 
    settings_download_queue::SettingsDownloads, 
    utils::{AsyncLoader, is_tauri, window}, settings_source::SettingsSource
};
use dominator::svg;
use dominator::{clone, html, link, routing, Dom};
use futures_signals::{signal::{Mutable, SignalExt, self}, signal_vec::{MutableSignalVec, MutableVec}, signal_vec::SignalVecExt};
use std::rc::Rc;


#[derive(Debug, Clone, Copy)]
enum State {
    Null,
    DeleteUserConfirm(i64)
}

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
    library_settings: Rc<LibrarySettings>,
    category_settings: Rc<SettingsCategories>,
    loader: Rc<AsyncLoader>,
    spinner: Rc<Spinner>,
    state: Mutable<State>
}

impl Settings {
    pub fn new(server_version: String, category: SettingCategory) -> Rc<Self> {
        let loader = Rc::new(AsyncLoader::new());
        let spinner = Spinner::new_with_fullscreen_and_callback(true, clone!(loader => move || {
            loader.cancel();
        }));

        Rc::new(Settings {
            server_version,
            page: Mutable::new(category),
            installed_sources: MutableVec::new(),
            available_sources: MutableVec::new(),
            me: Mutable::new(None),
            users: MutableVec::new(),
            appearance_settings: AppearanceSettings::new(),
            reader_settings: ReaderSettings::new(true, false),
            chapter_settings: ChapterSettings::new(true, false),
            library_settings: LibrarySettings::new(true, false),
            category_settings: SettingsCategories::new(),
            loader,
            spinner,
            state: Mutable::new(State::Null)
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
                        has_update: s.has_update,
                        installed: true,
                    }).collect());

                    settings.available_sources.lock_mut().replace_cloned(result.available_sources.iter().map(|s| Source {
                        id: s.id,
                        name: s.name.clone(),
                        version: s.version.clone(),
                        icon: s.icon.clone(),
                        has_update: false,
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

    fn delete_user(settings: Rc<Self>, user_id: i64) {
        settings.loader.load(clone!(settings => async move {
            match query::delete_user(user_id).await {
                Ok(_) => {
                    Self::fetch_user_list(settings.clone());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }

            settings.state.set(State::Null);
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
        settings.spinner.set_active(true);
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
                        has_update: s.has_update,
                        installed: true,
                    }).collect());

                    settings.available_sources.lock_mut().replace_cloned(result.available_sources.iter().map(|s| Source {
                        id: s.id,
                        name: s.name.clone(),
                        version: s.version.clone(),
                        icon: s.icon.clone(),
                        has_update: false,
                        installed: false,
                    }).collect());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }

            settings.spinner.set_active(false);
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
                        has_update: s.has_update,
                        installed: true,
                    }).collect());

                    settings.available_sources.lock_mut().replace_cloned(result.available_sources.iter().map(|s| Source {
                        id: s.id,
                        name: s.name.clone(),
                        version: s.version.clone(),
                        icon: s.icon.clone(),
                        has_update: false,
                        installed: false,
                    }).collect());
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    pub fn render_topbar(settings: Rc<Self>) -> Dom {
        html!("div", {
            .class("topbar")
            .class_signal("tauri", settings.page.signal_cloned().map(|x|
                matches!(x, SettingCategory::None) && is_tauri()
            ))
            .children(&mut [
                html!("button", {
                    .style("justify-self", "start")
                    .style("min-width", "5.5rem")
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
                                        .attr("xmlns", "http://www.w3.org/2000/svg")
                                        .attr("fill", "none")
                                        .attr("viewBox", "0 0 24 24")
                                        .attr("stroke", "currentColor")
                                        .children(&mut [
                                            svg!("path", {
                                                .attr("stroke-linecap", "round")
                                                .attr("stroke-linejoin", "round")
                                                .attr("stroke-width", "2")
                                                .attr("d", "M15 19l-7-7 7-7")
                                            })
                                        ])
                                    }),
                                    html!("span", {
                                        .text("More")
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
                    .class("title")
                    .style("overflow", "hidden")
                    .style("text-overflow", "ellipsis")
                    .text_signal(settings.page.signal_cloned().map(|x|
                        match x {
                            SettingCategory::None => "More",
                            SettingCategory::Appearance => "Appearance",
                            SettingCategory::Chapters => "Chapters",
                            SettingCategory::Library => "Library",
                            SettingCategory::Category => "Category",
                            SettingCategory::Reader => "Reader",
                            SettingCategory::SourceList => "Sources",
                            SettingCategory::Source(_) => "Sources",
                            SettingCategory::Users => "Users",
                            SettingCategory::CreateUser => "Create User",
                            SettingCategory::User => "User",
                            SettingCategory::DownloadQueue => "Downloads Queue"
                        }
                    ))
                }),
                html!("button", {
                    .child_signal(settings.page.signal_cloned().map(move |page| {
                        match page {
                            SettingCategory::Users => {
                                Some(html!("buttom", {
                                    .event(|_:events::Click| {
                                        routing::go_to_url(&Route::Settings(SettingCategory::CreateUser).url());
                                    })
                                    .children(&mut [
                                        svg!("svg", {
                                            .class("icon")
                                            .attr("xmlns", "http://www.w3.org/2000/svg")
                                            .attr("fill", "none")
                                            .attr("viewBox", "0 0 24 24")
                                            .attr("stroke", "currentColor")
                                            .children(&mut [
                                                svg!("path", {
                                                    .attr("stroke-linecap", "round")
                                                    .attr("stroke-linejoin", "round")
                                                    .attr("stroke-width", "2")
                                                    .attr("d", "M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z")
                                                })
                                            ])
                                        })
                                    ])
                                }))
                            }
                            SettingCategory::Category => {
                                Some(html!("div", {
                                    .style("display", "flex")
                                    .style("justify-content", "flex-end")
                                    .child_signal(settings.category_settings.is_edit.signal().map(clone!(settings => move |is_edit| {
                                        let dom = if is_edit {
                                            html!("button", {
                                                .text("Cancel")
                                                .event(clone!(settings => move |_:events::Click| {
                                                    settings.category_settings.is_edit.set(false);
                                                }))
                                            })
                                        } else {
                                            html!("button", {
                                                .text("Edit")
                                                .event(clone!(settings => move |_:events::Click| {
                                                    settings.category_settings.is_edit.set(true);
                                                }))
                                            })
                                        };

                                        Some(dom)
                                    })))
                                }))
                            }
                            _ => {
                                Some(html!("div", {}))
                            }
                        }
                    }))
                }),
            ])
        })
    }

    pub fn render_categories() -> Dom {
        html!("ul", {
            .class(["list", "group"])
            .style("margin-bottom", "0.5rem")
            .children(&mut [
                link!(Route::Settings(SettingCategory::Appearance).url(), {
                    .class("list-item")
                    .text("Appearance")
                }),
                link!(Route::Settings(SettingCategory::Chapters).url(), {
                    .class("list-item")
                    .text("Chapters")
                }),
                link!(Route::Settings(SettingCategory::Library).url(), {
                    .class("list-item")
                    .text("Library")
                }),
                link!(Route::Settings(SettingCategory::Category).url(), {
                    .class("list-item")
                    .text("Categories")
                }),
                link!(Route::Settings(SettingCategory::Reader).url(), {
                    .class("list-item")
                    .text("Reader")
                })
            ])
        })
    }

    pub fn render_misc(settings: Rc<Self>) -> Dom {
        html!("ul", {
            .class(["list", "group"])
            .style("margin-bottom", "0.5rem")
            .children(&mut [
                link!(Route::Settings(SettingCategory::SourceList).url(), {
                    .class("list-item")
                    .text("Sources")
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
            .child_signal(settings.me.signal_cloned().map(|me| {
                if let Some(me) = me {
                    if me.is_admin {
                        Some(link!(Route::Settings(SettingCategory::DownloadQueue).url(), {
                            .class("list-item")
                            .text("Download Queue")
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

    pub fn render_info() -> Dom {
        html!("ul", {
            .class(["list", "group"])
            .children(&mut [
                html!("a", {
                    .class("list-item")
                    .attr("href", "https://github.com/faldez/tanoshi/blob/main/CHANGELOG.md")
                    .attr("target", "_blank")
                    .text("Changelog")
                }),
                html!("a", {
                    .class("list-item")
                    .attr("href", "https://github.com/faldez/tanoshi")
                    .attr("target", "_blank")
                    .text("Github")
                }),
                html!("a", {
                    .class("list-item")
                    .attr("href", "https://faldez.github.io/tanoshi")
                    .attr("target", "_blank")
                    .text("Website")
                }),
                html!("a", {
                    .class("list-item")
                    .attr("href", "https://discord.gg/wPSEftdDqB")
                    .attr("target", "_blank")
                    .text("Discord")
                }),
            ])
        })
    }

    pub fn render_users_management(settings: Rc<Self>) -> Dom {
        html!("ul", {
            .class(["list", "group"])
            .visible_signal(settings.me.signal_cloned().map(|me| me.map(|me| me.is_admin).unwrap_or(false)))
            .children_signal_vec(settings.users.signal_vec_cloned().map(clone!(settings => move |user|
                html!("li", {
                    .class("list-item")
                    .style("display", "flex")
                    .style("justify-content", "space-between")
                    .children(&mut [
                        html!("div", {
                            .style("display", "flex")
                            .style("align-items", "center")
                            .style("margin-right", "0.5rem")
                            .children(&mut [
                                html!("span", {
                                    .style("margin", "0.5rem")
                                    .text(&user.username)
                                }),
                            ])
                            .child_signal(signal::always(user.is_admin).map(|is_admin| is_admin.then(|| html!("div", {
                                .style("display", "flex")
                                .style("align-items", "center")
                                .style("margin-right", "0.5rem")
                                .children(&mut [
                                    svg!("svg", {
                                        .attr("xmlns", "http://www.w3.org/2000/svg")
                                        .attr("fill", "currentColor")
                                        .attr("viewBox", "0 0 20 20")
                                        .class("icon-sm")
                                        .children(&mut [
                                            svg!("path", {
                                                .attr("fill-rule", "evenodd")
                                                .attr("d", "M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z")
                                                .attr("clip-rule", "evenodd")
                                            })
                                        ])
                                    })
                                ])
                            }))))
                            .child_signal(signal::always(user.is_admin).map(|is_admin| is_admin.then(|| html!("div", {
                                .style("display", "flex")
                                .style("align-items", "center")
                                .style("margin-right", "0.5rem")
                                .children(&mut [
                                    svg!("svg", {
                                        .attr("xmlns", "http://www.w3.org/2000/svg")
                                        .attr("fill", "currentColor")
                                        .attr("viewBox", "0 0 20 20")
                                        .class("icon-sm")
                                        .children(&mut [
                                            svg!("path", {
                                                .attr("fill-rule", "evenodd")
                                                .attr("d", "M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z")
                                                .attr("clip-rule", "evenodd")
                                            })
                                        ])
                                    })
                                ])
                            }))))
                        }),
                    ])
                    .child_signal(settings.state.signal().map(clone!(settings, user => move |state| match state {
                        State::DeleteUserConfirm(user_id) if user_id == user.id  => Some(html!("div", {
                            .style("display", "flex")
                            .style("align-items", "center")
                            .children(&mut [
                                html!("button", {
                                    .event(clone!(settings, user => move |_:events::Click| {
                                        Self::delete_user(settings.clone(), user.id);
                                    }))
                                    .text("Confirm")
                                }),
                                html!("span", {
                                    .text("/")
                                }),
                                html!("button", {
                                    .event(clone!(settings => move |_:events::Click| {
                                        settings.state.set(State::Null);
                                    }))
                                    .text("Cancel")
                                }),
                            ])
                        })),
                        _ if settings.me.get_cloned().map(|me| me.id != user.id).unwrap_or(true) => Some(html!("button", {
                                .attr("id", "remove-user")
                                .children(&mut [
                                    svg!("svg", {
                                        .attr("xmlns", "http://www.w3.org/2000/svg")
                                        .attr("fill", "currentColor")
                                        .attr("viewBox", "0 0 20 20")
                                        .class("icon-sm")
                                        .children(&mut [
                                            svg!("path", {
                                                .attr("fill-rule", "evenodd")
                                                .attr("d", "M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z")
                                                .attr("clip-rule", "evenodd")
                                            })
                                        ])
                                    }),
                                ])
                                .event(clone!(settings, user => move |_:events::Click| {
                                    settings.state.set(State::DeleteUserConfirm(user.id));
                                }))
                            })
                        ),
                        _ => None
                    })))
                })
            )))
        })
    }

    pub fn render_user(settings: Rc<Self>) -> Dom {
        link!(Route::Settings(SettingCategory::User).url(), {
            .class("me")
            .children(&mut [
                svg!("svg", {
                    .class("me-icon")
                    .attr("xmlns", "http://www.w3.org/2000/svg")
                    .attr("viewBox", "0 0 24 24")
                    .attr("stroke", "currentColor")
                    .attr("fill", "none")
                    .children(&mut [
                        svg!("path", {
                            .attr("stroke-linecap", "round")
                            .attr("stroke-linejoin", "round")
                            .attr("stroke-width", "1")
                            .class("heroicon-ui")
                            .attr("d", "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z")
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
                    .attr("xmlns", "http://www.w3.org/2000/svg")
                    .attr("fill", "none")
                    .attr("viewBox", "0 0 24 24")
                    .attr("stroke", "currentColor")
                    .children(&mut [
                        svg!("path", {
                            .attr("stroke-linecap", "round")
                            .attr("stroke-linejoin", "round")
                            .attr("stroke-width", "2")
                            .attr("d", "M9 5l7 7-7 7")
                        })
                    ])
                })
            ])
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
                                if x.installed {
                                    link!(Route::Settings(SettingCategory::Source(x.id)).url(), {
                                        .style("display", "flex")
                                        .style("width", "100%")
                                        .children(&mut [
                                            html!("img", {
                                                .style("width", "2.5rem")
                                                .style("height", "2.5rem")
                                                .style("margin-right", "0.5rem")
                                                .attr("src", &x.icon)
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
                                    })
                                } else {
                                    html!("div", {
                                        .style("display", "flex")
                                        .style("width", "100%")
                                        .children(&mut [
                                            html!("img", {
                                                .style("width", "2.5rem")
                                                .style("height", "2.5rem")
                                                .style("margin-right", "0.5rem")
                                                .attr("src", &x.icon)
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
                                    })
                                },
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

    pub fn render(self: &Rc<Self>) -> Dom {
        match self.page.get() {
            SettingCategory::None => {
                Self::fetch_me(self.clone());
            },
            SettingCategory::SourceList => {
                Self::fetch_sources(self.clone());
            }
            SettingCategory::Users => Self::fetch_user_list(self.clone()),
            _ => {}
        }

        html!("div", {
            .class_signal("main", self.page.signal().map(|x| matches!(x, SettingCategory::None)))
            .style("padding", "0.5rem")
            .children(&mut [
                Self::render_topbar(self.clone()),
                html!("div", {
                    .class("topbar-spacing")
                }),
                Spinner::render(self.spinner.clone()),
            ])
            .child_signal(self.page.signal().map({
                let settings = self.clone();
                move |x|
                match x {
                    SettingCategory::None => Some(html!("div", {
                        .class("content")
                        .children(&mut [
                            Self::render_user(settings.clone()),
                            Self::render_categories(),
                            Self::render_misc(settings.clone()),
                            Self::render_info(),
                            html!("text", {
                                .style("font-size", "small")
                                .text(format!("v{}", settings.server_version).as_str())
                            })
                        ])
                    })),
                    SettingCategory::Appearance =>  Some(AppearanceSettings::render(settings.appearance_settings.clone())),
                    SettingCategory::Chapters => Some(ChapterSettings::render(settings.chapter_settings.clone())),
                    SettingCategory::Library => Some(LibrarySettings::render(settings.library_settings.clone())),
                    SettingCategory::Category => Some(SettingsCategories::render(settings.category_settings.clone())),
                    SettingCategory::Reader => Some(ReaderSettings::render(settings.reader_settings.clone())),
                    SettingCategory::SourceList => Some(html!("div", {
                        .class("content")
                        .children(&mut [
                            Self::render_source_list("Installed", settings.clone(), settings.installed_sources.signal_vec_cloned()),
                            Self::render_source_list("Available", settings.clone(), settings.available_sources.signal_vec_cloned()),
                        ])
                    })),
                    SettingCategory::Source(source_id) => Some(SettingsSource::render(Rc::new(SettingsSource::new(source_id)))),
                    SettingCategory::Users => Some(Self::render_users_management(settings.clone())),
                    SettingCategory::User => Some(Profile::render(Profile::new())),
                    SettingCategory::CreateUser => Some(Login::render(Login::new())),
                    SettingCategory::DownloadQueue => Some(SettingsDownloads::render(SettingsDownloads::new())),
                }
            }))            
        })
    }
}