#[derive(serde::Serialize)]
pub struct Source {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub version: String,
    pub icon: String,
}
