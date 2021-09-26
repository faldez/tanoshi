use dominator::{clone, events, html, Dom};
use futures_signals::signal::{Mutable, SignalExt};
use std::rc::Rc;

#[derive(Clone)]
pub struct Modal {
    first_render: Mutable<bool>,
    show: Mutable<bool>,
}

#[allow(dead_code)]
impl Modal {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            first_render: Mutable::new(true),
            show: Mutable::new(false),
        })
    }

    pub fn toggle_show(&self) {
        self.show.replace_with(|show| !*show);
        self.first_render.replace_with(|_| false);
    }

    pub fn render(modal: Rc<Self>, children: Dom) -> Dom {
        html!("div", {
            .children(&mut [
                html!("div", {
                    .visible_signal(modal.show.signal())
                    .class("reader-settings-background")
                    .event(clone!(modal => move |_: events::Click| {
                        modal.show.set_neq(false);
                    }))
                }),
                html!("div", {
                    .class("reader-settings")
                    .class("modal")
                    .class("animate__animated")
                    .class("animate__faster")
                    .class_signal("animate__slideInUp", modal.show.signal())
                    .class_signal("animate__slideOutDown", modal.show.signal().map(|x| !x))
                    .style("padding-bottom", "calc(env(safe-area-inset-bottom) + 0.5rem)")
                    .visible_signal(modal.first_render.signal().map(|x| !x))
                    .children(&mut [
                        children
                    ])
                })
            ])
        })
    }
}
