use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use toml_example::TomlExample;
use types::KbFocus;

use crate::directories;

mod types;

#[derive(Debug, Deserialize, TomlExample)]
pub struct Config {
    #[toml_example(default)]
    /// The name of the monitor to open on
    pub monitor: Option<String>,

    #[toml_example(default)]
    #[toml_example(enum)]
    /// Whether the bar should be able to get keyboard focus
    pub kb_focus: KbFocus,

    #[toml_example(default)]
    /// How often the bar should be updated
    pub reload_interval: f64,

    #[toml_example(default)]
    /// Whether to monitor for configuration changes
    pub hot_reloading: bool,

    #[toml_example(default)]
    /// Whether to fully reopen the bar when the configuration changes. This requires hot_reloading
    /// to be true. It is required for some settings like the bar anchor.
    pub hard_reloading: bool,

    #[toml_example(default)]
    /// The theme to use
    pub theme: String,

    #[toml_example(default)]
    /// The style to use
    pub style: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            monitor: Default::default(),
            kb_focus: Default::default(),
            reload_interval: 3.,
            hot_reloading: true,
            hard_reloading: false,
            theme: Default::default(),
            style: Default::default(),
        }
    }
}

impl Config {
    // pub fn exclusive_zone(&self) -> i32 {
    //     (match self.style.anchor {
    //         BarAnchor::Left | BarAnchor::Right => self.style.width.unwrap_or(30),
    //         BarAnchor::Top | BarAnchor::Bottom => self.style.height.unwrap_or(30),
    //     }) as i32
    // }
    //
    // /// Determine the size to pass to `get_layer_surface`
    // pub fn dimension(&self, x: u32, y: u32) -> Option<(Option<u32>, Option<u32>)> {
    //     let [width, height] = match self.style.anchor.is_vertical() {
    //         true => [
    //             self.style.width.unwrap_or(50),
    //             self.style.height.unwrap_or(y),
    //         ],
    //         false => [
    //             self.style.width.unwrap_or(x),
    //             self.style.height.unwrap_or(30),
    //         ],
    //     };
    //     Some((Some(width), Some(height)))
    // }
}
