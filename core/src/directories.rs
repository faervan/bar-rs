use std::path::PathBuf;

pub fn runtime_dir() -> std::ffi::OsString {
    let mut fallback_dir = from_env_or("/tmp", "XDG_RUNTIME_DIR");
    fallback_dir.push("/crabbar");
    from_env_or(fallback_dir, "CRABBAR_RUN_DIR")
}

pub fn log_dir() -> std::ffi::OsString {
    let home = std::env::var("HOME").unwrap();
    let fallback_dir = from_env_or(format!("{home}/.local/state"), "XDG_STATE_HOME");
    from_env_or(fallback_dir, "CRABBAR_LOG_DIR")
}

pub fn config_dir() -> std::ffi::OsString {
    let home = std::env::var("HOME").unwrap();
    let mut fallback_dir = from_env_or(format!("{home}/.config"), "XDG_CONFIG_HOME");
    fallback_dir.push("/crabbar");

    from_env_or(fallback_dir, "CRABBAR_CONFIG_DIR")
}

pub struct ConfigRoot(PathBuf);

impl ConfigRoot {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self(path.into())
    }

    /// `TODO`! This cannot be predefined, it has to depend on the packaging.
    /// https://refspecs.linuxfoundation.org/FHS_3.0/fhs/ch04s11.html
    /// https://wiki.archlinux.org/title/Arch_package_guidelines#Directories
    pub fn default_config_dir() -> Self {
        ConfigRoot(PathBuf::from("/usr/share/crabbar"))
    }

    pub fn config(&self) -> PathBuf {
        self.0.join("config.toml")
    }
    pub fn theme_dir(&self) -> PathBuf {
        self.0.join("themes")
    }
    pub fn style_dir(&self) -> PathBuf {
        self.0.join("styles")
    }
    pub fn module_dir(&self) -> PathBuf {
        self.0.join("modules")
    }
    pub fn source_dir(&self) -> PathBuf {
        self.0.join("sources")
    }
}

fn from_env_or<S: AsRef<std::ffi::OsStr>, T: Into<std::ffi::OsString>>(
    default: T,
    key: S,
) -> std::ffi::OsString {
    std::env::var(key)
        .map(Into::into)
        .unwrap_or_else(|_| default.into())
}
