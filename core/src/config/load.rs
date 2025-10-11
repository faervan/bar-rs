use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::Arc,
};

use log::{debug, error, info};
use serde::Deserialize;
use toml::Table;

use crate::{
    config::{MainConfig, style::ContainerStyleOverride},
    module::{CustomModules, Module},
    state::State,
};

impl State {
    pub fn load_config(&mut self) {
        info!("Loading configuration.");
        debug!("ConfigRoot: {:#?}", self.config_root);

        if let Err(e) = self.load_main_config() {
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

        if let Err(e) = self.load_module_config() {
            error!(
                "Failed to load the module config from {:?}: {e}",
                self.config_root.module_dir()
            );
        }
    }

    /// Load both the [GlobalConfig](crate::config::GlobalConfig)
    /// and the [configuration presets](crate::config::ConfigOptions)
    fn load_main_config(&mut self) -> anyhow::Result<()> {
        let main_cfg: MainConfig = parse_from_file(self.config_root.config())?;
        self.config = Arc::new(main_cfg.global);
        self.config_presets = main_cfg.bar;
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

    fn load_module_config(&mut self) -> anyhow::Result<()> {
        self.parse_from_dir(
            self.config_root.module_dir(),
            |state, variant, config: Table| {
                let is_custom = config
                    .get("type")
                    .and_then(|value| value.as_str())
                    .is_some_and(|s| s.to_lowercase() == "custom");
                let module = match is_custom {
                    true => Some((
                        TypeId::of::<CustomModules>(),
                        state.registry.get_module_mut::<CustomModules>() as &mut dyn Module,
                    )),
                    false => state
                        .registry
                        .module_by_name_mut(&variant)
                        .map(|(id, m)| (*id, m.as_mut())),
                };
                if let Some((type_id, module)) = module {
                    let style = config.get("appearance").and_then(|value| {
                        value
                            .clone()
                            .try_into::<ContainerStyleOverride>()
                            .inspect_err(|e| error!("Syntax error in `{variant}.toml`: {e}"))
                            .ok()
                    });

                    let (mut added, mut removed) = (vec![], vec![]);
                    let mut variants: HashSet<String> =
                        HashSet::from_iter(module.variant_names().into_iter().map(str::to_string));
                    module.read_config(&variant, config, &state.engine);

                    // Determine added/removed module variants due to the new config (e.g. a custom
                    // module might have been created or deleted)
                    let mut new = vec![];
                    for variant in module.variant_names() {
                        if !variants.remove(variant) {
                            new.push(variant.to_string());
                        }
                    }
                    added.push((type_id, new));
                    removed.extend(variants.into_iter());

                    for (id, added) in added.into_iter() {
                        state.registry.add_module_names(id, added.into_iter());
                    }
                    state.registry.remove_module_names(removed.into_iter());

                    if let Some(style) = style {
                        state.registry.set_style_override(&type_id, &variant, style);
                    }
                }
            },
        )?;
        debug!("Loaded module configurations");

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
