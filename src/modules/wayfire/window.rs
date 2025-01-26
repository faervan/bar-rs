use std::{any::TypeId, collections::HashMap};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::widget::{container, rich_text, span, text};
use iced::Element;

use crate::impl_wrapper;
use crate::tooltip::ElementExt;
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
    pub title: Option<String>,
    max_length: usize,
    cfg_override: ModuleConfigOverride,
}

impl Default for WayfireWindowMod {
    fn default() -> Self {
        Self {
            title: None,
            max_length: 25,
            cfg_override: Default::default(),
        }
    }
}

impl WayfireWindowMod {
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

impl Module for WayfireWindowMod {
    fn name(&self) -> String {
        "wayfire.window".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
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

    fn requires(&self) -> Vec<std::any::TypeId> {
        vec![TypeId::of::<WayfireListener>()]
    }

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _templates: &mut Handlebars,
    ) {
        self.cfg_override = config.into();
        self.max_length = config
            .get("max_length")
            .and_then(|v| v.as_ref().and_then(|v| v.parse().ok()))
            .unwrap_or(Self::default().max_length);
    }
}
