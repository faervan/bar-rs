use std::{any::TypeId, collections::HashMap};

use bar_rs_derive::Builder;
use iced::widget::text;

use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
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
    cfg_override: ModuleConfigOverride,
}

impl Default for NiriWindowMod {
    fn default() -> Self {
        Self {
            windows: HashMap::new(),
            focused: None,
            max_length: 25,
            cfg_override: Default::default(),
        }
    }
}

/*impl NiriWindowMod {
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title.map(|title| match title.len() > self.max_length {
            true => format!(
                "{}...",
                &title.chars().take(self.max_length - 3).collect::<String>()
            ),
            false => title,
        });
    }
}*/

impl Module for NiriWindowMod {
    fn name(&self) -> String {
        "niri.window".to_string()
    }

    fn view(&self, config: &LocalModuleConfig, anchor: &BarAnchor) -> iced::Element<Message> {
        list![
            anchor,
            text![
                "{}",
                self.focused
                    .and_then(|id| self.windows.get(&id).and_then(|w| w.0.as_ref()))
                    .unwrap_or(&String::new())
            ]
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
        self.cfg_override = config.into();
        if let Some(max_length) = config
            .get("max_length")
            .and_then(|v| v.as_ref().and_then(|v| v.parse().ok()))
        {
            self.max_length = max_length;
        }
    }
}
