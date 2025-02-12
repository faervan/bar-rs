use std::{any::TypeId, collections::HashMap};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::widget::{container, rich_text, span, text};
use iced::{
    futures::{channel::mpsc::Sender, SinkExt},
    Element,
};

use crate::config::popup_config::PopupConfig;
use crate::tooltip::ElementExt;
use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    listeners::hyprland::HyprListener,
    modules::{require_listener, Message, Module},
};
use crate::{impl_on_click, impl_wrapper};

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
    pub fn get_title(&self) -> Option<String> {
        self.title
            .as_ref()
            .map(|title| match title.len() > self.max_length {
                true => format!(
                    "{}...",
                    title.chars().take(self.max_length - 3).collect::<String>()
                ),
                false => title.to_string(),
            })
    }
}

impl Module for HyprWindowMod {
    fn name(&self) -> String {
        "hyprland.window".to_string()
    }

    fn active(&self) -> bool {
        self.title.is_some()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        _popup_config: &PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<Message> {
        container(
            rich_text([span(self.get_title().unwrap_or_default())
                .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                .color(self.cfg_override.text_color.unwrap_or(config.text_color))])
            .fill(anchor),
        )
        .padding(self.cfg_override.text_margin.unwrap_or(config.text_margin))
        .tooltip_maybe(
            self.get_title()
                .and_then(|t| (t.len() > self.max_length).then_some(text(t).size(12))),
        )
    }

    impl_wrapper!();

    fn requires(&self) -> Vec<TypeId> {
        vec![require_listener::<HyprListener>()]
    }

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _templates: &mut Handlebars,
    ) {
        self.cfg_override = config.into();
        self.max_length = config
            .get("max_length")
            .and_then(|v| v.as_ref().and_then(|v| v.parse().ok()))
            .unwrap_or(Self::default().max_length);
    }

    impl_on_click!();
}

pub async fn update_window(sender: &mut Sender<Message>, title: Option<String>) {
    sender
        .send(Message::update(move |reg| {
            reg.get_module_mut::<HyprWindowMod>().title = title
        }))
        .await
        .unwrap_or_else(|err| {
            eprintln!("Trying to send workspaces failed with err: {err}");
        });
}
