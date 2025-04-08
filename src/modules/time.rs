use std::collections::BTreeMap;

use bar_rs_derive::Builder;
use chrono::Local;

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
    fn context<'a>(&'a self) -> BTreeMap<String, Box<dyn ToString + Send + Sync>> {
        let time = Local::now();
        BTreeMap::from([
            (
                "time".to_string(),
                Box::new(time.format(&self.time_fmt).to_string())
                    as Box<dyn ToString + Send + Sync>,
            ),
            ("icon".to_string(), Box::new(self.icon.clone())),
        ])
    }
    fn module_format(&self) -> &str {
        &self.format
    }
    fn read_config<'a>(
        &mut self,
        config: &std::collections::HashMap<String, Option<String>>,
        _popup_config: &std::collections::HashMap<String, Option<String>>,
        _engine: &mut crate::template_engine::TemplateEngine,
    ) {
        if let Some(format) = config.get("format").and_then(|f| f.clone()) {
            self.format = format;
        }
    }
}
