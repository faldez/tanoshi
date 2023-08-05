use std::collections::HashMap;

use crate::models::{ChapterInfo, Input, MangaInfo, SourceInfo};
use anyhow::Result;

pub trait Extension: Send + Sync {
    fn get_source_info(&self) -> SourceInfo;

    fn headers(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn filter_list(&self) -> Vec<Input> {
        vec![]
    }

    fn get_preferences(&self) -> Result<Vec<Input>> {
        Ok(vec![])
    }

    fn set_preferences(&mut self, _preferences: Vec<Input>) -> Result<()> {
        Ok(())
    }

    fn get_popular_manga(&self, page: i64) -> Result<Vec<MangaInfo>>;

    fn get_latest_manga(&self, page: i64) -> Result<Vec<MangaInfo>>;

    fn search_manga(
        &self,
        page: i64,
        query: Option<String>,
        filters: Option<Vec<Input>>,
    ) -> Result<Vec<MangaInfo>>;

    fn get_manga_detail(&self, path: String) -> Result<MangaInfo>;

    fn get_chapters(&self, path: String) -> Result<Vec<ChapterInfo>>;

    fn get_pages(&self, path: String) -> Result<Vec<String>>;
}

/// A type represents an extension
pub struct PluginDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,
    pub register: unsafe fn(&mut dyn PluginRegistrar),
}

/// A trait for register an extension
pub trait PluginRegistrar {
    fn register_function(&mut self, extension: Box<dyn Extension>);
}

/// macro for export an extension
#[macro_export]
macro_rules! export_plugin {
    ($register:expr) => {
        #[doc(hidden)]
        #[no_mangle]
        pub static plugin_declaration: $crate::extensions::PluginDeclaration =
            $crate::extensions::PluginDeclaration {
                rustc_version: $crate::RUSTC_VERSION,
                core_version: $crate::LIB_VERSION,
                register: $register,
            };
    };
}
