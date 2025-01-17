use std::{any::TypeId, collections::HashMap};

use bar_rs_derive::Builder;
use iced::widget::text;

use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
        parse::StringExt,
    },
    fill::FillExt,
    listeners::niri::NiriListener,
    modules::{require_listener, Module},
    Message,
};

#[derive(Debug, Builder)]
pub struct NiriWindowMod {
    // (title, app_id)
    pub windows: HashMap<u64, (Option<String>, Option<String>)>,
    pub focused: Option<u64>,
    max_length: usize,
    show_app_id: bool,
    cfg_override: ModuleConfigOverride,
}

impl Default for NiriWindowMod {
    fn default() -> Self {
        Self {
            windows: HashMap::new(),
            focused: None,
            max_length: 25,
            show_app_id: false,
            cfg_override: Default::default(),
        }
    }
}

impl NiriWindowMod {
    fn get_title(&self) -> Option<String> {
        self.focused
            .and_then(|id| {
                self.windows.get(&id).and_then(|w| match self.show_app_id {
                    true => w.1.as_ref(),
                    false => w.0.as_ref(),
                })
            })
            .map(|title| match title.len() > self.max_length {
                true => format!(
                    "{}...",
                    &title.chars().take(self.max_length - 3).collect::<String>()
                ),
                false => title.to_string(),
            })
    }
}

impl Module for NiriWindowMod {
    fn name(&self) -> String {
        "niri.window".to_string()
    }

    fn view(&self, config: &LocalModuleConfig, anchor: &BarAnchor) -> iced::Element<Message> {
        list![
            anchor,
            text!["{}", self.get_title().unwrap_or_default()]
                .fill(anchor)
                .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                .color(self.cfg_override.text_color.unwrap_or(config.text_color))
        ]
        .into()
    }

    fn requires(&self) -> Vec<TypeId> {
        vec![require_listener::<NiriListener>()]
    }

    fn read_config(&mut self, config: &HashMap<String, Option<String>>) {
        let default = Self::default();
        self.cfg_override = config.into();
        self.max_length = config
            .get("max_length")
            .and_then(|v| v.as_ref().and_then(|v| v.parse().ok()))
            .unwrap_or(default.max_length);
        self.show_app_id = config
            .get("show_app_id")
            .map(|v| v.into_bool(default.show_app_id))
            .unwrap_or(default.show_app_id);
    }
}
