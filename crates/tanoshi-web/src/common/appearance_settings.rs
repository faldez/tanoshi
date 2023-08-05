use dominator::{clone, events, html, Dom};
use futures_signals::{
    map_ref,
    signal::{Mutable, Signal, SignalExt},
};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use crate::utils::{apply_theme, local_storage};

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Light => write!(f, "light"),
            Theme::Dark => write!(f, "dark"),
            Theme::Auto => write!(f, ""),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Auto
    }
}

pub struct AppearanceSettingsSignal {
    pub theme: Theme,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub theme: Mutable<Theme>,
}

impl AppearanceSettings {
    pub fn new() -> Rc<Self> {
        let settings = if let Ok(Some(settings)) = local_storage().get_item("settings:appearance") {
            serde_json::from_str::<AppearanceSettings>(&settings).unwrap_or_default()
        } else {
            AppearanceSettings::default()
        };

        Rc::new(settings)
    }

    pub fn save(&self) {
        let theme = self.theme.get().to_string();
        let _ =
            local_storage().set_item("settings:appearance", &serde_json::to_string(self).unwrap());
        let _ = local_storage().set_item("theme", &theme);
    }

    fn signal(&self) -> impl Signal<Item = AppearanceSettingsSignal> {
        map_ref! {
            let theme = self.theme.signal_cloned() =>

            AppearanceSettingsSignal {
                theme: *theme,
            }
        }
    }

    fn render_theme(reader: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .text("Theme")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "33.333333%")
                            .class_signal("active", reader.theme.signal_cloned().map(|x| match x {
                                Theme::Light => true,
                                Theme::Dark => false,
                                Theme::Auto => false
                            }))
                            .text("Light")
                            .event(clone!(reader => move |_: events::Click| reader.theme.set_neq(Theme::Light)))
                        }),
                        html!("button", {
                            .style("width", "33.333333%")
                            .class_signal("active", reader.theme.signal_cloned().map(|x| match x {
                                Theme::Light => false,
                                Theme::Dark => true,
                                Theme::Auto => false
                            }))
                            .text("Dark")
                            .event(clone!(reader => move |_: events::Click| reader.theme.set_neq(Theme::Dark)))
                        }),
                        html!("button", {
                            .style("width", "33.333333%")
                            .class_signal("active", reader.theme.signal_cloned().map(|x| match x {
                                Theme::Light => false,
                                Theme::Dark => false,
                                Theme::Auto => true
                            }))
                            .text("Auto")
                            .event(clone!(reader => move |_: events::Click| reader.theme.set_neq(Theme::Auto)))
                        }),
                    ])
                })
            ])
        })
    }

    pub fn render(reader: Rc<Self>) -> Dom {
        html!("div", {
            .future(reader.signal().for_each(clone!(reader => move |s| {
                reader.save();
                apply_theme(Some(s.theme.to_string()));

                async {}
            })))
            .class("reader-settings")
            .class("non-modal")
            .children(&mut [
                Self::render_theme(reader),
            ])
        })
    }
}
