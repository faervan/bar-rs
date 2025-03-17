use std::collections::BTreeMap;

use bar_rs_derive::Builder;
use chrono::Local;

use super::Module;

#[derive(Debug, Builder)]
pub struct TimeMod {
    icon: String,
    fmt: String,
}

impl Default for TimeMod {
    fn default() -> Self {
        Self {
            icon: "".to_string(),
            fmt: "%H:%M".to_string(),
        }
    }
}

impl Module for TimeMod {
    fn name(&self) -> String {
        "time".to_string()
    }
    fn context<'a>(&'a self) -> BTreeMap<String, Box<dyn ToString + Send + Sync>> {
        let time = Local::now();
        let mut map: BTreeMap<String, Box<dyn ToString + Send + Sync>> = BTreeMap::new();
        map.insert("time".to_string(), Box::new(time.format(&self.fmt).to_string()));
        map
    }
    fn module_format(&self) -> String {
        let time = Local::now();
        format!("row(icon(), {})", time.format(&self.fmt))
    }
}
