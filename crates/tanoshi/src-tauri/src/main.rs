#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use crate::server::Server;

mod server;

fn main() {
  tauri::Builder::default()
    .plugin(Server::new())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
