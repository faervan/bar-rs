use std::{any::TypeId, collections::HashMap};

use bar_rs_derive::Builder;
use iced::widget::text;

use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    listeners::wayfire::WayfireListener,
    modules::Module,
    Message,
};

#[derive(Debug, Builder)]
pub struct WayfireWindowMod {
    active: Option<String>,
    max_length: usize,
    cfg_override: ModuleConfigOverride,
}

impl Default for WayfireWindowMod {
    fn default() -> Self {
        Self {
            active: None,
            max_length: 25,
            cfg_override: Default::default(),
        }
    }
}

impl WayfireWindowMod {
    pub fn set_active(&mut self, active: Option<String>) {
        self.active = active.map(|title| match title.len() > self.max_length {
            true => format!(
                "{}...",
                &title.chars().take(self.max_length - 3).collect::<String>()
            ),
            false => title,
        });
    }
}

impl Module for WayfireWindowMod {
    fn name(&self) -> String {
        "wayfire.window".to_string()
    }

    fn view(&self, config: &LocalModuleConfig, anchor: &BarAnchor) -> iced::Element<Message> {
        text!("{}", self.active.as_ref().unwrap_or(&String::new()))
            .fill(anchor)
            .size(self.cfg_override.font_size.unwrap_or(config.font_size))
            .color(self.cfg_override.text_color.unwrap_or(config.text_color))
            .into()
    }

    fn requires(&self) -> Vec<std::any::TypeId> {
        vec![TypeId::of::<WayfireListener>()]
    }

    fn read_config(&mut self, config: &HashMap<String, Option<String>>) {
        self.cfg_override = config.into();
        if let Some(max_length) = config
            .get("max_length")
            .and_then(|v| v.as_ref().and_then(|v| v.parse().ok()))
        {
            self.max_length = max_length;
        }
    }
}
