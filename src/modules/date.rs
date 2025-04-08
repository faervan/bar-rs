use std::collections::HashMap;

use bar_rs_derive::Builder;
use chrono::Local;

use crate::template_engine::TemplateEngine;

use super::Module;

#[derive(Debug, Builder)]
pub struct DateMod {
    icon: String,
    date_fmt: String,
    format: String,
}

impl Default for DateMod {
    fn default() -> Self {
        Self {
            icon: "".to_string(),
            date_fmt: "%a, %d. %b".to_string(),
            format: "row(icon({{icon}}), {{date}})".to_string(),
        }
    }
}

impl Module for DateMod {
    fn name(&self) -> String {
        "date".to_string()
    }
    fn context<'a>(&'a self) -> HashMap<String, Box<dyn ToString + Send + Sync>> {
        let time = Local::now();
        create_map!(
            ("date", time.format(&self.date_fmt).to_string()),
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
        self.date_fmt = get("date_fmt", default.date_fmt);
    }
}
