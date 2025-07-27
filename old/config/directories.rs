const PROJECT_NAME: &str = "crabbar";

pub fn config_dir() -> Option<String> {
    if let Ok(path) = std::env::var("CRABBAR_CONFIG_PATH") {
        return Some(path);
    }

    if let Ok(cfg_home) = std::env::var("XDG_CONFIG_HOME") {
        return Some(format!("{cfg_home}/{PROJECT_NAME}"));
    }

    if let Ok(home) = std::env::var("HOME") {
        return Some(format!("{home}/.config/{PROJECT_NAME}"));
    }

    log::error!(
        "Failed to determine a configuration directory.\
        You have to set the CRABBAR_CONFIG_PATH environment variable manually."
    );

    None
}

pub fn config() -> Option<String> {
    config_dir().map(|mut dir| {
        dir.push_str("/config.toml");
        dir
    })
}

pub fn theme_dir() -> Option<String> {
    config_dir().map(|mut dir| {
        dir.push_str("/themes");
        dir
    })
}

pub fn style_dir() -> Option<String> {
    config_dir().map(|mut dir| {
        dir.push_str("/styles");
        dir
    })
}

/// TODO! This cannot be predefined, it has to depend on the packaging.
/// https://refspecs.linuxfoundation.org/FHS_3.0/fhs/ch04s11.html
/// https://wiki.archlinux.org/title/Arch_package_guidelines#Directories
fn default_config_dir() -> &'static str {
    "/usr/share/crabbar"
}

pub fn default_config() -> String {
    format!("{}/config.toml", default_config_dir())
}

pub fn default_theme_dir() -> String {
    format!("{}/themes", default_config_dir())
}

pub fn default_style_dir() -> String {
    format!("{}/styles", default_config_dir())
}
