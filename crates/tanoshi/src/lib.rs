#[macro_use]
extern crate log;
extern crate argon2;

#[cfg(feature = "embed")]
pub mod assets;
pub mod auth;
pub mod config;
pub mod db;
pub mod graphql;
pub mod notifier;
pub mod proxy;
#[cfg(feature = "server")]
pub mod server;
pub mod utils;
pub mod worker;
