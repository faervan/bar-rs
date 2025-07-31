use clap::Args;
use module::{ModuleLayout, ModuleLayoutOverride};
use serde::{Deserialize, Serialize};

pub mod module;
pub mod source;
pub mod style;
pub mod theme;

fn default_theme() -> String {
    format!(
        "iced/{}",
        match darkmode::detect().unwrap_or(darkmode::Mode::Dark) {
            darkmode::Mode::Light => iced::Theme::Light,
            darkmode::Mode::Dark | darkmode::Mode::Default => iced::Theme::Dark,
        }
    )
}
fn default() -> String {
    String::from("crabbar")
}

#[derive(Args, Debug, Serialize, Deserialize)]
pub struct ConfigOptions {
    #[arg(long, default_value = default_theme())]
    #[serde(default = "default_theme")]
    /// Name of the theme to use
    pub theme: String,

    #[arg(long, default_value = default())]
    #[serde(default = "default")]
    /// Name of the style to use
    pub style: String,

    #[command(flatten)]
    /// The modules that should be enabled
    pub modules: ModuleLayout,
}

#[derive(Args, Debug)]
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
}

impl ConfigOptions {
    pub fn merge_opt(
        &mut self,
        ConfigOptionOverride {
            theme,
            style,
            modules,
        }: ConfigOptionOverride,
    ) {
        if let Some(theme) = theme {
            self.theme = theme;
        }
        if let Some(style) = style {
            self.style = style;
        }
        self.modules.merge_opt(modules);
    }
}
