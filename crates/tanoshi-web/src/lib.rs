#[macro_use]
extern crate log;

mod app;
mod catalogue;
mod common;
mod histories;
mod library;
mod login;
mod manga;
#[allow(dead_code)]
mod query;
mod reader;
mod settings;
mod updates;
mod utils;

use utils::{local_storage, window};
use wasm_bindgen::{prelude::*, JsCast};

use app::App;
use web_sys::{Event, MediaQueryList, MediaQueryListEvent};

#[wasm_bindgen(start)]
pub async fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    utils::apply_theme(local_storage().get("theme").unwrap_throw());

    let closure = Closure::wrap(Box::new(|e: MediaQueryListEvent| {
        if e.matches() {
            utils::body().class_list().add_1("dark").unwrap_throw();
        } else {
            utils::body().class_list().remove_1("dark").unwrap_throw();
        }
    }) as Box<dyn FnMut(_)>);

    if let Ok(Some(media_query_list)) = window().match_media("prefers-color-scheme: dark)") {
        media_query_list.set_onchange(Some(closure.as_ref().unchecked_ref()))
    }

    dominator::append_dom(&dominator::body(), App::render(App::new()));

    Ok(())
}
