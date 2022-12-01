use serde::Deserialize;

#[derive(Deserialize)]
pub struct Board {
    pub id: u32,
    pub name: String,
}
