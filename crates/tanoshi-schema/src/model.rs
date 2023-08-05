use std::sync::Arc;

use futures_signals::{signal::Mutable, signal_vec::MutableVec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Source {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub icon: String,
    pub has_update: bool,
    pub installed: bool,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
    pub telegram_chat_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerStatus {
    pub activated: bool,
    pub version: String,
    pub loggedin: bool,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self {
            activated: false,
            version: "0.0.0".to_string(),
            loggedin: false,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DownloadQueue {
    pub source_name: String,
    pub manga_title: String,
    pub chapter_id: i64,
    pub chapter_title: String,
    pub downloaded: i64,
    pub total: i64,
    pub priority: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Category {
    pub id: Option<i64>,
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum TriState {
    Ignored = 0,
    Included = 1,
    Excluded = 2,
}

impl Default for TriState {
    fn default() -> Self {
        Self::Ignored
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum InputType {
    String(String),
    Number(f64),
    Boolean(bool),
}

impl From<String> for InputType {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for InputType {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<f64> for InputType {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<bool> for InputType {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl ToString for InputType {
    fn to_string(&self) -> String {
        match self {
            InputType::String(val) => val.to_string(),
            InputType::Number(val) => val.to_string(),
            InputType::Boolean(val) => val.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Input {
    Text {
        name: String,
        state: Mutable<Option<String>>,
    },
    Checkbox {
        name: String,
        state: Mutable<Option<bool>>,
    },
    Select {
        name: String,
        values: Vec<InputType>,
        state: Mutable<Option<i64>>,
    },
    Group {
        name: String,
        state: Arc<MutableVec<Input>>,
    },
    Sort {
        name: String,
        values: Vec<InputType>,
        selection: Mutable<Option<(i64, bool)>>,
    },
    State {
        name: String,
        selected: Mutable<Option<TriState>>,
    },
}
