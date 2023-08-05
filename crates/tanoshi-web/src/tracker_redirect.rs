use std::sync::Arc;

use dominator::{html, Dom};
use futures_signals::signal::{Mutable, SignalExt};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    query,
    utils::{session_storage, AsyncLoader},
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum AuthorizationState {
    Idle,
    Authorizing,
    Success,
    Failed(String),
}

pub struct TrackerRedirect {
    tracker: String,
    code: String,
    state: Option<String>,
    authorization_state: Mutable<AuthorizationState>,
    loader: AsyncLoader,
}

impl TrackerRedirect {
    pub fn new(tracker: String, code: String, state: Option<String>) -> Arc<Self> {
        Arc::new(Self {
            tracker,
            code,
            state,
            authorization_state: Mutable::new(AuthorizationState::Idle),
            loader: AsyncLoader::new(),
        })
    }

    fn fetch_myanimelist_login_end(self: Arc<Self>) {
        let code = self.code.clone();
        let state = if let Some(state) = self.state.clone() {
            state
        } else {
            self.authorization_state
                .set_neq(AuthorizationState::Failed("no state code".to_string()));
            return;
        };

        let tracker_redirect = self.clone();
        self.loader.load(async move {
            let session_storage = session_storage();
            let csrf_state = session_storage
                .get("csrf-state")
                .unwrap_throw()
                .unwrap_throw();
            let pkce_code_verifier = session_storage
                .get("pkce-code-verifier")
                .unwrap_throw()
                .unwrap_throw();

            tracker_redirect
                .authorization_state
                .set_neq(AuthorizationState::Authorizing);

            match query::myanimelist_login_end(code, state, csrf_state, pkce_code_verifier).await {
                Ok(()) => tracker_redirect
                    .authorization_state
                    .set_neq(AuthorizationState::Success),
                Err(e) => tracker_redirect
                    .authorization_state
                    .set_neq(AuthorizationState::Failed(format!("{e}"))),
            }
        });
    }

    fn fetch_anilist_login_end(self: Arc<Self>) {
        let code = self.code.clone();
        let tracker_redirect = self.clone();
        self.loader.load(async move {
            tracker_redirect
                .authorization_state
                .set_neq(AuthorizationState::Authorizing);

            match query::anilist_login_end(code).await {
                Ok(()) => tracker_redirect
                    .authorization_state
                    .set_neq(AuthorizationState::Success),
                Err(e) => tracker_redirect
                    .authorization_state
                    .set_neq(AuthorizationState::Failed(format!("{e}"))),
            }
        });
    }

    pub fn render(self: Arc<Self>) -> Dom {
        match self.tracker.as_str() {
            "myanimelist" => self.clone().fetch_myanimelist_login_end(),
            "anilist" => self.clone().fetch_anilist_login_end(),
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
                    .text_signal(self.authorization_state.signal_cloned().map({
                        let tracker = self.tracker.clone();
                        move |state| match state {
                            AuthorizationState::Idle | AuthorizationState::Authorizing => format!("Authorizing {tracker}..."),
                            AuthorizationState::Success => format!("Authorization {} success. You can close this window", tracker),
                            AuthorizationState::Failed(e) => format!("Authorization {} failed: {e}", tracker),
                        }
                    }))
                })
            ])
        })
    }
}
