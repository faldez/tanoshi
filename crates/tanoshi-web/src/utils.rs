use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{
    future::{abortable, AbortHandle},
    Future,
};
use futures_signals::signal::{Mutable, Signal};
use wasm_bindgen_futures::spawn_local;

use wasm_bindgen::prelude::*;
use web_sys::{Document, History, HtmlElement, Storage, Window};

thread_local! {
    static WINDOW: Window = web_sys::window().unwrap_throw();
    static DOCUMENT: Document = WINDOW.with(|w| w.document().unwrap_throw());
    static BODY: HtmlElement = DOCUMENT.with(|d| d.body().unwrap_throw());
    static LOCAL_STORAGE: Storage = WINDOW.with(|w| w.local_storage().unwrap_throw().unwrap_throw());
    static HISTORY: History = WINDOW.with(|w| w.history().unwrap_throw());
}

pub struct AsyncState {
    id: usize,
    handle: AbortHandle,
}

impl AsyncState {
    fn new(handle: AbortHandle) -> Self {
        static ID: AtomicUsize = AtomicUsize::new(0);
        let id = ID.fetch_add(1, Ordering::SeqCst);

        Self { id, handle }
    }
}

pub struct AsyncLoader {
    loading: Mutable<Option<AsyncState>>,
}

impl Default for AsyncLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncLoader {
    pub fn new() -> Self {
        Self {
            loading: Mutable::new(None),
        }
    }

    #[allow(dead_code)]
    pub fn cancel(&self) {
        self.replace(None)
    }

    pub fn replace(&self, value: Option<AsyncState>) {
        let mut loading = self.loading.lock_mut();
        if let Some(state) = loading.as_mut() {
            state.handle.abort();
        }
        *loading = value;
    }

    pub fn load<F>(&self, fut: F)
    where
        F: Future<Output = ()> + 'static,
    {
        let (fut, handle) = abortable(fut);

        let state = AsyncState::new(handle);
        let id = state.id;

        self.replace(Some(state));

        let loading = self.loading.clone();

        spawn_local(async move {
            match fut.await {
                Ok(()) => {
                    let mut loading = loading.lock_mut();

                    if let Some(current_id) = loading.as_ref().map(|x| x.id) {
                        if current_id == id {
                            *loading = None;
                        }
                    }
                }
                Err(e) => {
                    error!("failed to spawn task: {}", e);
                }
            }
        });
    }

    pub fn is_loading(&self) -> impl Signal<Item = bool> {
        self.loading.signal_ref(|x| x.is_some())
    }
}

pub fn proxied_image_url(image_url: &str) -> String {
    format!("/image/{}", image_url)
}

pub fn apply_theme(theme: Option<String>) {
    match theme {
        Some(theme) if theme == "dark" => {
            body().class_list().add_1("dark").unwrap_throw();
        }
        Some(theme) if theme == "light" => {
            body().class_list().remove_1("dark").unwrap_throw();
        }
        None | Some(_) => {
            if window()
                .match_media("(prefers-color-scheme: dark)")
                .unwrap_throw()
                .map(|m| m.matches())
                .unwrap_or(false)
            {
                body().class_list().add_1("dark").unwrap_throw();
            } else {
                body().class_list().remove_1("dark").unwrap_throw();
            }
        }
    }
}

pub fn window() -> Window {
    WINDOW.with(|s| s.clone())
}

pub fn local_storage() -> Storage {
    LOCAL_STORAGE.with(|s| s.clone())
}

pub fn history() -> History {
    HISTORY.with(|h| h.clone())
}

pub fn document() -> Document {
    DOCUMENT.with(|d| d.clone())
}

pub fn body() -> HtmlElement {
    BODY.with(|d| d.clone())
}
