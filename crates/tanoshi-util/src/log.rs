#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        let message = format!($($arg)+);
        let message = format!("[{}] {}", std::module_path!(), message);
        $crate::log::print_debug(message);
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        let message = format!($($arg)+);
        let message = format!("[{}] {}", std::module_path!(), message);
        $crate::log::print_error(message);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        let message = format!($($arg)+);
        let message = format!("[{}] {}", std::module_path!(), message);
        $crate::log::print_info(message);
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => {
        let message = format!($($arg)+);
        let message = format!("[{}] {}", std::module_path!(), message);
        $crate::log::print_trace(message);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        let message = format!($($arg)+);
        let message = format!("[{}] {}", std::module_path!(), message);
        $crate::log::print_warn(message);
    };
}

#[cfg(all(not(feature = "__test"), not(feature = "host")))]
pub fn print_debug(message: String) {
    crate::shim::write_err(message);
    unsafe { host_debug() };
}

#[cfg(all(not(feature = "__test"), not(feature = "host")))]
pub fn print_error(message: String) {
    crate::shim::write_err(message);
    unsafe { host_error() };
}

#[cfg(all(not(feature = "__test"), not(feature = "host")))]
pub fn print_info(message: String) {
    crate::shim::write_err(message);
    unsafe { host_info() };
}

#[cfg(all(not(feature = "__test"), not(feature = "host")))]
pub fn print_trace(message: String) {
    crate::shim::write_err(message);
    unsafe { host_trace() };
}

#[cfg(all(not(feature = "__test"), not(feature = "host")))]
pub fn warn(message: String) {
    crate::shim::write_err(message);
    unsafe { host_warn() };
}

#[cfg(all(not(feature = "__test"), not(feature = "host")))]
#[link(wasm_import_module = "tanoshi")]
extern "C" {
    fn host_debug();
    fn host_error();
    fn host_info();
    fn host_trace();
    fn host_warn();
}

#[cfg(any(feature = "__test", feature = "host"))]
pub fn print_debug(message: String) {
    log::debug!(target: "extension", "{}", message);
}

#[cfg(any(feature = "__test", feature = "host"))]
pub fn print_error(message: String) {
    log::error!(target: "extension", "{}", message);
}

#[cfg(any(feature = "__test", feature = "host"))]
pub fn print_info(message: String) {
    log::info!(target: "extension", "{}", message);
}

#[cfg(any(feature = "__test", feature = "host"))]
pub fn print_trace(message: String) {
    log::trace!(target: "extension", "{}", message);
}

#[cfg(any(feature = "__test", feature = "host"))]
pub fn print_warn(message: String) {
    log::warn!(target: "extension", "{}", message);
}
