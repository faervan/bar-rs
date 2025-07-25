use std::{fs, io::Write, path::PathBuf};

use serde::Deserialize;
use style::BarStyle;
use theme::BarTheme;
use types::{BarAnchor, KbFocus};

mod directories;
mod insets;
mod style;
mod theme;
/// All the custom config types and their parsing implementations
mod types;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub path: Option<String>,
    pub monitor: Option<String>,
    pub kb_focus: KbFocus,
    pub reload_interval: f64,
    pub hot_reloading: bool,
    pub hard_reloading: bool,
    pub theme: BarTheme,
    pub style: BarStyle,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: directories::config(),
            monitor: None,
            kb_focus: KbFocus::None,
            reload_interval: 3.,
            hot_reloading: true,
            hard_reloading: false,
            theme: BarTheme::default(),
            style: BarStyle::default(),
        }
    }
}

impl Config {
    pub fn load(path: Option<&String>) -> anyhow::Result<Self> {
        let default = Config::default();
        let path = PathBuf::from(
            path.unwrap_or(
                default
                    .path
                    .as_ref()
                    .ok_or(anyhow::anyhow!("Could not find a config path."))?,
            ),
        );

        let config_dir = path.parent().unwrap();
        if !matches!(fs::exists(config_dir), Ok(true)) {
            fs::create_dir(config_dir)?;
            log::info!(
                "Successfully created the configuration directory at {}.",
                config_dir.display()
            );
        }

        if !matches!(fs::exists(&path), Ok(true)) {
            let mut file = fs::File::create(&path)?;
            if let Ok(default) = fs::read(directories::default_config()) {
                file.write(&default)?;
                log::info!(
                    "The default configuration has been created at {}.",
                    path.display()
                );
            } else {
                log::error!(
                    "The default configuration could not be found at {}.",
                    directories::default_config()
                );
                log::info!(
                    "An empty configuration has been created at {}.",
                    path.display()
                );
            }
        }

        let config = config::Config::builder()
            .set_default("path", default.path)?
            .set_default("kb_focus", default.kb_focus)?
            .set_default("reload_interval", default.reload_interval)?
            .set_default("hot_reloading", default.hot_reloading)?
            .set_default("hard_reloading", default.hard_reloading)?
            .set_default("theme", default.theme)?
            .set_default("style", default.style)?
            .add_source(config::File::from(path))
            .build()?;
        Ok(config.try_deserialize()?)
    }

    pub fn exclusive_zone(&self) -> i32 {
        (match self.style.anchor {
            BarAnchor::Left | BarAnchor::Right => self.style.width.unwrap_or(30),
            BarAnchor::Top | BarAnchor::Bottom => self.style.height.unwrap_or(30),
        }) as i32
    }

    /// Determine the size to pass to `get_layer_surface`
    pub fn dimension(&self, x: u32, y: u32) -> Option<(Option<u32>, Option<u32>)> {
        let [width, height] = match self.style.anchor.is_vertical() {
            true => [
                self.style.width.unwrap_or(50),
                self.style.height.unwrap_or(y),
            ],
            false => [
                self.style.width.unwrap_or(x),
                self.style.height.unwrap_or(30),
            ],
        };
        Some((Some(width), Some(height)))
    }
}
