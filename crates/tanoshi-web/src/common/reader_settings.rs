use dominator::{clone, events, html, Dom};
use futures_signals::signal::{Mutable, SignalExt};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use crate::utils::{local_storage, document};

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
        let client_width = document().document_element().map(|el| el.client_width()).unwrap_or(0_i32);
        let client_height = document().document_element().map(|el| el.client_height()).unwrap_or(0_i32);

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

#[derive(PartialEq, Copy, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    pub reader_mode: ReaderMode,
    pub display_mode: DisplayMode,
    pub direction: Direction,
    pub background: Background,
    pub fit: Fit,
}

#[derive(PartialEq, Clone)]
pub struct Page {
    id: i64,
    url: String,
}

pub struct ReaderSettings {
    use_modal: Mutable<bool>,
    first_render: Mutable<bool>,
    show: Mutable<bool>,
    manga_id: Mutable<i64>,
    pub reader_mode: Mutable<ReaderMode>,
    pub display_mode: Mutable<DisplayMode>,
    pub direction: Mutable<Direction>,
    pub background: Mutable<Background>,
    pub fit: Mutable<Fit>,
}

impl ReaderSettings {
    pub fn new(show: bool, use_modal: bool) -> Rc<Self> {
        let settings = if let Ok(Some(settings)) = local_storage().get_item("settings:reader") {
            serde_json::from_str::<Settings>(&settings).unwrap_or_default()
        } else {
            Settings::default()
        };

        Rc::new(ReaderSettings {
            use_modal: Mutable::new(use_modal),
            first_render: Mutable::new(use_modal),
            show: Mutable::new(show),
            manga_id: Mutable::new(0),
            reader_mode: Mutable::new(settings.reader_mode),
            display_mode: Mutable::new(settings.display_mode),
            direction: Mutable::new(settings.direction),
            background: Mutable::new(settings.background),
            fit: Mutable::new(settings.fit),
        })
    }

    pub fn load_manga_reader_setting(&self, manga_id: i64) {
        if manga_id == 0 {
            return;
        }

        self.manga_id.replace(manga_id);

        let key = ["settings:reader", &manga_id.to_string()].join(":");
        let settings = if let Ok(Some(settings)) = local_storage().get_item(&key) {
            serde_json::from_str::<Settings>(&settings).unwrap_or_default()
        } else {
            return;
        };

        self.reader_mode.replace(settings.reader_mode);
        self.display_mode.replace(settings.display_mode);
        self.direction.replace(settings.direction);
        self.background.replace(settings.background);
    }

    pub fn toggle_show(&self) {
        self.show.replace_with(|show| !*show);
        self.first_render.replace_with(|_| false);
    }

    pub fn render_apply_button(reader: Rc<Self>) -> Dom {
        html!("button", {
            .text("Apply")
            .event(clone!(reader => move |_: events::Click| {
                let settings = Settings {
                    reader_mode: reader.reader_mode.get(),
                    display_mode: reader.display_mode.get(),
                    direction: reader.direction.get(),
                    background: reader.background.get(),
                    fit: reader.fit.get(),
                };

                let mut key = "settings:reader".to_string();
                if *reader.manga_id.lock_ref() > 0 {
                    key = [key, (*reader.manga_id.lock_ref()).to_string()].join(":");
                }

                let _ = local_storage().set_item(&key, &serde_json::to_string(&settings).unwrap());
                if *reader.use_modal.lock_ref() {
                    reader.show.set_neq(false);
                }
            }))
        })
    }

