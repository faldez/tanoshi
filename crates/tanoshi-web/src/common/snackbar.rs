use std::rc::Rc;

use dominator::{clone, html, svg, Dom};
use futures_signals::signal::{Mutable, SignalExt};
use gloo_timers::callback::Timeout;

use crate::common::events;

thread_local! {
    static SNACKBAR: std::cell::RefCell<Rc<Snackbar>> = std::cell::RefCell::new(Snackbar::new());
}

fn close() {
    SNACKBAR.with(|s| s.borrow().close());
}

pub fn show(message: String) {
    SNACKBAR.with(|s| s.borrow().show(message));

    let timeout = Timeout::new(5_000, || {
        close();
    });
    timeout.forget();
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

    pub fn close(&self) {
        self.message.set(None);
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
                            .event(clone!(snackbar => move |_: events::Click| snackbar.close()))
                            .children(&mut [
                                svg!("svg", {
                                    .attr("xmlns", "http://www.w3.org/2000/svg")
                                    .attr("viewBox", "0 0 24 24")
                                    .attr("stroke", "currentColor")
                                    .attr("fill", "none")
                                    .class("icon")
                                    .children(&mut [
                                        svg!("path", {
                                            .attr("stroke-linecap", "round")
                                            .attr("stroke-linejoin", "round")
                                            .attr("stroke-width", "2")
                                            .class("heroicon-ui")
                                            .attr("d", "M6 18L18 6M6 6l12 12")
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
