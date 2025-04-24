use std::{path::PathBuf, str::FromStr};

use config::ValueKind;
use iced::runtime::platform_specific::wayland::layer_surface::IcedMargin;
use serde::{Deserialize, Serialize};
use serde_variant::to_variant_name;
use smithay_client_toolkit::shell::wlr_layer::{Anchor, KeyboardInteractivity};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub path: String,
    pub monitor: Option<String>,
    pub anchor: BarAnchor,
    pub kb_focus: KbFocus,
    pub reload_interval: f64,
    width: Option<u32>,
    height: Option<u32>,
    pub margin: Padding<i32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: format!(
                "{}/.config/bar-rs/bar-rs.toml",
                std::env::var("HOME").unwrap()
            ),
            monitor: None,
            anchor: BarAnchor::Top,
            kb_focus: KbFocus::None,
            reload_interval: 3.,
            width: None,
            height: None,
            margin: Padding::all(0),
        }
    }
}

impl Config {
    pub fn load(path: Option<&String>) -> anyhow::Result<Self> {
        let default = Config::default();
        let path = PathBuf::from(path.unwrap_or(&default.path));
        let config = config::Config::builder()
            .set_default("path", default.path)?
            .set_default("anchor", &default.anchor)?
            .set_default("kb_focus", &default.kb_focus)?
            .set_default("reload_interval", default.reload_interval)?
            .add_source(config::File::from(path))
            .build()?;
        Ok(config.try_deserialize()?)
    }

    pub fn exclusive_zone(&self) -> i32 {
        (match self.anchor {
            BarAnchor::Left | BarAnchor::Right => self.width.unwrap_or(30),
            BarAnchor::Top | BarAnchor::Bottom => self.height.unwrap_or(30),
        }) as i32
    }

    /// Determine the size to pass to `get_layer_surface`
    pub fn dimension(&self, x: u32, y: u32) -> Option<(Option<u32>, Option<u32>)> {
        let [width, height] = match self.anchor.is_vertical() {
            true => [self.width.unwrap_or(50), self.height.unwrap_or(y)],
            false => [self.width.unwrap_or(x), self.height.unwrap_or(30)],
        };
        Some((Some(width), Some(height)))
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BarAnchor {
    Top,
    Bottom,
    Left,
    Right,
}

impl From<&BarAnchor> for ValueKind {
    fn from(value: &BarAnchor) -> Self {
        let s = to_variant_name(value).unwrap();
        ValueKind::String(s.to_owned())
    }
}

impl From<&BarAnchor> for Anchor {
    fn from(value: &BarAnchor) -> Self {
        match value {
            BarAnchor::Top => Anchor::TOP,
            BarAnchor::Bottom => Anchor::BOTTOM,
            BarAnchor::Left => Anchor::LEFT,
            BarAnchor::Right => Anchor::RIGHT,
        }
    }
}

impl BarAnchor {
    pub fn is_vertical(&self) -> bool {
        match self {
            BarAnchor::Top | BarAnchor::Bottom => false,
            _ => true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KbFocus {
    None,
    Exclusive,
    OnDemand,
}

impl From<&KbFocus> for ValueKind {
    fn from(value: &KbFocus) -> Self {
        let s = to_variant_name(value).unwrap();
        ValueKind::String(s.to_owned())
    }
}

impl From<&KbFocus> for KeyboardInteractivity {
    fn from(value: &KbFocus) -> Self {
        match value {
            KbFocus::None => KeyboardInteractivity::None,
            KbFocus::Exclusive => KeyboardInteractivity::Exclusive,
            KbFocus::OnDemand => KeyboardInteractivity::OnDemand,
        }
    }
}

#[derive(Debug)]
pub struct Padding<T: FromStr> {
    t: T,
    b: T,
    l: T,
    r: T,
}

impl<T: FromStr + Copy> Padding<T> {
    fn all(v: T) -> Self {
        Padding {
            t: v,
            b: v,
            l: v,
            r: v,
        }
    }
}

impl From<&Padding<i32>> for IcedMargin {
    fn from(p: &Padding<i32>) -> Self {
        IcedMargin {
            top: p.t,
            right: p.r,
            bottom: p.b,
            left: p.l,
        }
    }
}

impl<'de, T> Deserialize<'de> for Padding<T>
where
    T: FromStr,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.trim().split_whitespace().collect();

        let parse = |v: &str| {
            v.parse().map_err(|_| {
                serde::de::Error::invalid_type(serde::de::Unexpected::Str(v), &"a number")
            })
        };

        if let [t, r, b, l] = parts[..] {
            return Ok(Padding {
                t: parse(t)?,
                b: parse(b)?,
                l: parse(l)?,
                r: parse(r)?,
            });
        }

        Err(serde::de::Error::invalid_length(
            4,
            &"expected 1, 2 or 4 arguments",
        ))
    }
}
