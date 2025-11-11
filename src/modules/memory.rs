use std::{collections::HashMap, process::Command};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::widget::container;
use iced::{widget::text, Element};

use crate::config::popup_config::PopupConfig;
use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    Message, NERD_FONT,
};
use crate::{impl_on_click, impl_wrapper};

use super::Module;

#[derive(Debug, Default, Builder)]
pub struct MemoryMod {
    cfg_override: ModuleConfigOverride,
    icon: Option<String>,
}

impl Module for MemoryMod {
    fn name(&self) -> String {
        "memory".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        _popup_config: &PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<'_, Message> {
        let usage = Command::new("sh")
            .arg("-c")
            .arg("free | grep Mem | awk '{printf \"%.0f\", $3/$2 * 100.0}'")
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
            .unwrap_or_else(|e| {
                eprintln!("Failed to get memory usage. err: {e}");
                "0".to_string()
            })
            .parse()
            .unwrap_or_else(|e| {
                eprintln!("Failed to parse memory usage (output from free), e: {e}");
                999
            });

        list![
            anchor,
            container(
                text!("{}", self.icon.as_ref().unwrap_or(&"󰍛".to_string()))
                    .fill(anchor)
                    .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                    .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                    .font(NERD_FONT)
            )
            .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin)),
            container(
                text!["{}%", usage]
                    .fill(anchor)
                    .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                    .color(self.cfg_override.text_color.unwrap_or(config.text_color))
            )
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
        self.cfg_override = config.into();
        self.icon = config.get("icon").and_then(|v| v.clone());
    }

    impl_on_click!();
}
