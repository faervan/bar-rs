use configparser::ini::Ini;
use iced::Color;

use crate::{modules::hyprland::get_monitor_name, registry::Registry, OptionExt};

use super::{Config, Thrice};

impl From<(&Ini, &Registry)> for Config {
    fn from((ini, registry): (&Ini, &Registry)) -> Self {
        let enabled_modules = ini.into();
        Self {
            close_on_fullscreen: ini.get("general", "close_on_fullscreen").into_bool(true),
            enabled_listeners: registry
                .all_listeners()
                .fold(vec![], |mut acc, (id, l)| {
                    l.config().into_iter().for_each(|option| {
                        if ini
                            .get(&option.section, &option.name)
                            .into_bool(option.default)
                        {
                            acc.push(*id);
                        }
                    });
                    acc
                })
                .into_iter()
                .chain(registry.enabled_listeners(&enabled_modules))
                .collect(),
            enabled_modules,
            module_config: ini.into(),
            monitor: ini.get("general", "monitor").unwrap_or(get_monitor_name()),
        }
    }
}

pub trait StringExt {
    fn into_bool(self, default: bool) -> bool;
    fn into_color(self) -> Option<Color>;
    fn into_float(self) -> Option<f32>;
    fn into_thrice_float(self) -> Option<Thrice<f32>>;
}

impl StringExt for &Option<String> {
    fn into_bool(self, default: bool) -> bool {
        self.as_ref()
            .and_then(|v| match v.to_lowercase().as_str() {
                "0" | "f" | "n" | "no" | "false" | "disabled" | "disable" | "off" => Some(false),
                "1" | "t" | "y" | "yes" | "true" | "enabled" | "enable" | "on" => Some(true),
                _ => None,
            })
            .unwrap_or(default)
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
            if let [left, center, right] =
                value.split(',').map(|i| i.trim()).collect::<Vec<&str>>()[..]
            {
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
