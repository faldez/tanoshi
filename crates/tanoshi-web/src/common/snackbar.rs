use std::rc::Rc;

use dominator::{clone, html, svg, Dom};
use futures_signals::signal::{Mutable, SignalExt};

use crate::common::events;

thread_local! {
    static SNACKBAR: std::cell::RefCell<Rc<Snackbar>> = std::cell::RefCell::new(Snackbar::new());
}

pub fn show(message: String) {
    SNACKBAR.with(|s| s.borrow().show(message));
}

pub fn render() -> Dom {
    SNACKBAR.with(|s| Snackbar::render(s.borrow().clone()))
}

pub struct Snackbar {
    message: Mutable<Option<String>>,
}

impl Snackbar {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            message: Mutable::new(None),
        })
    }

    pub fn show(&self, message: String) {
        self.message.set(Some(message));
    }

    pub fn render(snackbar: Rc<Self>) -> Dom {
        html!("div", {
            .class("snackbar")
            .visible_signal(snackbar.message.signal_cloned().map(|message| message.is_some()))
            .children(&mut [
                html!("div", {
                    .child_signal(snackbar.message.signal_cloned().map(|message| message.map(|msg| html!("span", {
                            .text(msg.as_str())
                        })
                    )))
                    .children(&mut [
                        html!("button", {
                            .event(clone!(snackbar => move |_: events::Click| snackbar.message.set(None)))
                            .children(&mut [
                                svg!("svg", {
                                    .attribute("xmlns", "http://www.w3.org/2000/svg")
                                    .attribute("viewBox", "0 0 24 24")
                                    .attribute("stroke", "currentColor")
                                    .attribute("fill", "none")
                                    .class("icon")
                                    .children(&mut [
                                        svg!("path", {
                                            .attribute("stroke-linecap", "round")
                                            .attribute("stroke-linejoin", "round")
                                            .attribute("stroke-width", "2")
                                            .class("heroicon-ui")
                                            .attribute("d", "M6 18L18 6M6 6l12 12")
                                        }),
                                    ])
                                })
                            ])
                        })
                    ])
                })
            ])
        })
    }
}
