use std::collections::HashMap;

use configparser::ini::Ini;
use iced::{Background, Border, Color, Padding};

use super::parse::StringExt;

#[derive(Debug)]
pub struct PopupConfig {
    pub width: i32,
    pub height: i32,
    /// Whether the content of the popup should fill the size of the popup window
    pub fill_content_to_size: bool,
    pub padding: Padding,
    pub text_color: Color,
    pub icon_color: Color,
    pub font_size: f32,
    pub icon_size: f32,
    pub text_margin: Padding,
    pub icon_margin: Padding,
    pub spacing: f32,
    pub background: Background,
    pub border: Border,
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            width: 300,
            height: 300,
            fill_content_to_size: false,
            padding: [10, 20].into(),
            text_color: Color::WHITE,
            icon_color: Color::WHITE,
            font_size: 16.,
            icon_size: 20.,
            text_margin: Padding::default(),
            icon_margin: Padding::default(),
            spacing: 0.,
            background: Background::Color(Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 0.8,
            }),
            border: Border::default().rounded(8),
        }
    }
}

#[derive(Debug, Default)]
pub struct PopupConfigOverride {
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub fill_content_to_size: Option<bool>,
    pub padding: Option<Padding>,
    pub text_color: Option<Color>,
    pub icon_color: Option<Color>,
    pub font_size: Option<f32>,
    pub icon_size: Option<f32>,
    pub text_margin: Option<Padding>,
    pub icon_margin: Option<Padding>,
    pub spacing: Option<f32>,
    pub background: Option<Background>,
    pub border: Option<Border>,
}

impl From<&Ini> for PopupConfig {
    fn from(ini: &Ini) -> Self {
        let default = Self::default();
        let section = "popup_style";
        Self {
            width: ini
                .get(section, "width")
                .and_then(|s| s.parse().ok())
                .unwrap_or(default.width),
            height: ini
                .get(section, "height")
                .and_then(|s| s.parse().ok())
                .unwrap_or(default.height),
            fill_content_to_size: ini
                .get(section, "fill_content_to_size")
                .into_bool()
                .unwrap_or(default.fill_content_to_size),
            padding: ini
                .get(section, "padding")
                .into_insets()
                .map(|i| i.into())
                .unwrap_or(default.padding),
            text_color: ini
                .get(section, "text_color")
                .into_color()
                .unwrap_or(default.text_color),
            icon_color: ini
                .get(section, "icon_color")
                .into_color()
                .unwrap_or(default.icon_color),
            font_size: ini
                .get(section, "font_size")
                .into_float()
                .unwrap_or(default.font_size),
            icon_size: ini
                .get(section, "icon_size")
                .into_float()
                .unwrap_or(default.icon_size),
            text_margin: ini
                .get(section, "text_margin")
                .into_insets()
                .map(|i| i.into())
                .unwrap_or(default.text_margin),
            icon_margin: ini
                .get(section, "icon_margin")
                .into_insets()
                .map(|i| i.into())
                .unwrap_or(default.icon_margin),
            spacing: ini
                .get(section, "spacing")
                .into_float()
                .unwrap_or(default.spacing),
            background: ini
                .get(section, "background")
                .into_background()
                .unwrap_or(default.background),
            border: {
                let color = ini
                    .get(section, "border_color")
                    .into_color()
                    .unwrap_or(default.border.color);
                let width = ini
                    .get(section, "border_width")
                    .into_float()
                    .unwrap_or(default.border.width);
                let radius = ini
                    .get(section, "border_radius")
                    .into_insets()
                    .map(|i| i.into())
                    .unwrap_or(default.border.radius);
                Border {
                    color,
                    width,
                    radius,
                }
            },
        }
    }
}

impl From<&HashMap<String, Option<String>>> for PopupConfigOverride {
    fn from(map: &HashMap<String, Option<String>>) -> Self {
        Self {
            width: map
                .get("width")
                .and_then(|s| s.as_ref().and_then(|v| v.parse().ok())),
            height: map
                .get("height")
                .and_then(|s| s.as_ref().and_then(|v| v.parse().ok())),
            fill_content_to_size: map.get("fill_content_to_size").and_then(|s| s.into_bool()),
            padding: map
                .get("padding")
                .and_then(|s| s.into_insets().map(|i| i.into())),
            text_color: map.get("text_color").and_then(|s| s.into_color()),
            icon_color: map.get("icon_color").and_then(|s| s.into_color()),
            font_size: map.get("font_size").and_then(|s| s.into_float()),
            icon_size: map.get("icon_size").and_then(|s| s.into_float()),
            text_margin: map
                .get("text_margin")
                .and_then(|s| s.into_insets().map(|i| i.into())),
            icon_margin: map
                .get("icon_margin")
                .and_then(|s| s.into_insets().map(|i| i.into())),
            spacing: map.get("spacing").and_then(|s| s.into_float()),
            background: map.get("background").and_then(|s| s.into_background()),
            border: {
                let color = map.get("border_color").and_then(|s| s.into_color());
                let width = map.get("border_width").and_then(|s| s.into_float());
                let radius = map
                    .get("border_radius")
                    .and_then(|s| s.into_insets().map(|i| i.into()));
                if color.is_some() || width.is_some() || radius.is_some() {
                    Some(Border {
                        color: color.unwrap_or_default(),
                        width: width.unwrap_or_default(),
                        radius: radius.unwrap_or_default(),
                    })
                } else {
                    None
                }
            },
        }
    }
}
