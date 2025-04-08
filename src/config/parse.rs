use configparser::ini::Ini;
use iced::{
    platform_specific::shell::commands::layer_surface::KeyboardInteractivity, Background, Color,
};

use crate::{registry::Registry, OptionExt};

use super::{anchor::BarAnchor, insets::Insets, Config, Thrice};

impl From<(&Ini, &Registry)> for Config {
    fn from((ini, registry): (&Ini, &Registry)) -> Self {
        let enabled_modules = ini.into();
        let default = Self::default(registry);
        Self {
            hard_reload: ini
                .get("general", "hard_reloading")
                .into_bool()
                .unwrap_or(default.hard_reload),
            enabled_listeners: registry
                .all_listeners()
                .fold(vec![], |mut acc, (id, l)| {
                    l.config().into_iter().for_each(|option| {
                        if ini
                            .get(&option.section, &option.name)
                            .into_bool()
                            .unwrap_or(option.default)
                        {
                            acc.push(*id);
                        }
                    });
                    acc
                })
                .into_iter()
                .chain(registry.enabled_listeners(&enabled_modules, &None))
                .collect(),
            enabled_modules,
            module_config: ini.into(),
            popup_config: ini.into(),
            anchor: ini
                .get("general", "anchor")
                .into_anchor()
                .unwrap_or(default.anchor),
            monitor: ini.get("general", "monitor"),
            kb_focus: ini
                .get("general", "kb_focus")
                .into_kb_focus()
                .unwrap_or(default.kb_focus),
        }
    }
}

pub trait StringExt {
    fn into_bool(self) -> Option<bool>;
    fn into_color(self) -> Option<Color>;
    fn into_float(self) -> Option<f32>;
    fn into_thrice_float(self) -> Option<Thrice<f32>>;
    fn into_anchor(self) -> Option<BarAnchor>;
    fn into_insets(self) -> Option<Insets>;
    fn into_background(self) -> Option<Background>;
    fn into_kb_focus(self) -> Option<KeyboardInteractivity>;
}

impl StringExt for &Option<String> {
    fn into_bool(self) -> Option<bool> {
        self.as_ref().and_then(|v| match v.to_lowercase().as_str() {
            "0" | "f" | "n" | "no" | "false" | "disabled" | "disable" | "off" => Some(false),
            "1" | "t" | "y" | "yes" | "true" | "enabled" | "enable" | "on" => Some(true),
            _ => None,
        })
    }
    fn into_color(self) -> Option<Color> {
        self.as_ref().and_then(|color| {
            csscolorparser::parse(color)
                .map(|v| v.into_ext())
                .ok()
                .map_none(|| println!("Failed to parse color!"))
        })
    }
    fn into_float(self) -> Option<f32> {
        self.as_ref().and_then(|v| v.parse().ok())
    }
    fn into_thrice_float(self) -> Option<Thrice<f32>> {
        self.as_ref().and_then(|value| {
            if let [left, center, right] = value.split_whitespace().collect::<Vec<&str>>()[..] {
                left.parse()
                    .and_then(|l| center.parse().map(|c| (l, c)))
                    .and_then(|(l, c)| right.parse().map(|r| (l, c, r)))
                    .ok()
                    .map(|all| all.into())
            } else {
                value.parse::<f32>().ok().map(|all| all.into())
            }
            .map_none(|| eprintln!("Failed to parse value as float"))
        })
    }
    fn into_anchor(self) -> Option<BarAnchor> {
        self.as_ref()
            .and_then(|value| Some(BarAnchor::from(value.as_str())))
    }
    fn into_insets(self) -> Option<Insets> {
        self.as_ref().and_then(|value| {
            let values = value
                .split_whitespace()
                .filter_map(|i| i.parse::<f32>().ok())
                .collect::<Vec<f32>>();
            match values[..] {
                [all] => Some(Insets::new(all, all, all, all)),
                [vertical, horizontal] => {
                    Some(Insets::new(vertical, horizontal, vertical, horizontal))
                }
                [top, right, bottom, left] => Some(Insets::new(top, right, bottom, left)),
                _ => {
                    eprintln!("Failed to parse value as insets");
                    None
                }
            }
        })
    }
    fn into_background(self) -> Option<Background> {
        self.into_color().map(Background::Color)
    }
    fn into_kb_focus(self) -> Option<KeyboardInteractivity> {
        self.as_ref().and_then(|v| match v.as_str() {
            "none" => Some(KeyboardInteractivity::None),
            "on_demand" => Some(KeyboardInteractivity::OnDemand),
            "exclusive" => Some(KeyboardInteractivity::Exclusive),
            _ => None,
        })
    }
}

pub trait IntoExt<T> {
    fn into_ext(self) -> T;
}

impl IntoExt<Color> for csscolorparser::Color {
    fn into_ext(self) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}
