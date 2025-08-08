use clap::Args;
use module::{ModuleLayout, ModuleLayoutOverride};
use optfield::optfield;
use serde::{Deserialize, Serialize};
use toml_example::TomlExample;
use window::{WindowConfig, WindowConfigOverride};

pub mod module;
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
#[derive(Args, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GlobalConfig {
    #[arg(long)]
    /// How often the windows should be updated with new content (in seconds)
    pub reload_interval: f32,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            reload_interval: 3.,
        }
    }
}

// Note: all fields from ConfigOptions need to be present for ConfigOptionOverride as well!
#[derive(Args, Debug, Clone, Serialize, Deserialize, TomlExample)]
#[serde(default)]
pub struct ConfigOptions {
    #[arg(long)]
    #[toml_example(default)]
    /// Name of the theme to use
    pub theme: String,

    #[arg(long)]
    /// Name of the style to use
    pub style: String,

    #[command(flatten)]
    /// The modules that should be enabled
    pub modules: ModuleLayout,

    #[command(flatten)]
    #[serde(flatten)]
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
            modules: ModuleLayout {
                left: vec!["workspaces".to_string(), "window".to_string()],
                center: vec!["date".to_string(), "time".to_string()],
                right: vec![
                    "mpris".to_string(),
                    "volume".to_string(),
                    "cpu".to_string(),
                    "memory".to_string(),
                    "disk_space".to_string(),
                ],
            },
            window: WindowConfig::default(),
        }
    }
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOptionOverride {
    #[arg(long)]
    /// Name of the theme to use
    pub theme: Option<String>,

    #[arg(long)]
    /// Name of the style to use
    pub style: Option<String>,

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
        self.modules.merge_opt(modules);
        self.window.merge_opt(window);
    }
}
