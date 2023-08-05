use std::{collections::BTreeMap, rc::Rc};

use crate::common::{Input, TriState};
use dominator::{clone, events, html, svg, with_node, Dom};
use futures_signals::signal::{self, Mutable, ReadOnlyMutable, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVecExt};
use futures_signals::{signal::Signal, signal_map::MutableBTreeMap, signal_vec};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlElement, HtmlInputElement, HtmlOptionElement, HtmlSelectElement};

#[derive(Deserialize, Serialize)]
pub struct InputList {
    #[serde(skip, default = "default_first_render")]
    first_render: Mutable<bool>,
    pub input_list: MutableVec<Input>,
    #[serde(skip)]
    show: Mutable<bool>,
    collapse: MutableBTreeMap<usize, bool>,
    use_modal: bool,
}

fn default_first_render() -> Mutable<bool> {
    Mutable::new(true)
}

impl InputList {
    pub fn new(use_modal: bool) -> Self {
        Self {
            first_render: Mutable::new(use_modal),
            input_list: MutableVec::new(),
            show: Mutable::new(false),
            collapse: MutableBTreeMap::new(),
            use_modal,
        }
    }

    pub fn set(&self, input_list: Vec<Input>) {
        let collapse_map: BTreeMap<usize, bool> = input_list
            .iter()
            .enumerate()
            .filter_map(|(index, input)| {
                matches!(input, Input::Sort { .. } | Input::Group { .. }).then(|| (index, false))
            })
            .collect();
        info!("{:?}", collapse_map);
        self.collapse.lock_mut().replace_cloned(collapse_map);
        self.input_list.lock_mut().replace_cloned(input_list);
    }

    pub fn show(&self) {
        self.show.set(true);
        self.first_render.replace_with(|_| false);
    }

    fn collapse_signal(&self, index: usize) -> impl Signal<Item = bool> {
        self.collapse
            .entries_cloned()
            .filter_map(move |(i, state)| (index == i && state).then(|| 1))
            .sum()
            .map(|sum| sum == 1)
    }

    fn render_collapsible_header(
        input_list: Rc<Self>,
        index: ReadOnlyMutable<Option<usize>>,
        name: String,
    ) -> Dom {
        html!("div", {
            .style("display", "flex")
            .style("justify-content", "space-between")
            .event(clone!(input_list, index => move |_: events::Click| {
                if let Some(index) = index.get() {
                    let state = if let Some(state) = input_list.collapse.lock_ref().get(&index).cloned() {
                        state
                    } else {
                        return;
                    };
                    input_list.collapse.lock_mut().insert(index, !state);
                }
            }))
            .children(&mut [
                html!("label", {
                    .text(&name)
                }),
                svg!("svg", {
                    .attr("xmlns", "http://www.w3.org/2000/svg")
                    .attr("fill", "currentColor")
                    .attr("viewBox", "0 0 20 20")
                    .class("icon")
                    .children(&mut [
                        svg!("path", {
                            .attr_signal("d", input_list.collapse_signal(index.get().unwrap_throw()).map(|visible| {
                                if visible {
                                    Some("M14.707 12.707a1 1 0 01-1.414 0L10 9.414l-3.293 3.293a1 1 0 01-1.414-1.414l4-4a1 1 0 011.414 0l4 4a1 1 0 010 1.414z")
                                } else {
                                    Some("M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z")
                                }
                            }))
                        }),
                    ])
                })
            ])
        })
    }

