use iced::Color;
use serde::Deserialize;
use toml::{map::Map, Table, Value};

#[derive(Debug)]
pub struct BarTheme {
    pub name: String,
    pub description: String,
    pub background: Color,
    default: Palette,
    modules: Palette,
    popups: Palette,
}

#[derive(Debug, Default)]
struct Palette {
    background: Option<Color>,
    text: Option<Color>,
    icon: Option<Color>,
}

pub struct PaletteRef<'a> {
    pub background: &'a Color,
    pub text: &'a Color,
    pub icon: &'a Color,
}

impl Default for BarTheme {
    fn default() -> Self {
        Self {
            name: String::from("Default theme"),
            description: Default::default(),
            background: Default::default(),
            default: Default::default(),
            modules: Default::default(),
            popups: Default::default(),
        }
    }
}

impl BarTheme {
    pub fn module_palette<'a>(&'a self) -> PaletteRef<'a> {
        PaletteRef {
            background: self.modules.background.as_ref().unwrap_or_else(|| {
                self.default
                    .background
                    .as_ref()
                    .unwrap_or(&Color::TRANSPARENT)
            }),
            text: self
                .modules
                .text
                .as_ref()
                .unwrap_or_else(|| self.default.text.as_ref().unwrap_or(&Color::WHITE)),
            icon: self
                .modules
                .icon
                .as_ref()
                .unwrap_or_else(|| self.default.icon.as_ref().unwrap_or(&Color::WHITE)),
        }
    }

    pub fn popup_palette<'a>(&'a self) -> PaletteRef<'a> {
        PaletteRef {
            background: self.popups.background.as_ref().unwrap_or_else(|| {
                self.default
                    .background
                    .as_ref()
                    .unwrap_or(&Color::TRANSPARENT)
            }),
            text: self
                .popups
                .text
                .as_ref()
                .unwrap_or_else(|| self.default.text.as_ref().unwrap_or(&Color::WHITE)),
            icon: self
                .popups
                .icon
                .as_ref()
                .unwrap_or_else(|| self.default.icon.as_ref().unwrap_or(&Color::WHITE)),
        }
    }
}

impl<'de> Deserialize<'de> for BarTheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map: Table = Map::deserialize(deserializer)?;

        let get_palette = |k: &str, map: &mut Map<_, _>| {
            map.remove(k)
                .and_then(|v| {
                    Palette::deserialize(v)
                        .inspect_err(|e: &toml::de::Error| {
                            log::error!("{k} has an invalid format: {e}")
                        })
                        .ok()
                })
                .unwrap_or_default()
        };

        Ok(BarTheme {
            name: map
                .get("name")
                .ok_or_else(|| {
                    log::error!("Theme is missing a name.");
                    serde::de::Error::missing_field("name")
                })?
                .to_string(),
            description: map
                .get("description")
                .map(|s| s.to_string())
                .unwrap_or_default(),
            background: get_color("background", &map).unwrap_or_default(),
            default: get_palette("default", &mut map),
            modules: get_palette("modules", &mut map),
            popups: get_palette("popups", &mut map),
        })
    }
}

impl<'de> Deserialize<'de> for Palette {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: Table = Map::deserialize(deserializer)?;

        Ok(Palette {
            background: get_color("background", &map),
            text: get_color("text", &map),
            icon: get_color("icon", &map),
        })
    }
}

fn get_color(key: &str, map: &Table) -> Option<Color> {
    map.get(key).and_then(|v| {
        csscolorparser::parse(&v.to_string())
            .inspect_err(|e| log::error!("Failed to parse {key} color: {e}"))
            .ok()
            .map(|c| Color::new(c.r, c.g, c.b, c.a))
    })
}

impl From<BarTheme> for Value {
    fn from(value: BarTheme) -> Self {
        let map = Map::from_iter([
            (String::from("name"), value.name.into()),
            (String::from("description"), value.description.into()),
            (
                String::from("background"),
                color_to_value(value.background).into(),
            ),
            (String::from("default"), value.default.into()),
            (String::from("modules"), value.modules.into()),
            (String::from("popups"), value.popups.into()),
        ]);
        Value::Table(map)
    }
}

impl From<Palette> for Value {
    fn from(value: Palette) -> Self {
        let map = Map::from_iter(
            [
                value
                    .background
                    .map(|c| (String::from("background"), color_to_value(c))),
                value
                    .text
                    .map(|c| (String::from("text"), color_to_value(c))),
                value
                    .icon
                    .map(|c| (String::from("icon"), color_to_value(c))),
            ]
            .into_iter()
            .flatten(),
        );
        Value::Table(map)
    }
}

fn color_to_value(c: Color) -> Value {
    let color = csscolorparser::Color::new(c.r, c.g, c.b, c.a);
    color.to_string().into()
}
