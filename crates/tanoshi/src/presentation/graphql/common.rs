use async_graphql::{connection::CursorType, scalar, SimpleObject};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tanoshi_lib::prelude::Input;

pub struct Cursor(pub i64, pub i64);

impl CursorType for Cursor {
    type Error = anyhow::Error;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        let cursor = String::from_utf8(base64::decode(s)?)?;
        let decoded = cursor.split('#').collect::<Vec<&str>>();
        let timestamp = decoded[0].parse()?;
        let id = decoded[1].parse()?;
        Ok(Self(timestamp, id))
    }

    fn encode_cursor(&self) -> String {
        base64::encode(format!("{}#{}", self.0, self.1))
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct ReadProgress {
    pub at: NaiveDateTime,
    pub last_page: i64,
    pub is_complete: bool,
}

#[derive(Deserialize, Serialize)]
pub struct InputList(pub Vec<Input>);

scalar!(InputList);
