use std::{fs::{create_dir_all, File}, path::PathBuf};

use configparser::ini::Ini;
use directories::ProjectDirs;
pub use enabled_modules::EnabledModules;

use crate::modules::hyprland::get_monitor_name;

mod enabled_modules;

#[derive(Default, Debug)]
pub struct Config {
    pub enabled_modules: EnabledModules,
    pub monitor: String,
}


pub fn get_config_dir() -> PathBuf {
    let config_dir = ProjectDirs::from("fun.killarchive", "faervan foss", "bar-rs")
        .map(|dirs| dirs.config_local_dir().to_path_buf())
        .unwrap_or_else(|| {
            eprintln!("Failed to get config directory, defaulting to '~/.config'");
            PathBuf::from("~/.config")
        });
    let _ = create_dir_all(&config_dir);
    let config_file = config_dir.join("bar-rs.ini");

    if File::create_new(&config_file).is_ok() {
        let mut ini = Ini::new();
        EnabledModules::default().write_to_ini(&mut ini);
        ini.set("general", "monitor", Some(get_monitor_name()));
        ini.write(&config_file)
            .unwrap_or_else(|e|
                panic!("Couldn't persist default config to {}: {e}",
                    config_file.to_string_lossy())
            );
    }

    config_file
}

pub fn read_config(path: &PathBuf) -> Config {
    let mut ini = Ini::new();
    let Ok(_) = ini.load(path) else {
        eprintln!("Failed to read config from {}", path.to_string_lossy());
        return Config::default();
    };
    (&ini).into()
}

impl From<&Ini> for Config {
    fn from(ini: &Ini) -> Self {
        Self {
            enabled_modules: ini.into(),
            monitor: ini.get("general", "monitor")
                .unwrap_or(get_monitor_name()),
        }
    }
}

#[allow(dead_code)]
trait StringExt {
    fn as_bool(self) -> Option<bool>;
}

impl StringExt for String {
    fn as_bool(self) -> Option<bool> {
        match self.to_lowercase().as_str() {
            "0" | "f" | "false" | "disabled" | "disable" | "off" => Some(false),
            "1" | "t" | "true" | "enabled" | "enable" | "on" => Some(true),
            _ => None
        }
    }
}
