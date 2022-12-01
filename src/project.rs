use serde::Deserialize;

#[derive(Deserialize)]
pub struct Project {
    pub key: String,
    pub name: String,
}
