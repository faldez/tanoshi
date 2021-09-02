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
use wasm_bindgen::prelude::*;

use app::App;

#[wasm_bindgen(start)]
pub async fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    match local_storage().get("theme").unwrap_throw() {
        Some(theme) => {
            if theme == "dark" {
                utils::body().class_list().add_1("dark").unwrap_throw();
            } else {
                utils::body().class_list().remove_1("dark").unwrap_throw();
            }
        }
        None => {
            if window()
                .match_media("(prefers-color-scheme: dark)")
                .unwrap_throw()
                .map(|m| m.matches())
                .unwrap_or(false)
            {
                utils::body().class_list().add_1("dark").unwrap_throw();
            } else {
                utils::body().class_list().remove_1("dark").unwrap_throw();
            }
        }
    }

    dominator::append_dom(&dominator::body(), App::render(App::new()));

    Ok(())
}
