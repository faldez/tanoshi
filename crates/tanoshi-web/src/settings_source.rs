use crate::{
    common::{events, snackbar, InputList, Route, SettingCategory, Source},
    query,
    utils::AsyncLoader,
};
use dominator::{clone, html, routing, Dom};
use futures_signals::signal::{self, Mutable, SignalExt};
use std::rc::Rc;

pub struct SettingsSource {
    source_id: i64,
    source: Mutable<Option<Source>>,
    input_list: Rc<InputList>,
    loader: AsyncLoader,
}

impl SettingsSource {
    pub fn new(source_id: i64) -> Self {
        Self {
            source_id,
            input_list: Rc::new(InputList::new(false)),
            source: Mutable::new(None),
            loader: AsyncLoader::new(),
        }
    }

    fn fetch_source(settings: Rc<Self>) {
        settings.loader.load(clone!(settings => async move {
            match query::fetch_source(settings.source_id).await {
                Ok(s) => {
                    settings.source.set(Some(Source {
                        id: s.id,
                        name: s.name.clone(),
                        version: s.version.clone(),
                        icon: s.icon.clone(),
                        has_update: false,
                        installed: true,
                    }));
                    settings.input_list.set(s.preferences);
                },
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    fn set_source_preferences(settings: Rc<Self>) {
        settings.loader.load(clone!(settings => async move {
            match query::set_preferences(settings.source_id, settings.input_list.input_list.lock_ref().to_vec().clone()).await {
                Ok(()) => {},
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        }));
    }

    fn uninstall_source(settings: Rc<Self>, id: i64) {
        settings.loader.load(async move {
            match query::uninstall_source(id).await {
                Ok(_) => {
                    routing::go_to_url(&Route::Settings(SettingCategory::SourceList).url());
                }
                Err(err) => {
                    snackbar::show(format!("{}", err));
                }
            }
        });
    }

    pub fn render(settings: Rc<Self>) -> Dom {
        Self::fetch_source(settings.clone());

        html!("div", {
            .class("content")
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("align-items", "center")
            .child_signal(settings.source.signal_cloned().map(|s| s.map(|source| html!("div", {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("align-items", "center")
                .children(&mut [
                    html!("img", {
                        .style("width", "3rem")
                        .style("height", "3rem")
                        .attr("src", &source.icon)
                    }),
                    html!("span", {
                        .text(&source.name)
                    }),
                    html!("span", {
                        .text(&source.version)
                    })
                ])
            }))))
            .children(&mut [
                InputList::render(settings.input_list.clone(), clone!(settings => move || {
                    Self::set_source_preferences(settings.clone());
                }))
            ])
            .child_signal(signal::always(settings.source_id).map(clone!(settings => move |source_id| (source_id > 1).then(|| html!("button", {
                .class("uninstall-btn")
                .children(&mut [
                    html!("span", {
                        .text("Uninstall")
                        .event(clone!(settings => move |_: events::Click| {
                            Self::uninstall_source(settings.clone(), source_id);
                        }))
                    })
                ])
            })))))
        })
    }
}
