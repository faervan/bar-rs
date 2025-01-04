use std::{any::TypeId, collections::HashSet, fs::{create_dir_all, File}, path::PathBuf, sync::Arc};

use configparser::ini::Ini;
use directories::ProjectDirs;
pub use enabled_modules::EnabledModules;
use iced::futures::{channel::mpsc::Sender, SinkExt};
use tokio::sync::mpsc;

use crate::{modules::hyprland::get_monitor_name, registry::Registry, Message};

mod enabled_modules;

#[derive(Debug)]
pub struct Config {
    pub close_on_fullscreen: bool,
    pub enabled_modules: EnabledModules,
    pub enabled_listeners: HashSet<TypeId>,
    pub monitor: String,
}

impl Config {
    fn default(registry: &Registry) -> Self {
        let enabled_modules = EnabledModules::default();
        Self {
            close_on_fullscreen: true,
            enabled_listeners: registry.enabled_listeners(&enabled_modules)
                .chain(
                    registry.all_listeners()
                        .flat_map(|(l_id, l)|
                            l.config()
                                .into_iter()
                                .map(move |option| (l_id, option))
                        )
                        .filter_map(|(l_id, option)|
                            option.default.then_some(*l_id)
                        )
                ).collect(),
            enabled_modules,
            monitor: get_monitor_name(),
        }
    }
}

pub fn get_config_dir(registry: &Registry) -> PathBuf {
    let config_dir = ProjectDirs::from("fun.killarchive", "faervan foss", "bar-rs")
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
        registry.get_listeners(&config.enabled_listeners)
            .flat_map(|l| l.config().into_iter())
            .for_each(|option| {
                ini.set(&option.section, &option.name, Some(option.default.to_string()));
            });
        config.enabled_modules.write_to_ini(&mut ini);
        ini.set("general", "monitor",
            Some(config.monitor));
        ini.write(&config_file)
            .unwrap_or_else(|e|
                panic!("Couldn't persist default config to {}: {e}",
                    config_file.to_string_lossy())
            );
    }

    config_file
}

pub fn read_config(path: &PathBuf, registry: &Registry) -> Config {
    let mut ini = Ini::new();
    let Ok(_) = ini.load(path) else {
        eprintln!("Failed to read config from {}", path.to_string_lossy());
        return Config::default(registry);
    };
    (&ini, registry).into()
}

pub async fn get_config(sender: &mut Sender<Message>) -> (Arc<PathBuf>, Arc<Config>) {
    let (sx, mut rx) = mpsc::channel(1);
    sender.send(Message::GetConfig(sx))
        .await
        .unwrap_or_else(|err| {
            eprintln!("Trying to request config failed with err: {err}");
        });
    rx.recv().await.unwrap()
}

impl From<(&Ini, &Registry)> for Config {
    fn from((ini, registry): (&Ini, &Registry)) -> Self {
        let enabled_modules = ini.into();
        Self {
            close_on_fullscreen: ini.get("general", "close_on_fullscreen")
                .as_bool(true),
            enabled_listeners: registry.all_listeners()
                .fold(vec![], |mut acc, (id, l)| {
                    l.config().into_iter().for_each(
                        |option| if ini.get(&option.section, &option.name).as_bool(option.default) {
                            acc.push(*id);
                        }
                    );
                    acc
                })
                .into_iter()
                .chain(registry.enabled_listeners(&enabled_modules))
                .collect(),
            enabled_modules,
            monitor: ini.get("general", "monitor")
                .unwrap_or(get_monitor_name()),
        }
    }
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

trait StringExt {
    fn as_bool(self, default: bool) -> bool;
}

impl StringExt for Option<String> {
    fn as_bool(self, default: bool) -> bool {
        self.and_then(|v| match v.to_lowercase().as_str() {
            "0" | "f" | "false" | "disabled" | "disable" | "off" => Some(false),
            "1" | "t" | "true" | "enabled" | "enable" | "on" => Some(true),
            _ => None
        })
        .unwrap_or(default)
    }
}
