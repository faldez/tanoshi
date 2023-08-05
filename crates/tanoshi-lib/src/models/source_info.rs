use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Lang {
    All,
    Single(String),
    Multi(Vec<String>),
}

/// A type represent source
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceInfo {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub version: &'static str,
    pub icon: &'static str,
    pub languages: Lang,
    pub nsfw: bool,
}
