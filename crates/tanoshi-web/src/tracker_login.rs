use std::sync::Arc;

use dominator::{html, Dom};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    common::snackbar,
    query,
    utils::{session_storage, window, AsyncLoader},
};

pub struct TrackerLogin {
    tracker: String,
    loader: AsyncLoader,
}

impl TrackerLogin {
    pub fn new(tracker: String) -> Arc<Self> {
        Arc::new(Self {
            tracker,
            loader: AsyncLoader::new(),
        })
    }

    fn fetch_myanimelist_login_start(self: Arc<Self>) {
        self.loader.load(async move {
            match query::myanimelist_login_start().await {
                Ok(session) => {
                    let session_storage = session_storage();
                    session_storage
                        .set("csrf-state", &session.csrf_state)
                        .unwrap_throw();
                    if let Some(pkce_code_verifier) = session.pkce_code_verifier {
                        session_storage
                            .set("pkce-code-verifier", &pkce_code_verifier)
                            .unwrap_throw();
                    }
                    window()
                        .location()
                        .replace(&session.authorize_url)
                        .unwrap_throw();
                }
                Err(e) => {
                    snackbar::show(format!("error redirecting: {e}"));
                }
            }
        });
    }

    fn fetch_anilist_login_start(self: Arc<Self>) {
        self.loader.load(async move {
            match query::anilist_login_start().await {
                Ok(session) => {
                    window()
                        .location()
                        .replace(&session.authorize_url)
                        .unwrap_throw();
                }
                Err(e) => {
                    snackbar::show(format!("error redirecting: {e}"));
                }
            }
        });
    }

    pub fn render(self: Arc<Self>) -> Dom {
        match self.tracker.as_str() {
            "myanimelist" => self.clone().fetch_myanimelist_login_start(),
            "anilist" => self.clone().fetch_anilist_login_start(),
            _ => {}
        }

        html!("div", {
            .class("content")
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("max-width", "1024px")
            .style("margin", "auto")
            .style("padding", "0.5rem")
            .children(&mut [
                html!("img", {
                    .style("width", "8rem")
                    .style("height", "8rem")
                    .style("border-radius", "0.5rem")
                    .style("margin", "auto")
                    .attr("src", "/icons/512.png")
                }),
                html!("div", {
                    .style("padding", "0.5rem")
                    .style("margin", "0.5rem")
                    .style("text-align", "center")
                    .text(&format!("Redirecting to {}", self.tracker))
                })
            ])
        })
    }
}
