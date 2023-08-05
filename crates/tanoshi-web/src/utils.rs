use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::anyhow;
use futures::{
    future::{abortable, AbortHandle},
    Future,
};
use futures_signals::signal::{self, Mutable, Signal};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, History, HtmlElement, Storage, Window};

thread_local! {
    static WINDOW: Window = web_sys::window().unwrap_throw();
    static DOCUMENT: Document = WINDOW.with(|w| w.document().unwrap_throw());
    static BODY: HtmlElement = DOCUMENT.with(|d| d.body().unwrap_throw());
    static LOCAL_STORAGE: Storage = WINDOW.with(|w| w.local_storage().unwrap_throw().unwrap_throw());
    static SESSION_STORAGE: Storage = WINDOW.with(|w| w.session_storage().unwrap_throw().unwrap_throw());
    static HISTORY: History = WINDOW.with(|w| w.history().unwrap_throw());
    static IMAGE_PROXY_HOST: std::cell::RefCell<String> = std::cell::RefCell::new("/image".to_string());
    static GRAPHQL_HOST: std::cell::RefCell<String> = std::cell::RefCell::new("/graphql".to_string());
    static GRAPHQL_WS_HOST: std::cell::RefCell<String> = std::cell::RefCell::new("/ws".to_string());
    static IS_TAURI: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
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
    format!("{}/{}", image_proxy_host(), image_url)
}

pub fn initialize_urls() {
    match js_sys::eval("window.__TAURI__") {
        Ok(val) if !val.is_undefined() => {
            IS_TAURI.with(|s| *s.borrow_mut() = true);
        }
        _ => {}
    };

    let image_proxy_host = match js_sys::eval("window.__TANOSHI_PORT__") {
        Ok(val) if !val.is_undefined() => {
            format!(
                "http://localhost:{}/image",
                val.as_f64().unwrap_or_default() as i64
            )
        }
        _ => "/image".to_string(),
    };

    IMAGE_PROXY_HOST.with(|s| *s.borrow_mut() = image_proxy_host);

    let graphql_host = match js_sys::eval("window.__TANOSHI_PORT__") {
        Ok(val) if !val.is_undefined() => {
            format!(
                "http://localhost:{}",
                val.as_f64().unwrap_or_default() as i64
            )
        }
        _ => window()
            .document()
            .unwrap_throw()
            .location()
            .unwrap_throw()
            .origin()
            .unwrap_throw(),
    };

    GRAPHQL_HOST.with(|s| *s.borrow_mut() = format!("{}/graphql", graphql_host));
    GRAPHQL_WS_HOST
        .with(|s| *s.borrow_mut() = format!("{}/ws", graphql_host.replace("http", "ws")));
}

pub fn is_tauri() -> bool {
    IS_TAURI.with(|v| *v.borrow())
}

pub fn is_tauri_signal() -> impl Signal<Item = bool> {
    signal::always(is_tauri())
}

pub fn image_proxy_host() -> String {
    IMAGE_PROXY_HOST.with(|v| v.borrow().clone())
}

pub fn graphql_host() -> String {
    GRAPHQL_HOST.with(|v| v.borrow().clone())
}

pub fn graphql_ws_host() -> String {
    GRAPHQL_WS_HOST.with(|v| v.borrow().clone())
}

pub fn apply_theme(theme: Option<String>) {
    let mut status_bar_color = "#5b749b";
    match theme {
        Some(theme) if theme == "dark" => {
            body().class_list().add_1("dark").unwrap_throw();
            status_bar_color = "#090909";
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
                status_bar_color = "#090909";
            } else {
                body().class_list().remove_1("dark").unwrap_throw();
            }
        }
    }

    apply_theme_color(status_bar_color).unwrap_throw();
}

pub fn apply_theme_color(status_bar_color: &str) -> Result<(), anyhow::Error> {
    if window()
        .match_media("(display-mode: standalone)")
        .map_err(|e| anyhow!("error mactch media: {:?}", e))?
        .ok_or_else(|| anyhow!("no display-mode query"))?
        .matches()
    {
        document()
            .query_selector("meta[name=\"theme-color\"]")
            .map_err(|e| anyhow!("error query meta: {:?}", e))?
            .ok_or_else(|| anyhow!("no theme-color meta"))?
            .set_attribute("content", status_bar_color)
            .map_err(|e| anyhow!("error set content: {:?}", e))?;
    }

    Ok(())
}

pub fn window() -> Window {
    WINDOW.with(|s| s.clone())
}

pub fn local_storage() -> Storage {
    LOCAL_STORAGE.with(|s| s.clone())
}

pub fn session_storage() -> Storage {
    SESSION_STORAGE.with(|s| s.clone())
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
