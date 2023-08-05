use dominator::clone;
use dominator::{html, Dom};
use futures_signals::signal::Signal;
use futures_signals::signal::SignalExt;
use futures_signals::signal::{self, Mutable};
use std::rc::Rc;

use super::events;

pub struct Spinner {
    pub active: Mutable<bool>,
    fullscreen: bool,
    callback: Option<Box<dyn Fn()>>,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            active: Mutable::new(false),
            fullscreen: false,
            callback: None,
        }
    }
}

impl Spinner {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            active: Mutable::new(false),
            fullscreen: false,
            callback: None,
        })
    }

    pub fn new_with_fullscreen_and_callback<F>(fullscreen: bool, f: F) -> Rc<Self>
    where
        F: Fn() + 'static,
    {
        Rc::new(Self {
            active: Mutable::new(false),
            fullscreen,
            callback: Some(Box::new(f)),
        })
    }

    pub fn new_with_fullscreen(fullscreen: bool) -> Rc<Self> {
        Rc::new(Self {
            active: Mutable::new(false),
            fullscreen,
            callback: None,
        })
    }

    pub fn set_active(&self, active: bool) {
        self.active.set_neq(active);
    }

    pub fn signal(&self) -> impl Signal<Item = bool> {
        self.active.signal_ref(|x| *x)
    }

    pub fn render(this: Rc<Self>) -> Dom {
        html!("div", {
            .class("spinner")
            .class_signal("fullscreen", signal::always(this.fullscreen))
            .visible_signal(this.active.signal())
            .children(&mut [
                html!("div", {
                    .class("loader")
                })
            ])
            .child_signal(signal::always(this.fullscreen).map(clone!(this => move |fullscreen| if fullscreen {
                Some(html!("button", {
                    .attr("margin-top", "0.25rem")
                    .event(clone!(this => move |_: events::Click| {
                        if let Some(callback) = this.callback.as_ref() {
                            callback();
                        }
                        this.set_active(false);
                    }))
                    .text("Cancel")
                }))
            } else {
                None
            })))
        })
    }

    pub fn render_spinner(fullscreen: bool) -> Dom {
        if fullscreen {
            html!("div", {
                .class(["spinner", "fullscreen"])
                .children(&mut [
                    html!("div", {
                        .class("loader")
                    }),
                    // html!("button", {
                    //     .attr("margin-top", "0.25rem")
                    //     .event(clone!(this => move |_: events::Click| {
                    //         this.set_active(false);
                    //     }))
                    //     .text("Cancel")
                    // })
                ])
            })
        } else {
            html!("div", {
                .class(["spinner"])
                .children(&mut [
                    html!("div", {
                        .class("loader")
                    })
                ])
            })
        }
    }
}
