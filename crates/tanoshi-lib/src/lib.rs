pub mod error;
pub mod extensions;
pub mod models;
pub mod prelude;

/// This is used to ensure both application and extension use the same version
pub static LIB_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Rust doesn't have stable ABI, this is used to ensure `rustc` version is match
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");
