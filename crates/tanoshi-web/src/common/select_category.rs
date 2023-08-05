
use crate::{common::{Modal,snackbar}, query, utils::{AsyncLoader,}};
use dominator::{Dom, clone, events, html, with_node};
use futures_signals::{signal::{ Mutable, SignalExt}, signal_vec::{MutableVec, SignalVecExt}};
use std::rc::Rc;
use web_sys::HtmlInputElement;

#[derive(Clone)]
pub struct Category {
    id: Option<i64>,
    name: String,
    selected: Mutable<bool>
}

pub struct SelectCategoryModal {
    categories: MutableVec<Category>,
    modal: Rc<Modal>,
    loader: AsyncLoader,
}

impl SelectCategoryModal {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            categories: MutableVec::new(),
            modal: Modal::new(),
            loader: AsyncLoader::new(),
        })
    }

    pub fn fetch_categories<F>(self: &Rc<Self>, f: F) where F: Fn(Vec<i64>) + Clone + 'static {
        let select = self.clone();
        self.loader.load(clone!(select => async move {
            match query::fetch_categories().await {
                Ok(res) => {
                    if res.len() == 1 {
                        f(vec![]);
                        return;
                    }
                    
                    select.modal.show();
                    select.categories.lock_mut().replace_cloned(res.into_iter().filter_map(|c| (c.id.is_some()).then(|| Category{
                        id: c.id,
                        name: c.name.clone(),
                        selected: Mutable::new(false),
                    })).collect());
                }
                Err(e) => {
                    snackbar::show(format!("failed to fetch categories {}", e));
                }
            }
        }));
    }

    pub fn render_header<F>(self: &Rc<Self>, f: F) -> Dom where F: Fn(Vec<i64>) + Clone + 'static {
        let select = self.clone();
        html!("div", {
            .style("display", "flex")
            .style("justify-content", "space-between")
            .style("margin-bottom", "0.5rem")
            .children(&mut [
                html!("span", {
                    .style("font-size", "large")
                    .text("Select Categories")
                }),
                html!("button", {
                    .text("OK")
                    .event(clone!(select, f => move |_: events::Click| {
                        let category_ids = select.categories.lock_ref().iter().filter_map(|cat| if cat.selected.get() { cat.id } else { None }).collect();
                        f(category_ids);
                        select.modal.hide();
                    }))
                })
            ])
        })
    }

    pub fn render_main(self: &Rc<Self>) -> Dom {
        let select = self.clone();
        html!("ul", {
            .class("list")
            .children_signal_vec(select.categories.signal_vec_cloned().map(|cat| html!{"li", {
                .class("list-item")
                .style("padding", "0.5rem")
                .children(&mut [
                    html!("input" => HtmlInputElement, {
                        .attr("type", "checkbox")
                        .style("height", "0.75rem")
                        .style("width", "0.75rem")
                        .style("margin-left", "0.5rem")
                        .style("margin-right", "0.5rem")
                        .style("margin-top", "auto")
                        .style("margin-bottom", "auto")
                        .with_node!(input => {
                            .future(cat.selected.signal().for_each(clone!(input => move |selected| {
                                input.set_checked(selected);

                                async{}
                            })))
                            .event(clone!(cat => move |_: events::Change| {
                                cat.selected.set_neq(input.checked());
                            }))
                        })
                    }),
                    html!("span", {
                        .style("margin-left", "0.5rem")
                        .text(&cat.name)
                    })
                ])
                .event(clone!(cat => move |_: events::Click| {
                    cat.selected.set_neq(!cat.selected.get());
                }))
            }}))
        })
    }

    pub fn render<F>(self: &Rc<Self>, f: F) -> Dom where F: Fn(Vec<i64>) + Clone + 'static {
        self.fetch_categories(f.clone());
        let select = self.clone();
        self.modal.render(&mut [
            select.render_header(f),
            select.render_main(),
        ])
    }
}