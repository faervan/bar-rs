use std::collections::HashMap;

use bar_rs_derive::Builder;
use chrono::Local;
use iced::{
    widget::{row, text},
    Length::Fill,
};

use crate::{
    config::module_config::{LocalModuleConfig, ModuleConfigOverride},
    Message, NERD_FONT,
};

use super::Module;

#[derive(Debug, Default, Builder)]
pub struct TimeMod {
    cfg_override: ModuleConfigOverride,
}

impl Module for TimeMod {
    fn id(&self) -> String {
        "time".to_string()
    }

    fn view(&self, config: &LocalModuleConfig) -> iced::Element<Message> {
        let time = Local::now();
        row![
            text!("")
                .center()
                .height(Fill)
                .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                .font(NERD_FONT),
            text![" {}", time.format("%H:%M")]
                .center()
                .height(Fill)
                .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                .color(self.cfg_override.text_color.unwrap_or(config.text_color)),
        ]
        .spacing(10)
        .into()
    }

    fn read_config(&mut self, config: &HashMap<String, Option<String>>) {
        self.cfg_override = config.into();
    }
}
