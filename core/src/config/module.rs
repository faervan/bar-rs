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
#[derive(Args, Debug, Clone, Serialize, Deserialize, TomlExample)]
pub struct ModuleLayout {
    #[arg(long = "modules_left")]
    /// Modules that should be displayed on the left of the bar
    pub left: Vec<String>,

    #[arg(long = "modules_center")]
    /// Modules that should be displayed on the center of the bar
    pub center: Vec<String>,

    #[arg(long = "modules_right")]
    /// Modules that should be displayed on the right of the bar
    pub right: Vec<String>,
}
