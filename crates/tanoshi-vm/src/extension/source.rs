use libloading::Library;
use once_cell::sync::OnceCell;
use tanoshi_lib::prelude::Extension;

pub struct Source {
    pub(crate) extension: OnceCell<Box<dyn Extension>>,
    #[allow(dead_code)]
    pub(crate) lib: Option<Library>,
    pub rustc_version: String,
    pub lib_version: String,
}

impl Source {
    pub fn new(lib: Library, rustc_version: &str, lib_version: &str) -> Source {
        Source {
            lib: Some(lib),
            rustc_version: rustc_version.to_string(),
            lib_version: lib_version.to_string(),
            extension: OnceCell::new(),
        }
    }

    pub fn from(extension: Box<dyn Extension>) -> Self {
        Self {
            lib: None,
            rustc_version: tanoshi_lib::RUSTC_VERSION.to_string(),
            lib_version: tanoshi_lib::LIB_VERSION.to_string(),
            extension: OnceCell::from(extension),
        }
    }
}

impl tanoshi_lib::extensions::PluginRegistrar for Source {
    fn register_function(&mut self, extension: Box<dyn Extension>) {
        self.extension
            .set(extension)
            .map_err(|_| "extension already initiated")
            .unwrap();
    }
}
