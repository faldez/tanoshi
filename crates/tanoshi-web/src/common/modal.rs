use dominator::{events, html, Dom};
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

    pub fn new_with_default(show: bool) -> Rc<Self> {
        Rc::new(Self {
            first_render: Mutable::new(false),
            show: Mutable::new(show),
        })
    }

    pub fn toggle_show(&self) {
        self.show.replace_with(|show| !*show);
        self.first_render.replace_with(|_| false);
    }

    pub fn show(&self) {
        self.show.set(true);
        self.first_render.replace_with(|_| false);
    }

    pub fn hide(&self) {
        self.show.set(false);
        self.first_render.replace_with(|_| false);
    }

    pub fn render(self: &Rc<Self>, children: &mut [Dom]) -> Dom {
        html!("div", {
            .style_important("overflow", "initial")
            .children(&mut [
                html!("div", {
                    .visible_signal(self.show.signal())
                    .class("reader-settings-background")
                    .event({
                        let modal = self.clone();
                        move |_: events::Click| {
                            modal.show.set_neq(false);
                        }
                    })
                }),
                html!("div", {
                    .class("reader-settings")
                    .class("modal")
                    .class("animate__animated")
                    .class("animate__faster")
                    .class_signal("animate__slideInUp", self.show.signal())
                    .class_signal("animate__slideOutDown", self.show.signal().map(|x| !x))
                    .style("padding-bottom", "calc(env(safe-area-inset-bottom) + 0.5rem)")
                    .style("max-height", "80vh")
                    .style("min-height", "5vh")
                    .visible_signal(self.first_render.signal().map(|x| !x))
                    .children(children)
                })
            ])
        })
    }
}
