use std::rc::Rc;

use dominator::{clone, html, Dom};
use futures_signals::{
    map_ref,
    signal::{self, Mutable, Signal, SignalExt},
};
use serde::{Deserialize, Serialize};

use crate::{common::events, utils::local_storage};

const KEY: &str = "settings:library";

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum LibrarySortBy {
    Alphabetical,
    RecentlyRead,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum LibraryOrder {
    Asc,
    Desc,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct LibrarySort {
    pub by: LibrarySortBy,
    pub order: LibraryOrder,
}

impl Default for LibrarySort {
    fn default() -> Self {
        Self {
            by: LibrarySortBy::Alphabetical,
            order: LibraryOrder::Asc,
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum LibraryFilter {
    None,
    Read,
    Unread,
}

impl Default for LibraryFilter {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy)]
pub struct LibrarySettingSignal {
    use_modal: bool,
    pub sort: LibrarySort,
    pub filter: LibraryFilter,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct LibrarySettings {
    #[serde(skip)]
    use_modal: bool,
    #[serde(skip)]
    first_render: Mutable<bool>,
    #[serde(skip)]
    show: Mutable<bool>,
    pub sort: Mutable<LibrarySort>,
    pub filter: Mutable<LibraryFilter>,
}

impl LibrarySettings {
    pub fn new(show: bool, use_modal: bool) -> Rc<Self> {
        Self::load(show, use_modal, 0)
    }

    pub fn load(show: bool, use_modal: bool, manga_id: i64) -> Rc<Self> {
        let mut key = KEY.to_string();
        if manga_id > 0 {
            key = [key, manga_id.to_string()].join(":");
        }

        let settings = if let Ok(Some(settings)) = local_storage().get_item(&key) {
            serde_json::from_str::<Self>(&settings).unwrap_or_default()
        } else {
            Self::default()
        };

        Rc::new(Self {
            use_modal,
            first_render: Mutable::new(use_modal),
            show: Mutable::new(show),
            ..settings
        })
    }

    pub fn toggle_show(&self) {
        self.show.replace_with(|show| !*show);
        self.first_render.replace_with(|_| false);
    }

    fn save(&self) {
        let key = KEY.to_string();
        let _ = local_storage().set_item(&key, &serde_json::to_string(self).unwrap());
        if self.use_modal {
            self.show.set_neq(false);
        }
    }

    pub fn render_apply_button(settings: Rc<Self>) -> Dom {
        html!("button", {
            .text("Save")
            .event(clone!(settings => move |_: events::Click| settings.save()))
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
                    .text("Library")
                }),
                Self::render_apply_button(settings)
            ])
        })
    }

    fn render_sort_setting(settings: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .text("Sort")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", settings.sort.signal_cloned().map(|sort| matches!(sort.by, LibrarySortBy::Alphabetical)))
                            .text("Alphabetical")
                            .event(clone!(settings => move |_: events::Click| settings.sort.set(LibrarySort { by: LibrarySortBy::Alphabetical, order: settings.sort.get_cloned().order})))
                        }),
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", settings.sort.signal_cloned().map(|sort| matches!(sort.by, LibrarySortBy::RecentlyRead)))
                            .text("Recently Read")
                            .event(clone!(settings => move |_: events::Click| settings.sort.set(LibrarySort { by: LibrarySortBy::RecentlyRead, order: settings.sort.get_cloned().order})))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_order_setting(settings: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .text("Order")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", settings.sort.signal_cloned().map(|sort| matches!(sort.order, LibraryOrder::Asc)))
                            .text("Ascending")
                            .event(clone!(settings => move |_: events::Click| settings.sort.set(LibrarySort { by: settings.sort.get_cloned().by, order: LibraryOrder::Asc})))
                        }),
                        html!("button", {
                            .style("width", "50%")
                            .class_signal("active", settings.sort.signal_cloned().map(|sort| matches!(sort.order, LibraryOrder::Desc)))
                            .text("Descending")
                            .event(clone!(settings => move |_: events::Click| settings.sort.set(LibrarySort { by: settings.sort.get_cloned().by, order: LibraryOrder::Desc})))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_filter_setting(settings: Rc<Self>) -> Dom {
        html!("div", {
            .children(&mut [
                html!("label", {
                    .style("margin", "0.5rem")
                    .text("Filter")
                }),
                html!("div", {
                    .class("reader-settings-row")
                    .children(&mut [
                        html!("button", {
                            .style("width", "33%")
                            .class_signal("active", settings.filter.signal_cloned().map(|x| matches!(x, LibraryFilter::None)))
                            .text("None")
                            .event(clone!(settings => move |_: events::Click| settings.filter.set(LibraryFilter::None)))
                        }),
                        html!("button", {
                            .style("width", "33%")
                            .class_signal("active", settings.filter.signal_cloned().map(|x| matches!(x, LibraryFilter::Read)))
                            .text("Read")
                            .event(clone!(settings => move |_: events::Click| settings.filter.set(LibraryFilter::Read)))
                        }),
                        html!("button", {
                            .style("width", "33%")
                            .class_signal("active", settings.filter.signal_cloned().map(|x| matches!(x, LibraryFilter::Unread)))
                            .text("Unread")
                            .event(clone!(settings => move |_: events::Click| settings.filter.set(LibraryFilter::Unread)))
                        }),
                    ])
                })
            ])
        })
    }

    fn signal(&self) -> impl Signal<Item = LibrarySettingSignal> {
        map_ref! {
            let use_modal = signal::always(self.use_modal),
            let sort = self.sort.signal_cloned(),
            let filter = self.filter.signal_cloned() =>

            LibrarySettingSignal {
                use_modal: *use_modal,
                sort: *sort,
                filter: *filter,
            }
        }
    }

    pub fn render(settings: Rc<Self>) -> Dom {
        let use_modal = settings.use_modal;
        html!("div", {
            .future(settings.signal().for_each(clone!(settings => move |s| {
                if !s.use_modal {
                    settings.save();
                    // info!("settings change");
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
                    .class_signal(["modal", "animate__animated"], signal::always(use_modal))
                    .class_signal("non-modal", signal::always(use_modal).map(|x| !x))
                    .class_signal("animate__slideInUp", settings.show.signal())
                    .class_signal("animate__slideOutDown", settings.show.signal().map(|x| !x))
                    .style_signal("padding-bottom", signal::always(use_modal).map(|use_modal| if use_modal {
                        Some("calc(env(safe-area-inset-bottom) + 0.5rem)")
                    } else {
                        None
                    }))
                    .visible_signal(settings.first_render.signal().map(|x| !x))
                    .children(&mut [
                        Self::render_header(settings.clone()),
                        Self::render_sort_setting(settings.clone()),
                        Self::render_order_setting(settings.clone()),
                        Self::render_filter_setting(settings.clone()),
                    ])
                })
            ])
        })
    }
}
