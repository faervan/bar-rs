use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use log::{debug, error, info};
use serde::Deserialize;

use crate::state::State;

impl State {
    pub fn load_config(&mut self) {
        info!("Loading configuration.");
        debug!("ConfigRoot: {:#?}", self.config_root);

        if let Err(e) = self.load_global_config() {
            error!(
                "Failed to load global configuration from {:?}: {e}",
                self.config_root.config()
            );
        }

        self.themes = HashMap::new();
        if let Err(e) = self.load_themes() {
            error!(
                "Failed to load themes from {:?}: {e}",
                self.config_root.theme_dir()
            );
        }

        self.styles = HashMap::new();
        if let Err(e) = self.load_styles() {
            error!(
                "Failed to load styles from {:?}: {e}",
                self.config_root.style_dir()
            );
        }
    }

    fn load_global_config(&mut self) -> anyhow::Result<()> {
        self.config = Arc::new(parse_from_file(self.config_root.config())?);
        debug!("Loaded global configuration");

        Ok(())
    }

    fn load_themes(&mut self) -> anyhow::Result<()> {
        self.parse_from_dir(self.config_root.theme_dir(), |state, theme_name, theme| {
            state.themes.insert(theme_name, theme);
        })?;
        debug!("Loaded themes");

        Ok(())
    }

    fn load_styles(&mut self) -> anyhow::Result<()> {
        self.parse_from_dir(self.config_root.style_dir(), |state, style_name, style| {
            state.styles.insert(style_name, style);
        })?;
        debug!("Loaded styles");

        Ok(())
    }

    /// `entry_handler` takes the filename (`.toml` stripped) and the parsed content [T] to put this
    /// value into the [State]
    fn parse_from_dir<T: for<'a> Deserialize<'a>>(
        &mut self,
        dir_path: PathBuf,
        entry_handler: fn(&mut State, String, T),
    ) -> anyhow::Result<()> {
        for entry in fs::read_dir(dir_path)?.flat_map(|t| {
            t.inspect_err(|e| error!("Invalid directory entry: {e}"))
                .ok()
        }) {
            let name = entry.file_name().into_string().map_err(|os_string| {
                anyhow::anyhow!("{os_string:?} cannot be converted to a valid Rust String.")
            })?;

            let Some(cfg_name) = name.strip_suffix(".toml") else {
                debug!("Skipped `{name}` because it does not have a `.toml` extension");
                continue;
            };

            let cfg = parse_from_file(entry.path())?;
            entry_handler(self, cfg_name.to_string(), cfg);
        }

        Ok(())
    }
}

fn parse_from_file<T: for<'a> Deserialize<'a>>(path: PathBuf) -> anyhow::Result<T> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}
