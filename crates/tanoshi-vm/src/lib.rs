#[macro_use]
extern crate log;

pub mod extension;
pub mod prelude;

#[cfg(target_os = "windows")]
pub const PLUGIN_EXTENSION: &str = "dll";
#[cfg(target_os = "macos")]
pub const PLUGIN_EXTENSION: &str = "dylib";
#[cfg(target_os = "linux")]
pub const PLUGIN_EXTENSION: &str = "so";
