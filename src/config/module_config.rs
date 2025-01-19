use std::collections::HashMap;

use configparser::ini::Ini;
use iced::{
    runtime::platform_specific::wayland::layer_surface::IcedMargin, Background, Border, Color,
    Padding,
};

use super::{parse::StringExt, Thrice};

#[derive(Debug, Default)]
pub struct ModuleConfig {
    pub global: GlobalModuleConfig,
    pub local: LocalModuleConfig,
}

#[derive(Debug)]
pub struct GlobalModuleConfig {
    pub spacing: Thrice<f32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub margin: IcedMargin,
    pub padding: Padding,
    pub background_color: Color,
}

impl Default for GlobalModuleConfig {
    fn default() -> Self {
        Self {
            spacing: 20_f32.into(),
            width: None,
            height: None,
            margin: IcedMargin::default(),
            padding: Padding::default(),
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
    pub text_margin: Padding,
    pub icon_margin: Padding,
    pub spacing: f32,
    pub margin: Padding,
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
            text_margin: Padding::default(),
            icon_margin: Padding::default(),
            spacing: 10.,
            margin: Padding::default(),
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
    pub text_margin: Option<Padding>,
    pub icon_margin: Option<Padding>,
    pub spacing: Option<f32>,
    pub margin: Option<Padding>,
    pub padding: Option<Padding>,
    pub background: Option<Option<Background>>,
    pub border: Option<Border>,
}

impl From<&HashMap<String, Option<String>>> for ModuleConfigOverride {
    fn from(map: &HashMap<String, Option<String>>) -> Self {
        Self {
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
            margin: map
                .get("margin")
                .and_then(|s| s.into_insets().map(|i| i.into())),
            padding: map
                .get("padding")
                .and_then(|s| s.into_insets().map(|i| i.into())),
            background: map.get("background").map(|s| s.into_background()),
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
                height: ini.get(section, "height").and_then(|v| v.parse().ok()),
                width: ini.get(section, "width").and_then(|v| v.parse().ok()),
                margin: ini
                    .get(section, "margin")
                    .into_insets()
                    .map(|i| i.into())
                    .unwrap_or(global.margin),
                padding: ini
                    .get(section, "padding")
                    .into_insets()
                    .map(|i| i.into())
                    .unwrap_or(global.padding),
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
                text_margin: ini
                    .get(module_section, "text_margin")
                    .into_insets()
                    .map(|i| i.into())
                    .unwrap_or(local.text_margin),
                icon_margin: ini
                    .get(module_section, "icon_margin")
                    .into_insets()
                    .map(|i| i.into())
                    .unwrap_or(local.icon_margin),
                spacing: ini
                    .get(module_section, "spacing")
                    .into_float()
                    .unwrap_or(local.spacing),
                margin: ini
                    .get(module_section, "margin")
                    .into_insets()
                    .map(|i| i.into())
                    .unwrap_or(local.margin),
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
