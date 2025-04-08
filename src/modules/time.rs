use std::collections::HashMap;

use bar_rs_derive::Builder;
use chrono::Local;

use crate::template_engine::TemplateEngine;

use super::Module;

#[derive(Debug, Builder)]
pub struct TimeMod {
    icon: String,
    time_fmt: String,
    format: String,
}

impl Default for TimeMod {
    fn default() -> Self {
        Self {
            icon: "".to_string(),
            time_fmt: "%H:%M".to_string(),
            format: "row(icon({{icon}}), {{time}})".to_string(),
        }
    }
}

impl Module for TimeMod {
    fn name(&self) -> String {
        "time".to_string()
    }
    fn context<'a>(&'a self) -> HashMap<String, Box<dyn ToString + Send + Sync>> {
        let time = Local::now();
        create_map!(
            ("time", time.format(&self.time_fmt).to_string()),
            ("icon", self.icon.clone())
        )
    }
    fn module_format(&self) -> &str {
        &self.format
    }
    fn read_config<'a>(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _engine: &mut TemplateEngine,
    ) {
        let get = |cfg, default| config.get(cfg).cloned().flatten().unwrap_or(default);
        let default = Self::default();

        self.icon = get("icon", default.icon);
        self.format = get("format", default.format);
        self.time_fmt = get("time_fmt", default.time_fmt);
    }
}
