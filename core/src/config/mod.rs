use clap::Args;
use merge::Merge;
use module::{ModuleLayout, ModuleLayoutOverride};
use optfield::optfield;
use serde::{Deserialize, Serialize};
use toml_example::TomlExample;
use window::{WindowConfig, WindowConfigOverride};

use crate::helpers::merge::overwrite_none;

pub mod load;
pub mod module;
pub mod prepare;
pub mod source;
pub mod style;
pub mod theme;
pub mod window;

#[optfield(
    pub GlobalConfigOverride,
    attrs = (derive(Args, Debug, Clone, Serialize, Deserialize)),
    field_doc,
    field_attrs,
    merge_fn
)]
#[derive(Args, Debug, Clone, Serialize, Deserialize, TomlExample, PartialEq)]
#[serde(default)]
pub struct GlobalConfig {
    #[arg(long)]
    /// Whether to watch the configuration directory for file changes and automatically update the
    /// config. Even if this is true, windows can be individually configured not to be affected.
    pub hot_reloading: bool,
    #[arg(long)]
    /// How often the windows should be updated with new content (in seconds)
    pub reload_interval: f32,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            hot_reloading: true,
            reload_interval: 3.,
        }
    }
}

// Note: all fields from ConfigOptions need to be present for ConfigOptionOverride as well!
#[derive(Args, Debug, Clone, Serialize, Deserialize, TomlExample, PartialEq)]
#[serde(default)]
pub struct ConfigOptions {
    #[arg(long)]
    /// Name of the theme to use
    pub theme: String,

    #[arg(long)]
    /// Name of the style to use
    pub style: String,

    #[arg(long)]
    /// Whether to update apply changes when the configuration files where changed. This will not
    /// overwrite settings set through the IPC.
    ///
    /// This will not take affect if the global `hot_reloading` setting is disabled.
    pub hot_reloading: bool,

    #[command(flatten)]
    #[toml_example(nesting)]
    /// The modules that should be enabled
    pub modules: ModuleLayout,

    #[command(flatten)]
    #[serde(flatten)]
    #[toml_example(nesting)]
    pub window: WindowConfig,
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self {
            theme: format!(
                "iced/{}",
                match darkmode::detect().unwrap_or(darkmode::Mode::Dark) {
                    darkmode::Mode::Light => iced::Theme::Light,
                    darkmode::Mode::Dark | darkmode::Mode::Default => iced::Theme::Dark,
                }
            ),
            style: String::from("crabbar"),
            hot_reloading: true,
            modules: ModuleLayout::default(),
            window: WindowConfig::default(),
        }
    }
}

#[derive(Args, Merge, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigOptionOverride {
    #[arg(long)]
    #[merge(strategy = overwrite_none)]
    /// Name of the theme to use
    pub theme: Option<String>,

    #[arg(long)]
    #[merge(strategy = overwrite_none)]
    /// Name of the style to use
    pub style: Option<String>,

    #[arg(long)]
    #[merge(strategy = overwrite_none)]
    /// Whether to update apply changes when the configuration files where changed. This will not
    /// overwrite settings set through the IPC.
    pub hot_reloading: Option<bool>,

    #[command(flatten)]
    /// The modules that should be enabled
    pub modules: ModuleLayoutOverride,

    #[command(flatten)]
    pub window: WindowConfigOverride,
}

impl ConfigOptions {
    pub fn merge_opt(
        &mut self,
        ConfigOptionOverride {
            theme,
            style,
            hot_reloading,
            modules,
            window,
        }: ConfigOptionOverride,
    ) {
        if let Some(theme) = theme {
            self.theme = theme;
        }
        if let Some(style) = style {
            self.style = style;
        }
        if let Some(hot_reloading) = hot_reloading {
            self.hot_reloading = hot_reloading;
        }
        self.modules.merge_opt(modules);
        self.window.merge_opt(window);
    }
}
