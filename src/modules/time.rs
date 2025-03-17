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
    fn context(&self) -> BTreeMap<&str, Box<dyn ToString + '_>> {
        let time = Local::now();
        let mut map: BTreeMap<&str, Box<dyn ToString>> = BTreeMap::new();
        map.insert("time", Box::new(time.format(&self.fmt)));
        map
    }
    fn module_format(&self) -> &str {
        "row(icon(), {{time}})"
    }
}
