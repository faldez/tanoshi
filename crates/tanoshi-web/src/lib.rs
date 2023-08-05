#[macro_use]
extern crate log;

mod app;
mod catalogue;
mod catalogue_list;
mod common;
mod histories;
mod library;
mod library_list;
mod login;
mod manga;
#[allow(dead_code)]
mod query;
mod reader;
mod settings;
mod settings_categories;
mod settings_download_queue;
mod settings_manage_downloads;
mod settings_source;
mod tracker_login;
mod tracker_redirect;
mod updates;
mod utils;

use utils::{apply_theme_color, initialize_urls, local_storage, window};
use wasm_bindgen::{prelude::*, JsCast};

use app::App;
use web_sys::MediaQueryListEvent;

#[wasm_bindgen(start)]
pub async fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    initialize_urls();

    if utils::is_tauri() {
        utils::body().class_list().add_1("tauri").unwrap_throw();
    }

    utils::apply_theme(local_storage().get("theme").unwrap_throw());

    let closure = Closure::wrap(Box::new(|e: MediaQueryListEvent| {
        let mut status_bar_color = "#5b749b";
        if e.matches() {
            utils::body().class_list().add_1("dark").unwrap_throw();
            status_bar_color = "#090909";
        } else {
            utils::body().class_list().remove_1("dark").unwrap_throw();
        }

        apply_theme_color(status_bar_color).unwrap_throw();
    }) as Box<dyn FnMut(_)>);

    if let Ok(Some(media_query_list)) = window().match_media("(prefers-color-scheme: dark)") {
        media_query_list.set_onchange(Some(closure.as_ref().unchecked_ref()))
    }

    closure.forget();

    dominator::append_dom(&dominator::body(), App::render(App::new()));

    Ok(())
}
