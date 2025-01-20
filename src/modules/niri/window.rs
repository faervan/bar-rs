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
    fn get_title(&self) -> Option<&String> {
        self.focused.and_then(|id| {
            self.windows.get(&id).and_then(|w| match self.show_app_id {
                true => w.1.as_ref(),
                false => w.0.as_ref(),
            })
        })
    }

    fn trimmed_title(&self) -> String {
        self.get_title()
            .map(|title| match title.len() > self.max_length {
                true => format!(
                    "{}...",
                    &title.chars().take(self.max_length - 3).collect::<String>()
                ),
                false => title.to_string(),
            })
            .unwrap_or_default()
    }
}

impl Module for NiriWindowMod {
    fn name(&self) -> String {
        "niri.window".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<Message> {
        /*container(
            rich_text([span(self.trimmed_title())
                .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                .color(self.cfg_override.text_color.unwrap_or(config.text_color))])
            .fill(anchor),
        )
        .padding(self.cfg_override.text_margin.unwrap_or(config.text_margin))
        .tooltip_maybe(
            self.get_title()
                .and_then(|t| (t.len() > self.max_length).then_some(text(t).size(12))),
        )*/
        iced::widget::button(text("press me")).on_press(Message::Hello).into()
    }

    impl_wrapper!();

    fn requires(&self) -> Vec<TypeId> {
        vec![require_listener::<NiriListener>()]
    }

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _templates: &mut Handlebars,
    ) {
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
