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
    listeners::hyprland::HyprListener,
    modules::{require_listener, Message, Module},
};

#[derive(Debug, Default, Builder)]
pub struct HyprWindowMod {
    window: Option<String>,
    cfg_override: ModuleConfigOverride,
}

impl Module for HyprWindowMod {
    fn id(&self) -> String {
        "hyprland.window".to_string()
    }

    fn view(&self, config: &LocalModuleConfig, anchor: &BarAnchor) -> iced::Element<Message> {
        list![
            anchor,
            text!["{}", self.window.as_ref().unwrap_or(&"".to_string())]
                .center()
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
    }
}

pub async fn update_window(sender: &mut Sender<Message>, window: Option<String>) {
    sender
        .send(Message::update(move |reg| {
            reg.get_module_mut::<HyprWindowMod>().window = window
        }))
        .await
        .unwrap_or_else(|err| {
            eprintln!("Trying to send workspaces failed with err: {err}");
        });
}
