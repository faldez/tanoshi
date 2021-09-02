use std::fmt::Debug;

use crate::data::{Chapter, Manga, Param, Source, ExtensionResult, Filters};

/// `Extension` trait is an implementation for building extensions
pub trait Extension: Send + Sync {
    /// Returns the information of the source
    fn detail(&self) -> Source;

    /// Returns filter supported by the source
    fn filters(&self) -> ExtensionResult<Option<Filters>>;

    /// Returns list of manga from the source
    ///
    /// # Arguments
    ///
    /// * `param` - Parameter to filter manga from source
    /// * `keyword` - Keyword of manga title to search
    /// * `genres` - List of genres of manga to search
    /// * `page` - Number of page
    /// * `sort_by` - Sort results by SortByParam
    /// * `sort_order` - Sort ascending or descending
    /// * `auth` - If source need login to search, this param used to provide credentials
    fn get_manga_list(&self, param: Param) -> ExtensionResult<Vec<Manga>>;

    /// Returns detail of manga
    fn get_manga_info(&self, path: String) -> ExtensionResult<Manga>;

    /// Returns list of chapters of a manga
    fn get_chapters(&self, path: String) -> ExtensionResult<Vec<Chapter>>;

    /// Returns list of pages from a chapter of a manga
    fn get_pages(&self, path: String) -> ExtensionResult<Vec<String>>;

    // /// Proxy image
    // fn get_page(&self, url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    //     let bytes = {
    //         let resp = ureq::get(url).call()?;
    //         let mut reader = resp.into_reader();
    //         let mut bytes = vec![];
    //         if reader.read_to_end(&mut bytes).is_err() {
    //             return Err(anyhow!("error read image"));
    //         }
    //         bytes
    //     };
    //     Ok(bytes)
    // }

    // /// Login to source
    // fn login(&self, _: SourceLogin) -> Result<SourceLoginResult, Box<dyn Error>> {
    //     Err(anyhow!("not implemented"))
    // }
}

impl Debug for dyn Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Extension: {:?}", self.detail())
    }
}