use crate::{
    common::{events, snackbar, Category},
    query,
    utils::AsyncLoader,
};
use dominator::{clone, html, svg, with_node, Dom, EventOptions};

use futures_signals::{
    signal::{Mutable, SignalExt},
    signal_vec::MutableVec,
    signal_vec::SignalVecExt,
};
use std::rc::Rc;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlInputElement;

pub struct SettingsCategories {
    pub is_edit: Mutable<bool>,
    categories: MutableVec<Category>,
    new_category: Mutable<Option<String>>,
    loader: AsyncLoader,
}

impl SettingsCategories {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            is_edit: Mutable::new(false),
            new_category: Mutable::new(None),
            categories: MutableVec::new(),
            loader: AsyncLoader::new(),
        })
    }

    fn create_category(self: &Rc<Self>, name: String) {
        AsyncLoader::new().load({
            let settings = self.clone();
            async move {
                match query::create_category(&name).await {
                    Ok(_) => {
                        settings.new_category.set(None);
                        settings.is_edit.set(false);
                        settings.fetch_categories();
                    }
                    Err(err) => {
                        snackbar::show(format!("{}", err));
                    }
                }
            }
        });
    }

    fn delete_category(self: &Rc<Self>, id: i64) {
        AsyncLoader::new().load({
            let settings = self.clone();
            async move {
                match query::delete_category(id).await {
                    Ok(_) => {
                        settings.fetch_categories();
                    }
                    Err(err) => {
                        snackbar::show(format!("{}", err));
                    }
                }
            }
        });
    }

    fn update_category(self: &Rc<Self>, id: i64, name: String) {
        AsyncLoader::new().load({
            let settings = self.clone();
            async move {
                match query::update_category(id, &name).await {
                    Ok(_) => {
                        settings.is_edit.set(false);
                        settings.fetch_categories();
                    }
                    Err(err) => {
                        snackbar::show(format!("{}", err));
                    }
                }
            }
        });
    }

    pub fn fetch_categories(self: Rc<Self>) {
        let settings = self.clone();
        self.loader.load(clone!(settings => async move {
            match query::fetch_categories().await {
                Ok(res) => {
                    let res: Vec<Category> = res.into_iter().filter_map(|c| (c.id.is_some()).then(|| Category{
                        id: c.id,
                        name: c.name.clone(),
                        count: c.count,
                    })).collect();
                    settings.categories.lock_mut().replace_cloned(res);
                }
                Err(e) => {
                    snackbar::show(format!("failed to fetch categories {}", e));
                }
            }
        }));
    }

    pub fn render(settings: Rc<Self>) -> Dom {
        settings.clone().fetch_categories();
        html!("div", {
            .class("content")
            .children(&mut [
                html!("div",{
                    .style("font-size", "smaller")
                    .style("display", "flex")
                    .style("justify-content", "flex-end")
                }),
                html!("ul", {
                    .class("list")
                    .children_signal_vec(settings.categories.signal_vec_cloned().enumerate().map(clone!(settings => move |(index, cat)|
                        html!("li", {
                            .class("list-item")
                            .style("display", "flex")
                            .style("align-items", "center")
                            .style("padding", "0.25rem")
                            .children(&mut [
                                html!("div", {
                                    .style("width", "100%")
                                    .child_signal(settings.is_edit.signal().map(clone!(settings, index, cat => move |is_edit| {
                                        let dom = if is_edit {
                                            html!("input" => HtmlInputElement, {
                                                .style("width", "100%")
                                                .style_important("background-color", "initial")
                                                .style_important("margin", "0")
                                                .style_important("padding-bottom", "0")
                                                .style_important("padding-top", "0")
                                                .attr("type", "text")
                                                .attr("value", &cat.name)
                                                .with_node!(input => {
                                                    .after_inserted(clone!(input, index => move |_| {
                                                        if index.get().map(|i| i == 0).unwrap_or(false) {
                                                            input.focus().unwrap_throw();
                                                        }
                                                    }))
                                                    .event_with_options(&EventOptions::preventable(), clone!(cat, input, settings => move |event: events::KeyDown| {
                                                        if event.key() == "Enter" {
                                                            event.prevent_default();
                                                            if let Some(cat_id) = cat.id {
                                                                settings.update_category(cat_id, input.value());
                                                            }
                                                        }
                                                    }))
                                                })
                                            })
                                        } else {
                                            html!("div", {
                                                .style("padding", "0.25rem")
                                                .children(&mut [
                                                    html!("span", {
                                                        .style("margin", "0.25rem")
                                                        .text(&cat.name)
                                                    })
                                                ])
                                            })
                                        };

                                        Some(dom)
                                    })))
                                }),
                            ])
                            .child_signal(settings.is_edit.signal().map(clone!(settings => move |is_edit| {
                                is_edit.then(|| html!("button", {
                                    .style("margin-right","0.5rem")
                                    .style("color","darkred")
                                    .event(clone!(cat, settings => move |_: events::Click| {
                                        if let Some(cat_id) = cat.id {
                                            settings.delete_category(cat_id);
                                        }
                                    }))
                                    .children(&mut [
                                        svg!("svg", {
                                            .attr("xmlns", "http://www.w3.org/2000/svg")
                                            .attr("fill", "currentColor")
                                            .attr("viewBox", "0 0 20 20")
                                            .class("icon-sm")
                                            .children(&mut [
                                                svg!("path", {
                                                    .attr("fill-rule", "evenodd")
                                                    .attr("clip-rule", "evenodd")
                                                    .attr("d", "M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z")
                                                })
                                            ])
                                        }),
                                    ])
                                }))
                            })))
                        })
                    )))
                    .child_signal(settings.is_edit.signal().map(clone!(settings => move |is_edit| {
                        is_edit.then(|| html!("li", {
                            .class("list-item")
                            .style("display", "flex")
                            .style("align-items", "center")
                            .children(&mut [
                                html!("input" => HtmlInputElement, {
                                    .style("width", "100%")
                                    .style_important("background-color", "initial")
                                    .attr("type", "text")
                                    .attr("placeholder", "New Category")
                                    .with_node!(input => {
                                        .event(clone!(input, settings => move |_: events::Change| {
                                            settings.new_category.set(Some(input.value()));
                                        }))
                                        .event_with_options(&EventOptions::preventable(), clone!(input, settings => move |event: events::KeyDown| {
                                            if event.key() == "Enter" {
                                                event.prevent_default();
                                                settings.create_category(input.value());
                                            }
                                        }))
                                    })
                                }),
                                html!("button", {
                                    .style("margin-right","0.5rem")
                                    .event(clone!(settings => move |_: events::Click| {
                                        if let Some(new_category_name) = settings.new_category.get_cloned() {
                                            settings.create_category(new_category_name);
                                        }
                                    }))
                                    .children(&mut [
                                        svg!("svg", {
                                            .attr("xmlns", "http://www.w3.org/2000/svg")
                                            .attr("fill", "currentColor")
                                            .attr("viewBox", "0 0 20 20")
                                            .class("icon-sm")
                                            .children(&mut [
                                                svg!("path", {
                                                    .attr("d", "M5 3a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2V5a2 2 0 00-2-2H5zM5 11a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2v-2a2 2 0 00-2-2H5zM11 5a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V5zM14 11a1 1 0 011 1v1h1a1 1 0 110 2h-1v1a1 1 0 11-2 0v-1h-1a1 1 0 110-2h1v-1a1 1 0 011-1z")
                                                })
                                            ])
                                        }),
                                    ])
                                })
                            ])
                        }))
                    })))
                })
            ])
        })
    }
}
