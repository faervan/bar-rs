use std::{any::TypeId, collections::HashMap};

use bar_rs_derive::Builder;
use iced::{
    futures::{channel::mpsc::Sender, SinkExt},
    widget::text,
};

use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    listeners::hyprland::HyprListener,
    modules::{require_listener, Message, Module},
};

#[derive(Debug, Builder)]
pub struct HyprWindowMod {
    title: Option<String>,
    max_length: usize,
    cfg_override: ModuleConfigOverride,
}

impl Default for HyprWindowMod {
    fn default() -> Self {
        Self {
            title: None,
            max_length: 25,
            cfg_override: Default::default(),
        }
    }
}

impl HyprWindowMod {
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title.map(|title| match title.len() > self.max_length {
            true => format!(
                "{}...",
                &title.chars().take(self.max_length - 3).collect::<String>()
            ),
            false => title,
        });
    }
}

impl Module for HyprWindowMod {
    fn name(&self) -> String {
        "hyprland.window".to_string()
    }

    fn view(&self, config: &LocalModuleConfig, anchor: &BarAnchor) -> iced::Element<Message> {
        list![
            anchor,
            text!["{}", self.title.as_ref().unwrap_or(&String::new())]
                .fill(anchor)
                .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                .color(self.cfg_override.text_color.unwrap_or(config.text_color))
        ]
        .into()
    }

    fn requires(&self) -> Vec<TypeId> {
        vec![require_listener::<HyprListener>()]
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

pub async fn update_window(sender: &mut Sender<Message>, title: Option<String>) {
    sender
        .send(Message::update(move |reg| {
            reg.get_module_mut::<HyprWindowMod>().set_title(title)
        }))
        .await
        .unwrap_or_else(|err| {
            eprintln!("Trying to send workspaces failed with err: {err}");
        });
}
