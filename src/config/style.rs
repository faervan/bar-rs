use config::{Map, ValueKind};
use serde::Deserialize;

use super::{insets::Insets, types::BarAnchor};

#[derive(Debug, Deserialize)]
pub struct BarStyle {
    pub name: String,
    pub description: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub margin: Insets<i32>,
    pub anchor: BarAnchor,
}

impl Default for BarStyle {
    fn default() -> Self {
        Self {
            name: String::from("Default style"),
            description: Default::default(),
            width: Default::default(),
            height: Default::default(),
            margin: Default::default(),
            anchor: Default::default(),
        }
    }
}

impl From<BarStyle> for ValueKind {
    fn from(value: BarStyle) -> Self {
        let map = Map::from([
            (String::from("name"), value.name.into()),
            (String::from("description"), value.description.into()),
            (String::from("width"), value.width.into()),
            (String::from("height"), value.height.into()),
            (String::from("margin"), value.margin.into()),
            (String::from("anchor"), value.anchor.into()),
        ]);
        ValueKind::Table(map)
    }
}
