#[macro_use]
extern crate log;

mod app;
mod catalogue;
mod common;
mod histories;
mod library;
mod manga;
mod query;
mod reader;
mod settings;
mod updates;
mod utils;


use wasm_bindgen::prelude::*;

use app::App;
use utils::{BODY};

#[wasm_bindgen(start)]
pub async fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    BODY.with(|b| {
        b.class_list()
            .add_2("bg-gray-100", "dark:bg-black")
            .unwrap_throw()
    });

    dominator::append_dom(&dominator::body(), App::render(App::new()));

    Ok(())
}