    fn render_input(
        input_list: Rc<Self>,
        index: ReadOnlyMutable<Option<usize>>,
        input: Input,
    ) -> Dom {
        match input {
            Input::Text { name, state } => html!("div", {
                .style("margin-top", "0.25rem")
                .style("margin-bottom", "0.25rem")
                .children(&mut [
                    html!("label", {
                        .text(&name)
                    }),
                    html!("div", {
                        .class("reader-settings-row")
                        .style("display", "flex")
                        .children(&mut [
                            html!("input" => HtmlInputElement, {
                                .attr("type", "text")
                                .attr_signal("value", state.signal_cloned())
                                .with_node!(input => {
                                    .event(clone!(state => move |_: events::Input| {
                                        state.set((!input.value().is_empty()).then(|| input.value()));
                                    }))
                                })
                            })
                        ])
                    })
                ])
            }),
            Input::Checkbox { name, state } => html!("div", {
                .style("margin-top", "0.25rem")
                .style("margin-bottom", "0.25rem")
                .style("display", "flex")
                .style("justify-content", "space-between")
                .children(&mut [
                    html!("label", {
                        .attr("for", &name)
                        .text(&name)
                    }),
                    html!("div", {
                        .class("reader-settings-row")
                        .children(&mut [
                            html!("input", {
                                .attr("id", &name)
                                .attr("type", "checkbox")
                                .attr_signal("checked", state.signal().map(|state| state.and_then(|state| state.then(|| "checked"))))
                                .event(clone!(state => move |e: events::Change| {
                                    state.set(e.checked());
                                }))
                            }),
                        ])
                    })
                ])
            }),
            Input::Select {
                name,
                values,
                state,
            } => html!("div", {
                .style("margin-top", "0.25rem")
                .style("margin-bottom", "0.25rem")
                .style("display", "flex")
                .style("justify-content", "space-between")
                .children(&mut [
                    html!("label", {
                        .text(&name)
                    }),
                    html!("div", {
                        .class("reader-settings-row")
                        .children(&mut [
                            html!("select" => HtmlSelectElement, {
                                .children_signal_vec(signal_vec::always(values).map(clone!(state => move |value| html!("option" => HtmlOptionElement, {
                                    .text(&value.to_string())
                                    .with_node!(option => {
                                        .attr_signal("selected", state.signal_cloned().map(clone!(option => move |state| if state.unwrap_or_default() == option.index() as i64 { Some("true") } else { None })))
                                    })
                                }))))
                                .with_node!(select => {
                                    .event(clone!(select, state => move |_: events::Change| {
                                        let selected_index = select.selected_index() as i64;
                                        state.set(Some(selected_index));
                                    }))
                                })
                            }),
                        ])
                    })
                ])
            }),
            Input::Group { name, state } => html!("div", {
                .style("margin-top", "0.25rem")
                .style("margin-bottom", "0.25rem")
                .children(&mut [
                    Self::render_collapsible_header(input_list.clone(), index.clone(), name),
                    html!("div" => HtmlElement, {
                        .class("reader-settings-row")
                        .style("padding", "0.5rem")
                        .visible_signal(input_list.collapse_signal(index.get().unwrap_throw()))
                        .children_signal_vec(state.signal_vec_cloned().map(clone!(input_list => move |value| Self::render_input(input_list.clone(), index.clone(), value))))
                    })
                ])
            }),
            Input::Sort {
                name,
                values,
                selection,
            } => html!("div", {
                .style("margin-top", "0.25rem")
                .style("margin-bottom", "0.25rem")
                .children(&mut [
                    Self::render_collapsible_header(input_list.clone(), index.clone(), name.clone()),
                    html!("div" => HtmlElement, {
                        .class("reader-settings-row")
                        .style("padding", "0.5rem")
                        .visible_signal(input_list.collapse_signal(index.get().unwrap_throw()))
                        .children_signal_vec(signal_vec::always(values).enumerate().map(clone!(name, selection => move |(index, value)| html!("div", {
                            .style("margin", "0.5rem")
                            .style("display", "flex")
                            .style("align-items", "center")
                            .children(&mut [
                                html!("label", {
                                    .style("margin-right", "0.5rem")
                                    .children(&mut [
                                        html!("input" => HtmlInputElement, {
                                            .class("tri-state")
                                            .attr("type", "radio")
                                            .attr("name", &name)
                                            .attr("id", &value.to_string())
                                            .with_node!(input => {
                                                .attr_signal("checked", selection.signal_cloned().map(clone!(index => move |selection| {
                                                    if let Some((i, _)) = selection {
                                                        if index.get().unwrap_throw() == (i as usize) {
                                                            Some("true")
                                                        } else {
                                                            None
                                                        }
                                                    } else {
                                                        None
                                                    }
                                                })))
                                                .event(clone!(input, index, selection => move |_: events::Change| {
                                                    if input.checked() {
                                                        selection.set(Some((index.get().unwrap_throw() as i64, true)));
                                                    }
                                                }))
                                            })
                                        }),
                                        svg!("svg", {
                                            .attr("xmlns", "http://www.w3.org/2000/svg")
                                            .attr("fill", "currentColor")
                                            .attr("viewBox", "0 0 20 20")
                                            .class("icon")
                                            .children(&mut [
                                                svg!("path", {
                                                    .attr_signal("d", selection.signal_cloned().map(clone!(index => move |selection| {
                                                        if let Some((i, asc)) = selection {
                                                            if index.get().unwrap_throw() == (i as usize) {
                                                                if asc {
                                                                    Some("M5.293 9.707a1 1 0 010-1.414l4-4a1 1 0 011.414 0l4 4a1 1 0 01-1.414 1.414L11 7.414V15a1 1 0 11-2 0V7.414L6.707 9.707a1 1 0 01-1.414 0z")
                                                                } else {
                                                                    Some("M14.707 10.293a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 111.414-1.414L9 12.586V5a1 1 0 012 0v7.586l2.293-2.293a1 1 0 011.414 0z")
                                                                }
                                                            } else {
                                                                None
                                                            }
                                                        } else {
                                                            None
                                                        }
                                                    })))
                                                }),
                                            ])
                                        })
                                    ])
                                }),
                                html!("label", {
                                    .attr("for", &value.to_string())
                                    .text(&value.to_string())
                                    .event(clone!(index, selection => move |_:events::Click| {
                                        match selection.get()  {
                                            Some((i, true)) if index.get().unwrap_throw() as i64 == i => {
                                                selection.set(Some((i, false)));
                                            }
                                            Some((i, false)) if index.get().unwrap_throw() as i64 == i => {
                                                selection.set(Some((i, true)));
                                            }
                                            _ => {}
                                        }
                                    }))
                                })
                            ])
                        }))))
                    })
                ])
            }),
            Input::State { name, selected } => html!("div" => HtmlElement, {
                .style("margin-top", "0.25rem")
                .style("margin-bottom", "0.25rem")
                .style("display", "flex")
                .style("align-items", "center")
                .with_node!(row => {
                    .event(clone!(row, selected => move |_:events::Click| {
                        let input = row
                            .get_elements_by_tag_name("input")
                            .get_with_index(0)
                            .unwrap_throw()
                            .dyn_into::<HtmlInputElement>()
                            .unwrap_throw();
                        match selected.get() {
                            Some(TriState::Ignored) | None => {
                                selected.set(Some(TriState::Included));
                                input.set_indeterminate(false);
                                input.set_checked(true);
                            }
                            Some(TriState::Included) => {
                                selected.set(Some(TriState::Excluded));
                                input.set_indeterminate(false);
                                input.set_checked(false);
                            }
                            Some(TriState::Excluded) => {
                                selected.set(Some(TriState::Ignored));
                                input.set_indeterminate(true);
                                input.set_checked(false);
                            }
                        }
                    }))
                })
                .children(&mut [
                    html!("input" => HtmlInputElement, {
                        .class("tri-state")
                        .style("margin-top", "0.25rem")
                        .style("margin-bottom", "0.25rem")
                        .attr("type", "checkbox")
                        .after_inserted(clone!(selected => move |input| {
                            if selected.get().is_none() {
                                input.set_indeterminate(true);
                            }
                        }))
                        .attr_signal("checked", selected.signal().map(|selected| if matches!(selected.unwrap_or_default(), TriState::Included) { Some("true") } else { None }))
                    }),
                    html!("label", {
                        .style_signal("text-decoration", selected.signal().map(|selected| if matches!(selected.unwrap_or_default(), TriState::Excluded) { Some("line-through") } else { None }))
                        .text(&name)
                    })
                ])
            }),
        }
    }

