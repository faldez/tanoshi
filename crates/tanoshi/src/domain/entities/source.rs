pub struct Source {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub version: String,
    pub rustc_version: String,
    pub lib_version: String,
    pub icon: String,
    pub has_update: bool,
}

impl From<tanoshi_lib::models::SourceInfo> for Source {
    fn from(s: tanoshi_lib::models::SourceInfo) -> Self {
        Self {
            id: s.id,
            name: s.name.to_string(),
            url: s.url.to_string(),
            version: s.version.to_string(),
            rustc_version: "".to_string(),
            lib_version: "".to_string(),
            icon: s.icon.to_string(),
            has_update: false,
        }
    }
}
