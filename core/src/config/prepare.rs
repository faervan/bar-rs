use std::collections::HashMap;

use crate::{
    config::{style::ContainerStyle, theme::Theme, ConfigOptions},
    window::WindowRuntimeOptions,
};

/// Load the configured presets, then merge them with the [WindowRuntimeOptions]
pub fn merge_config(
    opts: &WindowRuntimeOptions,
    config_presets: &HashMap<String, ConfigOptions>,
    theme_presets: &HashMap<String, Theme>,
    style_presets: &HashMap<String, ContainerStyle>,
) -> (ConfigOptions, Theme, ContainerStyle) {
    let mut config = match config_presets.get(&opts.name) {
        Some(config) => config.clone(),
        None => {
            log::error!("No such configuration: {}", opts.name);
            ConfigOptions::default()
        }
    };
    config.merge_opt(opts.config.clone());

    // TODO! Handle iced builtin themes
    let mut theme = match theme_presets.get(&config.theme) {
        Some(theme) => theme.clone(),
        None => {
            log::error!("No such theme: {}", config.theme);
            Theme::default()
        }
    };
    theme.merge_opt(opts.theme.clone());

    let mut style = match style_presets.get(&config.style) {
        Some(style) => style.clone(),
        None => {
            log::error!("No such style: {}", config.style);
            ContainerStyle::default()
        }
    };
    style.merge_opt(opts.style.clone());

    (config, theme, style)
}
