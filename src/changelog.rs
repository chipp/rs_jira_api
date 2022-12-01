use serde::Deserialize;

use crate::User;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Changelog {
    pub histories: Vec<History>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct History {
    pub author: User,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub field: String,
    pub from: Option<String>,
    pub from_string: Option<String>,
    pub to: Option<String>,
    pub to_string: Option<String>,
}
