use serde::Deserialize;
use toml::{map::Map, Value};

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
//
// impl From<BarStyle> for Value {
//     fn from(value: BarStyle) -> Self {
//         let mut map = Map::from_iter([
//             (String::from("name"), value.name.into()),
//             (String::from("description"), value.description.into()),
//         ]);
//         if let Some(width) = value.width {
//             map.insert(String::from("width"), width.into());
//         }
//         if let Some(height) = value.height {
//             map.insert(String::from("height"), height.into());
//         }
//         map.extend([
//             (String::from("margin"), value.margin.into()),
//             (String::from("anchor"), value.anchor.into()),
//         ]);
//         Value::Table(map)
//     }
// }
