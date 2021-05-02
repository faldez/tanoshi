use serde::{Deserialize, Serialize};
use yew::format::Text;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub username: String,
    pub password: Option<String>,
    pub role: String,
    pub telegram_chat_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: i64,
    pub telegram_chat_id: Option<i64>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub enum ReadingDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub enum PageRendering {
    SinglePage,
    DoublePage,
    LongStrip,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub enum BackgroundColor {
    Black,
    White,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub struct SettingParams {
    pub page_rendering: PageRendering,
    pub background_color: BackgroundColor,
    pub reading_direction: ReadingDirection,
    pub dark_mode: bool,
}

impl Default for SettingParams {
    fn default() -> Self {
        SettingParams {
            page_rendering: PageRendering::SinglePage,
            background_color: BackgroundColor::Black,
            reading_direction: ReadingDirection::LeftToRight,
            dark_mode: false,
        }
    }
}

impl SettingParams {
    pub fn parse_from_local_storage() -> SettingParams {
        if let Ok(storage) = crate::app::api::get_local_storage() {
            if let Ok(settings) = storage.restore("settings") {
                if let Ok(settings) = serde_json::from_str(settings.as_str()) {
                    return settings;
                }
            }
        }
        return SettingParams::default();
    }

    pub fn save(&self) {
        if let Ok(mut storage) = crate::app::api::get_local_storage() {
            storage.store("settings", self);
        }
    }
}

impl From<&SettingParams> for Text {
    fn from(param: &SettingParams) -> Self {
        let val = serde_json::to_string(&param).unwrap();
        Text::Ok(val)
    }
}