use std::collections::HashMap;

use configparser::ini::Ini;
use iced::{Background, Border, Color, Padding};

use super::{parse::StringExt, Thrice};

#[derive(Debug, Default)]
pub struct ModuleConfig {
    pub global: GlobalModuleConfig,
    pub local: LocalModuleConfig,
}

#[derive(Debug)]
pub struct GlobalModuleConfig {
    pub spacing: Thrice<f32>,
    pub background_color: Color,
}

impl Default for GlobalModuleConfig {
    fn default() -> Self {
        Self {
            spacing: 20_f32.into(),
            background_color: Color::from_rgba(0., 0., 0., 0.5),
        }
    }
}

#[derive(Debug)]
pub struct LocalModuleConfig {
    pub text_color: Color,
    pub icon_color: Color,
    pub font_size: f32,
    pub icon_size: f32,
    pub spacing: f32,
    pub padding: Padding,
    pub background: Option<Background>,
    pub border: Border,
}

impl Default for LocalModuleConfig {
    fn default() -> Self {
        Self {
            text_color: Color::WHITE,
            icon_color: Color::WHITE,
            font_size: 16.,
            icon_size: 20.,
            spacing: 10.,
            padding: Padding::default(),
            background: None,
            border: Border::default(),
        }
    }
}

#[derive(Default, Debug)]
pub struct ModuleConfigOverride {
    pub text_color: Option<Color>,
    pub icon_color: Option<Color>,
    pub font_size: Option<f32>,
    pub icon_size: Option<f32>,
    pub spacing: Option<f32>,
}

impl From<&HashMap<String, Option<String>>> for ModuleConfigOverride {
    fn from(map: &HashMap<String, Option<String>>) -> Self {
        Self {
            text_color: map.get("text_color").and_then(|s| s.into_color()),
            icon_color: map.get("icon_color").and_then(|s| s.into_color()),
            font_size: map.get("font_size").and_then(|s| s.into_float()),
            icon_size: map.get("icon_size").and_then(|s| s.into_float()),
            spacing: map.get("spacing").and_then(|s| s.into_float()),
        }
    }
}

impl From<&Ini> for ModuleConfig {
    fn from(ini: &Ini) -> Self {
        let global = Self::default().global;
        let local = Self::default().local;
        let section = "style";
        let module_section = "module_style";
        ModuleConfig {
            global: GlobalModuleConfig {
                background_color: ini
                    .get(section, "background")
                    .into_color()
                    .unwrap_or(global.background_color),
                spacing: ini
                    .get(section, "spacing")
                    .into_thrice_float()
                    .unwrap_or(global.spacing),
            },
            local: LocalModuleConfig {
                text_color: ini
                    .get(module_section, "text_color")
                    .into_color()
                    .unwrap_or(local.text_color),
                icon_color: ini
                    .get(module_section, "icon_color")
                    .into_color()
                    .unwrap_or(local.icon_color),
                font_size: ini
                    .get(module_section, "font_size")
                    .into_float()
                    .unwrap_or(local.font_size),
                icon_size: ini
                    .get(module_section, "icon_size")
                    .into_float()
                    .unwrap_or(local.icon_size),
                spacing: ini
                    .get(module_section, "spacing")
                    .into_float()
                    .unwrap_or(local.spacing),
                padding: ini
                    .get(module_section, "padding")
                    .into_insets()
                    .map(|i| i.into())
                    .unwrap_or(local.padding),
                background: ini.get(module_section, "background").into_background(),
                border: {
                    let color = ini
                        .get(module_section, "border_color")
                        .into_color()
                        .unwrap_or_default();
                    let width = ini
                        .get(module_section, "border_width")
                        .into_float()
                        .unwrap_or(1.);
                    let radius = ini
                        .get(module_section, "border_radius")
                        .into_insets()
                        .map(|i| i.into())
                        .unwrap_or_default();
                    Border {
                        color,
                        width,
                        radius,
                    }
                },
            },
        }
    }
}
