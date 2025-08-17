use clap::Args;
use optfield::optfield;
use serde::{Deserialize, Serialize};
use toml_example::TomlExample;

#[optfield(
    pub ModuleLayoutOverride,
    attrs = (derive(Args, Debug, Clone, Serialize, Deserialize)),
    field_doc,
    field_attrs,
    merge_fn = pub,
)]
#[derive(Args, Debug, Clone, Serialize, Deserialize, TomlExample, PartialEq)]
#[serde(default)]
pub struct ModuleLayout {
    #[arg(short = 'L', long = "module_left")]
    /// Modules that should be displayed on the left of the bar
    pub left: Vec<String>,

    #[arg(short = 'C', long = "module_center")]
    /// Modules that should be displayed on the center of the bar
    pub center: Vec<String>,

    #[arg(short = 'R', long = "module_right")]
    /// Modules that should be displayed on the right of the bar
    pub right: Vec<String>,
}

impl Default for ModuleLayout {
    fn default() -> Self {
        ModuleLayout {
            left: vec!["workspaces".to_string(), "window".to_string()],
            center: vec!["date".to_string(), "time".to_string()],
            right: vec![
                "mpris".to_string(),
                "volume".to_string(),
                "cpu".to_string(),
                "memory".to_string(),
                "disk_space".to_string(),
            ],
        }
    }
}