    pub fn render<F>(input_list: Rc<Self>, f: F) -> Dom
    where
        F: Fn() + Clone + 'static,
    {
        let use_modal = input_list.use_modal;
        html!("div", {
            .style_signal("width", signal::always(use_modal).map(|use_modal| (!use_modal).then(|| "100%")))
            .children(&mut [
                html!("div", {
                    .visible_signal(input_list.show.signal().map(move |show| show && use_modal))
                    .class("reader-settings-background")
                    .event(clone!(input_list =>  move |_: events::Click| {
                        input_list.show.set(false);
                    }))
                }),
                html!("div", {
                    .class("reader-settings")
                    .style_important_signal("padding-top", signal::always(use_modal).map(|x| if x { Some("0")} else { Some("0.5rem") }))
                    .class_signal(["modal", "animate__animated", "animate__faster"], signal::always(use_modal))
                    .class_signal("non-modal", signal::always(use_modal).map(|x| !x))
                    .class_signal("animate__slideInUp", input_list.show.signal())
                    .class_signal("animate__slideOutDown", input_list.show.signal().map(|x| !x))
                    .style_signal("padding-bottom", signal::always(use_modal).map(|use_modal| if use_modal {
                        Some("calc(env(safe-area-inset-bottom) + 0.5rem)")
                    } else {
                        None
                    }))
                    .style_signal("max-height", signal::always(use_modal).map(|use_modal| use_modal.then(|| "80%")))
                    .style_signal("overflow", signal::always(use_modal).map(|use_modal| use_modal.then(|| "auto")))
                    .visible_signal(input_list.first_render.signal().map(|x| !x))
                    .children(&mut[
                        html!("div", {
                            .style("display", "flex")
                            .style("position", "sticky")
                            .style("top", "0")
                            .style("padding-top", "0.5rem")
                            .style("justify-content", "space-between")
                            .style("margin-bottom", "0.5rem")
                            .style_important("background-color", "var(--modal-background-color)")
                            .visible_signal(signal::always(use_modal))
                            .children(&mut [
                                html!("span", {
                                    .style("font-size", "large")
                                    .text("Filters")
                                }),
                                html!("div", {
                                    .children(&mut [
                                        html!("button", {
                                            .text("Apply")
                                            .event(clone!(input_list, f => move |_: events::Click| {
                                                f();
                                                input_list.show.set(false);
                                            }))
                                        })
                                    ])
                                })
                            ])
                        })
                    ])
                    .children_signal_vec(input_list.input_list.signal_vec_cloned().enumerate().map(clone!(input_list => move |(index, input)| Self::render_input(input_list.clone(), index, input))))
                }),
            ])
            .child_signal(signal::always(use_modal).map(clone!(f => move |use_modal| (!use_modal).then(|| html!("button", {
                .class("uninstall-btn")
                .style("margin-top", "0.5rem")
                .style("margin-bottom", "0.5rem")
                .children(&mut [
                    html!("span", {
                        .style_important("color", "var(--primary-color)")
                        .text("Apply")
                        .event(clone!(f => move |_:events::Click| {
                            f();
                        }))
                    })
                ])
            })))))
        })
    }
}
