use std::collections::HashMap;

use bar_rs_derive::Builder;
use chrono::Local;
use handlebars::Handlebars;
use iced::widget::{container, text};
use iced::Element;

use crate::config::popup_config::PopupConfig;
use crate::impl_wrapper;
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
pub struct TimeMod {
    cfg_override: ModuleConfigOverride,
    icon: String,
    fmt: String,
}

impl Default for TimeMod {
    fn default() -> Self {
        Self {
            cfg_override: Default::default(),
            icon: "".to_string(),
            fmt: "%H:%M".to_string(),
        }
    }
}

impl Module for TimeMod {
    fn name(&self) -> String {
        "time".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        _popup_config: &PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<Message> {
        let time = Local::now();
        list![
            anchor,
            container(
                text!("{}", self.icon)
                    .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                    .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                    .font(NERD_FONT)
                    .fill(anchor)
            )
            .fill(anchor)
            .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin)),
            container(
                text!("{}", time.format(&self.fmt))
                    .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                    .color(self.cfg_override.text_color.unwrap_or(config.text_color))
                    .fill(anchor)
            )
            .fill(anchor)
            .padding(self.cfg_override.text_margin.unwrap_or(config.text_margin)),
        ]
        .spacing(self.cfg_override.spacing.unwrap_or(config.spacing))
        .into()
    }

    impl_wrapper!();

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _templates: &mut Handlebars,
    ) {
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
