use dominator::{clone, events, html, Dom};
use futures_signals::{
    map_ref,
    signal::{self, Mutable, Signal, SignalExt},
};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use crate::utils::{document, local_storage};

const KEY: &str = "settings:reader";

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum ReaderMode {
    Continous,
    Paged,
}

impl Default for ReaderMode {
    fn default() -> Self {
        ReaderMode::Paged
    }
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum DisplayMode {
    Single,
    Double,
    Auto,
}

impl Default for DisplayMode {
    fn default() -> Self {
        DisplayMode::Single
    }
}

impl DisplayMode {
    fn is_landscape(&self) -> bool {
        let client_width = document()
            .document_element()
            .map(|el| el.client_width())
            .unwrap_or(0_i32);
        let client_height = document()
            .document_element()
            .map(|el| el.client_height())
            .unwrap_or(0_i32);

        client_width > client_height
    }

    pub fn get(&self) -> Self {
        match self {
            DisplayMode::Single => DisplayMode::Single,
            DisplayMode::Double => DisplayMode::Double,
            DisplayMode::Auto => {
                if self.is_landscape() {
                    DisplayMode::Double
                } else {
                    DisplayMode::Single
                }
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::LeftToRight
    }
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Background {
    White,
    Black,
}

impl Default for Background {
    fn default() -> Self {
        Background::White
    }
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Fit {
    Height,
    Width,
    All,
}

impl Default for Fit {
    fn default() -> Self {
        Fit::All
    }
}

pub struct ReaderSettingSignal {
    pub use_modal: bool,
    pub reader_mode: ReaderMode,
    pub padding: bool,
    pub display_mode: DisplayMode,
    pub direction: Direction,
    pub background: Background,
    pub fit: Fit,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ReaderSettings {
    #[serde(skip)]
    use_modal: bool,
    #[serde(skip)]
    first_render: Mutable<bool>,
    #[serde(skip)]
    show: Mutable<bool>,
    #[serde(skip)]
    manga_id: Mutable<i64>,
    pub reader_mode: Mutable<ReaderMode>,
    pub padding: Mutable<bool>,
    pub display_mode: Mutable<DisplayMode>,
    pub direction: Mutable<Direction>,
    pub background: Mutable<Background>,
    pub fit: Mutable<Fit>,
}

impl ReaderSettings {
    pub fn new(show: bool, use_modal: bool) -> Rc<Self> {
        Self::load(show, use_modal, 0)
    }

    pub fn load(show: bool, use_modal: bool, manga_id: i64) -> Rc<Self> {
        let mut key = KEY.to_string();
        if manga_id > 0 {
            key = [key, manga_id.to_string()].join(":");
        }

        let settings = if let Ok(Some(settings)) = local_storage().get_item(&key) {
            serde_json::from_str::<ReaderSettings>(&settings).unwrap_or_default()
        } else {
            ReaderSettings::default()
        };

        Rc::new(ReaderSettings {
            use_modal,
            first_render: Mutable::new(use_modal),
            show: Mutable::new(show),
            manga_id: Mutable::new(manga_id),
            ..settings
        })
    }

    pub fn load_by_manga_id(&self, manga_id: i64) {
        if manga_id == 0 {
            return;
        }

        self.manga_id.replace(manga_id);

        let key = [KEY.to_string(), manga_id.to_string()].join(":");
        let settings = if let Ok(Some(settings)) = local_storage().get_item(&key) {
            serde_json::from_str::<ReaderSettings>(&settings).unwrap_or_default()
        } else {
            return;
        };
        self.reader_mode.replace(settings.reader_mode.get());
        self.padding.replace(settings.padding.get());
        self.display_mode.replace(settings.display_mode.get());
        self.direction.replace(settings.direction.get());
        self.background.replace(settings.background.get());
        self.fit.replace(settings.fit.get());
    }

    pub fn toggle_show(&self) {
        self.show.replace_with(|show| !*show);
        self.first_render.replace_with(|_| false);
    }

    fn save(&self) {
        let mut key = KEY.to_string();
        if *self.manga_id.lock_ref() > 0 {
            key = [key, (*self.manga_id.lock_ref()).to_string()].join(":");
        }

        self.save_with_key(&key);
    }

    fn save_with_key(&self, key: &str) {
        let _ = local_storage().set_item(key, &serde_json::to_string(self).unwrap());
        if self.use_modal {
            self.show.set_neq(false);
        }
    }

    fn reset(&self) {
        let mut key = KEY.to_string();
        if *self.manga_id.lock_ref() > 0 {
            key = [key, (*self.manga_id.lock_ref()).to_string()].join(":");
        }

        let _ = local_storage().delete(&key);
        if self.use_modal {
            self.show.set_neq(false);
        }
    }

    pub fn reader_direction_signal(&self) -> impl Signal<Item = (ReaderMode, Direction)> {
        map_ref! {
            let r = self.reader_mode.signal(),
            let d = self.direction.signal() =>

            (*r, *d)
        }
    }

    pub fn render_apply_button(settings: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("button", {
                    .text("Default")
                    .event(clone!(settings => move |_: events::Click| {
                        settings.reset();
                        settings.save_with_key(KEY);
                    }))
                }),
                html!("button", {
                    .text("Save")
                    .event(clone!(settings => move |_: events::Click| {
                        settings.save();
                    }))
                })
            ])
        })
    }

    pub fn render_header(settings: Rc<Self>) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("justify-content", "space-between")
            .style("margin-bottom", "0.5rem")
            .visible_signal(signal::always(settings.use_modal))
            .children(&mut [
                html!("span", {
                    .style("font-size", "large")
                    .text("Settings")
                }),
                Self::render_apply_button(settings)
            ])
        })
    }

    fn render_reader_mode(settings: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .text("Reader Mode")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", settings.reader_mode.signal_cloned().map(|x| match x {
                                ReaderMode::Continous => true,
                                ReaderMode::Paged => false,
                            }))
                            .text("Continous")
                            .event(clone!(settings => move |_: events::Click| settings.reader_mode.set_neq(ReaderMode::Continous)))
                        }),
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", settings.reader_mode.signal_cloned().map(|x| match x {
                                ReaderMode::Continous => false,
                                ReaderMode::Paged => true,
                            }))
                            .text("Paged")
                            .event(clone!(settings => move |_: events::Click| settings.reader_mode.set_neq(ReaderMode::Paged)))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_padding(settings: Rc<Self>) -> Dom {
        html!("div", {
            .style_signal("opacity", settings.reader_mode.signal_cloned().map(|x| match x {
                ReaderMode::Continous => "1",
                ReaderMode::Paged => "0.5",
            }))
            .attr_signal("disabled", settings.reader_mode.signal_cloned().map(|x| match x {
                ReaderMode::Continous => None,
                ReaderMode::Paged => Some("true"),
            }))
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .style("width", "50%")
                    .text("Padding")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", settings.padding.signal_cloned())
                            .text("Enable")
                            .event(clone!(settings => move |_: events::Click| settings.padding.set_neq(true)))
                        }),
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", settings.padding.signal_cloned().map(|x| !x))
                            .text("Disable")
                            .event(clone!(settings => move |_: events::Click| settings.padding.set_neq(false)))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_display_mode(settings: Rc<Self>) -> Dom {
        html!("div", {
            .style_signal("opacity", settings.reader_mode.signal_cloned().map(|x| match x {
                ReaderMode::Continous => "0.5",
                ReaderMode::Paged => "1",
            }))
            .attr_signal("disabled", settings.reader_mode.signal_cloned().map(|x| match x {
                ReaderMode::Continous => Some("true"),
                ReaderMode::Paged => None,
            }))
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .text("Display Mode")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "33%")
                            .class_signal("active", settings.display_mode.signal_cloned().map(|x| matches!(x, DisplayMode::Single)))
                            .text("Single")
                            .event(clone!(settings => move |_: events::Click| settings.display_mode.set_neq(DisplayMode::Single)))
                        }),
                        html!("button", {
                            .style("width", "33%")
                            .class_signal("active", settings.display_mode.signal_cloned().map(|x| matches!(x, DisplayMode::Double)))
                            .text("Double")
                            .event(clone!(settings => move |_: events::Click| settings.display_mode.set_neq(DisplayMode::Double)))
                        }),
                        html!("button", {
                            .style("width", "33%")
                            .class_signal("active", settings.display_mode.signal_cloned().map(|x| matches!(x, DisplayMode::Auto)))
                            .text("Auto")
                            .event(clone!(settings => move |_: events::Click| settings.display_mode.set_neq(DisplayMode::Auto)))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_direction(settings: Rc<Self>) -> Dom {
        html!("div", {
            .style_signal("opacity", settings.reader_mode.signal_cloned().map(|x| match x {
                ReaderMode::Continous => "0.5",
                ReaderMode::Paged => "1",
            }))
            .attr_signal("disabled", settings.reader_mode.signal_cloned().map(|x| match x {
                ReaderMode::Continous => Some("true"),
                ReaderMode::Paged => None,
            }))
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .text("Direction")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", settings.direction.signal_cloned().map(|x| match x {
                                Direction::LeftToRight => true,
                                Direction::RightToLeft => false,
                            }))
                            .text("Left to Right")
                            .event(clone!(settings => move |_: events::Click| settings.direction.set_neq(Direction::LeftToRight)))
                        }),
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", settings.direction.signal_cloned().map(|x| match x {
                                Direction::LeftToRight => false,
                                Direction::RightToLeft => true,
                            }))
                            .text("Right to Left")
                            .event(clone!(settings => move |_: events::Click| settings.direction.set_neq(Direction::RightToLeft)))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_background(settings: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .text("Background")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", settings.background.signal_cloned().map(|x| match x {
                                Background::Black => true,
                                Background::White => false,
                            }))
                            .text("Black")
                            .event(clone!(settings => move |_: events::Click| settings.background.set_neq(Background::Black)))
                        }),
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", settings.background.signal_cloned().map(|x| match x {
                                Background::Black => false,
                                Background::White => true,
                            }))
                            .text("White")
                            .event(clone!(settings => move |_: events::Click| settings.background.set_neq(Background::White)))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_fit_screen(settings: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .text("Fit")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "33.333333%")
                            .class_signal("active", settings.fit.signal_cloned().map(|x| match x {
                                Fit::Height => true,
                                Fit::Width => false,
                                Fit::All => false
                            }))
                            .text("Height")
                            .event(clone!(settings => move |_: events::Click| settings.fit.set_neq(Fit::Height)))
                        }),
                        html!("button", {
                            .style("width", "33.333333%")
                            .class_signal("active", settings.fit.signal_cloned().map(|x| match x {
                                Fit::Height => false,
                                Fit::Width => true,
                                Fit::All => false
                            }))
                            .text("Width")
                            .event(clone!(settings => move |_: events::Click| settings.fit.set_neq(Fit::Width)))
                        }),
                        html!("button", {
                            .style("width", "33.333333%")
                            .class_signal("active", settings.fit.signal_cloned().map(|x| match x {
                                Fit::Height => false,
                                Fit::Width => false,
                                Fit::All => true
                            }))
                            .text("All")
                            .event(clone!(settings => move |_: events::Click| settings.fit.set_neq(Fit::All)))
                        }),
                    ])
                })
            ])
        })
    }

    fn signal(&self) -> impl Signal<Item = ReaderSettingSignal> {
        map_ref! {
            let use_modal = signal::always(self.use_modal),
            let reader_mode = self.reader_mode.signal_cloned(),
            let padding = self.padding.signal_cloned(),
            let display_mode = self.display_mode.signal_cloned(),
            let direction = self.direction.signal_cloned(),
            let background = self.background.signal_cloned(),
            let fit = self.fit.signal_cloned() =>

            ReaderSettingSignal {
                use_modal: *use_modal,
                reader_mode: *reader_mode,
                padding: *padding,
                display_mode: *display_mode,
                direction: *direction,
                background: *background,
                fit: *fit,
            }
        }
    }

    pub fn render(settings: Rc<Self>) -> Dom {
        let use_modal = settings.use_modal;
        html!("div", {
            .future(settings.signal().for_each(clone!(settings => move |s| {
                if !s.use_modal {
                    settings.save();
                }

                async {}
            })))
            .children(&mut [
                html!("div", {
                    .visible_signal(settings.show.signal().map(move |show| show && use_modal))
                    .class("reader-settings-background")
                    .event(clone!(settings => move |_: events::Click| {
                        settings.show.set_neq(false);
                    }))
                }),
                html!("div", {
                    .class("reader-settings")
                    .class_signal(["modal", "animate__animated", "animate__faster"], signal::always(settings.use_modal))
                    .class_signal("non-modal", signal::always(settings.use_modal).map(|x| !x))
                    .class_signal("animate__slideInUp", settings.show.signal())
                    .class_signal("animate__slideOutDown", settings.show.signal().map(|x| !x))
                    .style_signal("padding-bottom", signal::always(settings.use_modal).map(|use_modal| if use_modal {
                        Some("calc(env(safe-area-inset-bottom) + 0.5rem)")
                    } else {
                        None
                    }))
                    .visible_signal(settings.first_render.signal().map(|x| !x))
                    .children(&mut [
                        Self::render_header(settings.clone()),
                        Self::render_reader_mode(settings.clone()),
                        Self::render_padding(settings.clone()),
                        Self::render_display_mode(settings.clone()),
                        Self::render_direction(settings.clone()),
                        Self::render_background(settings.clone()),
                        Self::render_fit_screen(settings),
                    ])
                })
            ])
        })
    }
}
