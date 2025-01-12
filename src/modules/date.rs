use std::collections::HashMap;

use bar_rs_derive::Builder;
use chrono::Local;
use iced::widget::text;

use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    Message, NERD_FONT,
};

use super::Module;

#[derive(Debug, Builder)]
pub struct DateMod {
    cfg_override: ModuleConfigOverride,
    icon: String,
    fmt: String,
}

impl Default for DateMod {
    fn default() -> Self {
        Self {
            cfg_override: Default::default(),
            icon: "".to_string(),
            fmt: "%a, %d. %b".to_string(),
        }
    }
}

impl Module for DateMod {
    fn id(&self) -> String {
        "date".to_string()
    }

    fn view(&self, config: &LocalModuleConfig, anchor: &BarAnchor) -> iced::Element<Message> {
        let time = Local::now();
        list![
            anchor,
            text!("{}", self.icon)
                .fill(anchor)
                .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                .font(NERD_FONT),
            text!["{}", time.format(&self.fmt)]
                .fill(anchor)
                .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                .color(self.cfg_override.text_color.unwrap_or(config.text_color)),
        ]
        .spacing(self.cfg_override.spacing.unwrap_or(config.spacing))
        .into()
    }

    fn read_config(&mut self, config: &HashMap<String, Option<String>>) {
        let default = Self::default();
        self.cfg_override = config.into();
        self.icon = config
            .get("icon")
            .and_then(|v| v.clone())
            .unwrap_or(default.icon);
        self.fmt = config
            .get("format")
            .and_then(|v| v.clone())
            .unwrap_or(default.fmt);
    }
}
