#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[macro_use]
extern crate log;

mod server;

use crate::server::Server;

fn main() {
  env_logger::init();

  tauri::Builder::default()
    .plugin(Server::new())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