    pub fn render_header(reader: Rc<Self>) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("justify-content", "space-between")
            .style("margin-bottom", "0.5rem")
            .visible_signal(reader.use_modal.signal())
            .children(&mut [
                html!("span", {
                    .style("font-size", "large")
                    .text("Settings")
                }),
                Self::render_apply_button(reader)
            ])
        })
    }

    fn render_reader_mode(reader: Rc<Self>) -> Dom {
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
                            .class_signal("active", reader.reader_mode.signal_cloned().map(|x| match x {
                                ReaderMode::Continous => true,
                                ReaderMode::Paged => false,
                            }))
                            .text("Continous")
                            .event(clone!(reader => move |_: events::Click| reader.reader_mode.set_neq(ReaderMode::Continous)))
                        }),
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", reader.reader_mode.signal_cloned().map(|x| match x {
                                ReaderMode::Continous => false,
                                ReaderMode::Paged => true,
                            }))
                            .text("Paged")
                            .event(clone!(reader => move |_: events::Click| reader.reader_mode.set_neq(ReaderMode::Paged)))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_display_mode(reader: Rc<Self>) -> Dom {
        html!("div", {
            .style_signal("opacity", reader.reader_mode.signal_cloned().map(|x| match x {
                ReaderMode::Continous => "0.5",
                ReaderMode::Paged => "1",
            }))
            .attribute_signal("disabled", reader.reader_mode.signal_cloned().map(|x| match x {
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
                            .class_signal("active", reader.display_mode.signal_cloned().map(|x| matches!(x, DisplayMode::Single)))
                            .text("Single")
                            .event(clone!(reader => move |_: events::Click| reader.display_mode.set_neq(DisplayMode::Single)))
                        }),
                        html!("button", {
                            .style("width", "33%")
                            .class_signal("active", reader.display_mode.signal_cloned().map(|x| matches!(x, DisplayMode::Double)))
                            .text("Double")
                            .event(clone!(reader => move |_: events::Click| reader.display_mode.set_neq(DisplayMode::Double)))
                        }),
                        html!("button", {
                            .style("width", "33%")
                            .class_signal("active", reader.display_mode.signal_cloned().map(|x| matches!(x, DisplayMode::Auto)))
                            .text("Auto")
                            .event(clone!(reader => move |_: events::Click| reader.display_mode.set_neq(DisplayMode::Auto)))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_direction(reader: Rc<Self>) -> Dom {
        html!("div", {
            .style_signal("opacity", reader.reader_mode.signal_cloned().map(|x| match x {
                ReaderMode::Continous => "0.5",
                ReaderMode::Paged => "1",
            }))
            .attribute_signal("disabled", reader.reader_mode.signal_cloned().map(|x| match x {
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
                            .class_signal("active", reader.direction.signal_cloned().map(|x| match x {
                                Direction::LeftToRight => true,
                                Direction::RightToLeft => false,
                            }))
                            .text("Left to Right")
                            .event(clone!(reader => move |_: events::Click| reader.direction.set_neq(Direction::LeftToRight)))
                        }),
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", reader.direction.signal_cloned().map(|x| match x {
                                Direction::LeftToRight => false,
                                Direction::RightToLeft => true,
                            }))
                            .text("Right to Left")
                            .event(clone!(reader => move |_: events::Click| reader.direction.set_neq(Direction::RightToLeft)))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_background(reader: Rc<Self>) -> Dom {
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
                            .class_signal("active", reader.background.signal_cloned().map(|x| match x {
                                Background::Black => true,
                                Background::White => false,
                            }))
                            .text("Black")
                            .event(clone!(reader => move |_: events::Click| reader.background.set_neq(Background::Black)))
                        }),
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", reader.background.signal_cloned().map(|x| match x {
                                Background::Black => false,
                                Background::White => true,
                            }))
                            .text("White")
                            .event(clone!(reader => move |_: events::Click| reader.background.set_neq(Background::White)))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_fit_screen(reader: Rc<Self>) -> Dom {
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
                            .class_signal("active", reader.fit.signal_cloned().map(|x| match x {
                                Fit::Height => true,
                                Fit::Width => false,
                                Fit::All => false
                            }))
                            .text("Height")
                            .event(clone!(reader => move |_: events::Click| reader.fit.set_neq(Fit::Height)))
                        }),
                        html!("button", {
                            .style("width", "33.333333%")
                            .class_signal("active", reader.fit.signal_cloned().map(|x| match x {
                                Fit::Height => false,
                                Fit::Width => true,
                                Fit::All => false
                            }))
                            .text("Width")
                            .event(clone!(reader => move |_: events::Click| reader.fit.set_neq(Fit::Width)))
                        }),
                        html!("button", {
                            .style("width", "33.333333%")
                            .class_signal("active", reader.fit.signal_cloned().map(|x| match x {
                                Fit::Height => false,
                                Fit::Width => false,
                                Fit::All => true
                            }))
                            .text("All")
                            .event(clone!(reader => move |_: events::Click| reader.fit.set_neq(Fit::All)))
                        }),
                    ])
                })
            ])
        })
    }

    pub fn render(reader: Rc<Self>) -> Dom {
        let use_modal = reader.use_modal.get();
        html!("div", {
            .children(&mut [
                html!("div", {
                    .visible_signal(reader.show.signal().map(move |show| show && use_modal))
                    .class("reader-settings-background")
                    .event(clone!(reader => move |_: events::Click| {
                        reader.show.set_neq(false);
                    }))
                }),
                html!("div", {
                    .class("reader-settings")
                    .class_signal(["modal", "animate__animated"], reader.use_modal.signal())
                    .class_signal("non-modal", reader.use_modal.signal().map(|x| !x))
                    .class_signal("animate__slideInUp", reader.show.signal())
                    .class_signal("animate__slideOutDown", reader.show.signal().map(|x| !x))
                    .visible_signal(reader.first_render.signal().map(|x| !x))
                    .children(&mut [
                        Self::render_header(reader.clone()),
                        Self::render_reader_mode(reader.clone()),
                        Self::render_display_mode(reader.clone()),
                        Self::render_direction(reader.clone()),
                        Self::render_background(reader.clone()),
                        Self::render_fit_screen(reader.clone()),
                    ])
                    .child_signal(reader.use_modal.signal().map(|use_modal| if use_modal {
                        Some(html!("div", {
                            .class("bottombar-spacing")
                        }))
                    } else {
                        None
                    }))
                })
            ])
        })
    }
}
