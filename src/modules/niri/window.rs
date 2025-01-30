use std::collections::BTreeMap;
use std::{any::TypeId, collections::HashMap};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::widget::button::Style;
use iced::widget::{container, scrollable, text};
use iced::Element;
use niri_ipc::Window;

use crate::button::button;
use crate::config::popup_config::{PopupConfig, PopupConfigOverride};
use crate::helpers::UnEscapeString;
use crate::impl_wrapper;
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
    pub windows: HashMap<u64, Window>,
    pub focused: Option<u64>,
    max_length: usize,
    show_app_id: bool,
    cfg_override: ModuleConfigOverride,
    popup_cfg_override: PopupConfigOverride,
}

impl Default for NiriWindowMod {
    fn default() -> Self {
        Self {
            windows: HashMap::new(),
            focused: None,
            max_length: 25,
            show_app_id: false,
            cfg_override: Default::default(),
            popup_cfg_override: PopupConfigOverride {
                width: Some(400),
                height: Some(250),
                ..Default::default()
            },
        }
    }
}

impl NiriWindowMod {
    fn get_title(&self) -> Option<&String> {
        self.focused.and_then(|id| {
            self.windows.get(&id).and_then(|w| match self.show_app_id {
                true => w.app_id.as_ref(),
                false => w.title.as_ref(),
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
        popup_config: &PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<Message> {
        button(
            text(self.trimmed_title())
                .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                .color(self.cfg_override.text_color.unwrap_or(config.text_color))
                .fill(anchor),
        )
        .padding(self.cfg_override.text_margin.unwrap_or(config.text_margin))
        .on_event_with(Message::popup::<Self>(
            self.popup_cfg_override.width.unwrap_or(popup_config.width),
            self.popup_cfg_override
                .height
                .unwrap_or(popup_config.height),
            anchor,
        ))
        .style(|_, _| Style::default())
        .into()
    }

    fn popup_view<'a>(
        &'a self,
        config: &'a PopupConfig,
        template: &Handlebars,
    ) -> Element<'a, Message> {
        container(scrollable(
            container(
                if let Some(window) = self.focused.and_then(|id| self.windows.get(&id)) {
                    let unset = String::from("Unset");
                    let window_id = window.id.to_string();
                    let workspace_id = window.workspace_id.unwrap_or_default().to_string();
                    let ctx = BTreeMap::from([
                        ("title", window.title.as_ref().unwrap_or(&unset)),
                        ("app_id", window.app_id.as_ref().unwrap_or(&unset)),
                        ("window_id", &window_id),
                        ("workspace_id", &workspace_id),
                    ]);
                    text(template.render("niri.window", &ctx).unwrap_or_default())
                } else {
                    "No window focused".into()
                }
                .color(
                    self.popup_cfg_override
                        .text_color
                        .unwrap_or(config.text_color),
                )
                .size(
                    self.popup_cfg_override
                        .font_size
                        .unwrap_or(config.font_size),
                ),
            )
            .padding(
                self.popup_cfg_override
                    .text_margin
                    .unwrap_or(config.text_margin),
            ),
        ))
        .padding(self.popup_cfg_override.padding.unwrap_or(config.padding))
        .style(|_| container::Style {
            background: Some(
                self.popup_cfg_override
                    .background
                    .unwrap_or(config.background),
            ),
            border: self.popup_cfg_override.border.unwrap_or(config.border),
            ..Default::default()
        })
        .fill_maybe(
            self.popup_cfg_override
                .fill_content_to_size
                .unwrap_or(config.fill_content_to_size),
        )
        .into()
    }

    impl_wrapper!();

    fn requires(&self) -> Vec<TypeId> {
        vec![require_listener::<NiriListener>()]
    }

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        popup_config: &HashMap<String, Option<String>>,
        templates: &mut Handlebars,
    ) {
        let default = Self::default();
        self.cfg_override = config.into();
        self.popup_cfg_override.update(popup_config);
        self.max_length = config
            .get("max_length")
            .and_then(|v| v.as_ref().and_then(|v| v.parse().ok()))
            .unwrap_or(default.max_length);
        self.show_app_id = config
            .get("show_app_id")
            .and_then(|v| v.into_bool())
            .unwrap_or(default.show_app_id);
        templates
            .register_template_string(
                "niri.window",
                popup_config
                    .get("format")
                    .unescape()
                    .unwrap_or("Title: {{title}}\nApplication ID: {{app_id}}\nWindow ID: {{window_id}}\nWorkspace ID: {{workspace_id}}".to_string()),
            )
            .unwrap_or_else(|e| eprintln!("Failed to parse battery popup format: {e}"));
    }
}
