use std::sync::LazyLock;

use gloo::net::http::Request;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Course {
    pub code: String,
    pub name: String,
    pub activities: Vec<Activity>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Activity {
    pub name: String,
    pub classes: Vec<Class>,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Class {
    pub occurrence: u8,
    pub day: u8,
    pub start: u16,
    pub end: u16,
}

static INDEX: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    // INDEX_FILE is set by the build script
    const INDEX_BIN: &[u8] = include_bytes!(env!("INDEX_FILE"));
    postcard::from_bytes(INDEX_BIN).unwrap()
});

pub fn sessions() -> impl Iterator<Item = &'static str> {
    INDEX.iter().map(|(name, _)| name.as_ref())
}

pub fn default_session() -> &'static str {
    sessions().next().unwrap()
}

pub async fn get_courses(session: &str) -> Vec<Course> {
    // TODO: improve error handling?
    let (_, path) = INDEX.iter().find(|(name, _)| name == session).unwrap();
    let resp = Request::get(path).send().await.unwrap();
    let data = resp.binary().await.unwrap();
    postcard::from_bytes(&data).unwrap()
}
