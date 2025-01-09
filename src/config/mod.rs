use std::{
    any::TypeId,
    collections::HashSet,
    fs::{create_dir_all, File},
    path::PathBuf,
    sync::Arc,
};

use anchor::BarAnchor;
use configparser::ini::Ini;
use directories::ProjectDirs;
pub use enabled_modules::EnabledModules;
use iced::futures::{channel::mpsc::Sender, SinkExt};
use module_config::ModuleConfig;
use tokio::sync::mpsc;

use crate::{registry::Registry, Message};
pub use thrice::Thrice;

pub mod anchor;
mod enabled_modules;
pub mod module_config;
pub mod parse;
mod thrice;

#[derive(Debug)]
pub struct Config {
    pub enabled_modules: EnabledModules,
    pub enabled_listeners: HashSet<TypeId>,
    pub module_config: ModuleConfig,
    pub bar_height: u32,
    pub bar_width: u32,
    pub anchor: BarAnchor,
    pub monitor: Option<String>,
}

impl Config {
    fn default(registry: &Registry) -> Self {
        let enabled_modules = EnabledModules::default();
        Self {
            enabled_listeners: registry
                .enabled_listeners(&enabled_modules)
                .chain(
                    registry
                        .all_listeners()
                        .flat_map(|(l_id, l)| {
                            l.config().into_iter().map(move |option| (l_id, option))
                        })
                        .filter_map(|(l_id, option)| option.default.then_some(*l_id)),
                )
                .collect(),
            enabled_modules,
            module_config: ModuleConfig::default(),
            bar_width: 1920,
            bar_height: 30,
            anchor: BarAnchor::default(),
            monitor: None,
        }
    }

    pub fn exclusive_zone(&self) -> i32 {
        (match self.anchor {
            BarAnchor::Left | BarAnchor::Right => self.bar_width,
            BarAnchor::Top | BarAnchor::Bottom => self.bar_height,
        }) as i32
    }
}

pub fn get_config_dir(registry: &Registry) -> PathBuf {
    let config_dir = ProjectDirs::from("fun.killarchive", "faervan", "bar-rs")
        .map(|dirs| dirs.config_local_dir().to_path_buf())
        .unwrap_or_else(|| {
            eprintln!("Failed to get config directory");
            PathBuf::from("")
        });
    let _ = create_dir_all(&config_dir);
    let config_file = config_dir.join("bar-rs.ini");

    if File::create_new(&config_file).is_ok() {
        let mut ini = Ini::new();
        let config = Config::default(registry);
        registry
            .get_listeners(&config.enabled_listeners)
            .flat_map(|l| l.config().into_iter())
            .for_each(|option| {
                ini.set(
                    &option.section,
                    &option.name,
                    Some(option.default.to_string()),
                );
            });
        ini.set("general", "height", Some(config.bar_height.to_string()));
        ini.set("general", "width", Some(config.bar_width.to_string()));
        ini.set("general", "anchor", Some(config.anchor.into()));
        config.enabled_modules.write_to_ini(&mut ini);
        ini.write(&config_file).unwrap_or_else(|e| {
            panic!(
                "Couldn't persist default config to {}: {e}",
                config_file.to_string_lossy()
            )
        });
    }

    config_file
}

pub fn read_config(path: &PathBuf, registry: &mut Registry) -> Config {
    let mut ini = Ini::new();
    let Ok(_) = ini.load(path) else {
        eprintln!("Failed to read config from {}", path.to_string_lossy());
        return Config::default(registry);
    };
    let config: Config = (&ini, &*registry).into();
    registry
        .get_modules_mut(config.enabled_modules.get_all())
        .filter_map(|m| {
            ini.get_map_ref()
                .get(&format!("module:{}", m.id()))
                .map(|map| (m, map))
        })
        .for_each(|(m, map)| m.read_config(map));
    config
}

pub async fn get_config(sender: &mut Sender<Message>) -> (Arc<PathBuf>, Arc<Config>) {
    let (sx, mut rx) = mpsc::channel(1);
    sender
        .send(Message::GetConfig(sx))
        .await
        .unwrap_or_else(|err| {
            eprintln!("Trying to request config failed with err: {err}");
        });
    rx.recv().await.unwrap()
}

pub struct ConfigEntry {
    pub section: String,
    pub name: String,
    pub default: bool,
}

impl ConfigEntry {
    pub fn new<S: ToString>(section: S, name: S, default: bool) -> Self {
        Self {
            section: section.to_string(),
            name: name.to_string(),
            default,
        }
    }
}
