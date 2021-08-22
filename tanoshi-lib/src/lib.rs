pub mod data;
pub mod extensions;
pub mod prelude;

/// This is used to ensure both application and extension use the same version
pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[macro_export]
macro_rules! register_extension {
    ($t:ty) => {
        thread_local! {
            static EXT: std::cell::RefCell<$t> = std::cell::RefCell::new(Default::default());
        }

        fn main() {}

        #[no_mangle]
        fn detail() {
            let res = EXT.with(|ext| ext.borrow_mut().detail());
            tanoshi_util::shim::write_object(&res);
        }

        #[no_mangle]
        fn filters() {
            let res = EXT.with(|ext| ext.borrow_mut().filters());
            tanoshi_util::shim::write_object(&res);
        }

        #[no_mangle]
        fn get_manga_list() {
            if let Ok(obj) = tanoshi_util::shim::read_object() {
                let res = EXT.with(|ext| ext.borrow_mut().get_manga_list(obj));
                tanoshi_util::shim::write_object(&res);
            }
        }

        #[no_mangle]
        fn get_manga_info() {
            if let Ok(obj) = tanoshi_util::shim::read_object() {
                let res = EXT.with(|ext| ext.borrow_mut().get_manga_info(obj));
                tanoshi_util::shim::write_object(&res);
            }
        }

        #[no_mangle]
        fn get_chapters() {
            if let Ok(obj) = tanoshi_util::shim::read_object() {
                let res = EXT.with(|ext| ext.borrow_mut().get_chapters(obj));
                tanoshi_util::shim::write_object(&res);
            }
        }

        #[no_mangle]
        fn get_pages() {
            if let Ok(obj) = tanoshi_util::shim::read_object() {
                let res = EXT.with(|ext| ext.borrow_mut().get_pages(obj));
                tanoshi_util::shim::write_object(&res);
            }
        }

        // fn get_page(&self, url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        // }

        // fn login(&self, _: SourceLogin) -> Result<SourceLoginResult, Box<dyn Error>> {
        // }
    };
}

#[macro_export]
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key.to_string(), $val.to_string()); )*
         map
    }}
}
