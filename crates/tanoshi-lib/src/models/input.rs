use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Input {
    Text {
        name: String,
        state: Option<String>,
    },
    Checkbox {
        name: String,
        state: Option<bool>,
    },
    Select {
        name: String,
        values: Vec<InputType>,
        state: Option<i64>,
    },
    Group {
        name: String,
        state: Vec<Input>,
    },
    Sort {
        name: String,
        values: Vec<InputType>,
        selection: Option<(i64, bool)>,
    },
    State {
        name: String,
        selected: Option<TriState>,
    },
}

impl Input {
    pub fn name(&self) -> String {
        match self {
            Input::Text { name, .. } => name.clone(),
            Input::Checkbox { name, .. } => name.clone(),
            Input::Select { name, .. } => name.clone(),
            Input::Group { name, .. } => name.clone(),
            Input::Sort { name, .. } => name.clone(),
            Input::State { name, .. } => name.clone(),
        }
    }
}

impl PartialEq for Input {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text { name: l_name, .. }, Self::Text { name: r_name, .. }) => l_name == r_name,
            (Self::Checkbox { name: l_name, .. }, Self::Checkbox { name: r_name, .. }) => {
                l_name == r_name
            }
            (Self::Select { name: l_name, .. }, Self::Select { name: r_name, .. }) => {
                l_name == r_name
            }
            (Self::Group { name: l_name, .. }, Self::Group { name: r_name, .. }) => {
                l_name == r_name
            }
            (Self::Sort { name: l_name, .. }, Self::Sort { name: r_name, .. }) => l_name == r_name,
            (Self::State { name: l_name, .. }, Self::State { name: r_name, .. }) => {
                l_name == r_name
            }
            _ => false,
        }
    }
}
