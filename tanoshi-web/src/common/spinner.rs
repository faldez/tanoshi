use dominator::{html, Dom};
use futures_signals::signal::Mutable;
use futures_signals::signal::{SignalExt, Signal};
use std::rc::Rc;

pub struct Spinner {
    pub active: Mutable<bool>,
    fullscreen: Mutable<bool>,
}

impl Spinner {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            active: Mutable::new(false),
            fullscreen: Mutable::new(false),
        })
    }

    pub fn new_with_fullscreen(fullscreen: bool) -> Rc<Self> {
        Rc::new(Self {
            active: Mutable::new(false),
            fullscreen: Mutable::new(fullscreen),
        })
    }

    pub fn set_active(&self, active: bool) {
        self.active.set_neq(active);
    }

    pub fn set_fullscreen(&self, fullscreen: bool) {
        self.fullscreen.set_neq(fullscreen);
    }

    pub fn signal(&self) -> impl Signal<Item = bool> {
        self.active.signal_ref(|x| *x)
    }

    pub fn render(spinner: &Self) -> Dom {
        html!("div", {
            .class("spinner")
            .class_signal("fullscreen", spinner.fullscreen.signal_cloned())
            .visible_signal(spinner.active.signal_cloned())
            .children(&mut [
                html!("div", {
                    .class("loader")
                })
            ])
        })
    }
}
