use std::{any::TypeId, collections::HashMap};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::{
    widget::{container, rich_text, span},
    Background, Border, Color, Element, Padding,
};
use niri_ipc::Workspace;

use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
        parse::StringExt,
    },
    fill::FillExt,
    impl_wrapper, list,
    listeners::niri::NiriListener,
    modules::{require_listener, Module},
    Message, NERD_FONT,
};

#[derive(Debug, Builder)]
pub struct NiriWorkspaceMod {
    pub workspaces: HashMap<String, Vec<Workspace>>,
    pub focused: u64,
    cfg_override: ModuleConfigOverride,
    icon_padding: Padding,
    icon_background: Option<Background>,
    icon_border: Border,
    active_padding: Option<Padding>,
    active_size: f32,
    active_color: Color,
    active_background: Option<Background>,
    active_icon_border: Border,
    // Output, (idx, icon)
    icons: HashMap<String, HashMap<u8, String>>,
    fallback_icon: String,
    active_fallback_icon: String,
    output_order: Vec<String>,
}

impl Default for NiriWorkspaceMod {
    fn default() -> Self {
        Self {
            workspaces: HashMap::new(),
            focused: 0,
            cfg_override: Default::default(),
            icon_padding: Padding::default(),
            icon_background: None,
            icon_border: Border::default(),
            active_padding: None,
            active_size: 20.,
            active_color: Color::WHITE,
            active_background: None,
            active_icon_border: Border::default().rounded(8),
            icons: HashMap::new(),
            fallback_icon: String::from(""),
            active_fallback_icon: String::from(""),
            output_order: vec![],
        }
    }
}

impl NiriWorkspaceMod {
    fn sort_by_outputs<'a, F, I>(&'a self, f: F) -> Vec<Element<'a, Message>>
    where
        F: Fn((&'a String, &'a Vec<Workspace>)) -> I,
        I: Iterator<Item = Element<'a, Message>>,
    {
        match self.output_order.is_empty() {
            true => self
                .workspaces
                .iter()
                .flat_map(f)
                .collect::<Vec<Element<Message>>>(),
            false => self
                .output_order
                .iter()
                .filter_map(|o| self.workspaces.get_key_value(o))
                .flat_map(f)
                .collect::<Vec<Element<Message>>>(),
        }
    }
}

impl Module for NiriWorkspaceMod {
    fn name(&self) -> String {
        "niri.workspaces".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<Message> {
        list(
            anchor,
            self.sort_by_outputs(|(output, workspaces)| {
                workspaces.iter().map(|ws| {
                    let mut span = span(
                        self.icons
                            .get(&output.to_lowercase())
                            .and_then(|icons| icons.get(&ws.idx))
                            .unwrap_or(match ws.id == self.focused {
                                true => &self.active_fallback_icon,
                                false => &self.fallback_icon,
                            }),
                    )
                    .padding(self.icon_padding)
                    .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                    .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                    .background_maybe(self.icon_background)
                    .border(self.icon_border)
                    .font(NERD_FONT);
                    if ws.id == self.focused {
                        span = span
                            .padding(self.active_padding.unwrap_or(self.icon_padding))
                            .size(self.active_size)
                            .color(self.active_color)
                            .background_maybe(self.active_background)
                            .border(self.active_icon_border);
                    }
                    container(rich_text![span].fill(anchor))
                        .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin))
                        .into()
                })
            }),
        )
        .padding(self.cfg_override.padding.unwrap_or(config.padding))
        .spacing(self.cfg_override.spacing.unwrap_or(config.spacing))
        .into()
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
        self.icon_padding = config
            .get("icon_padding")
            .and_then(|v| v.into_insets().map(|i| i.into()))
            .unwrap_or(default.icon_padding);
        self.icon_background = config
            .get("icon_background")
            .map(|v| v.into_background())
            .unwrap_or(default.icon_background);
        self.icon_border = {
            let color = config.get("icon_border_color").and_then(|s| s.into_color());
            let width = config.get("icon_border_width").and_then(|s| s.into_float());
            let radius = config
                .get("icon_border_radius")
                .and_then(|s| s.into_insets().map(|i| i.into()));
            if color.is_some() || width.is_some() || radius.is_some() {
                Border {
                    color: color.unwrap_or_default(),
                    width: width.unwrap_or(1.),
                    radius: radius.unwrap_or(8_f32.into()),
                }
            } else {
                default.active_icon_border
            }
        };
        self.active_padding = config
            .get("active_padding")
            .map(|v| v.into_insets().map(|i| i.into()))
            .unwrap_or(default.active_padding);
        self.active_size = config
            .get("active_size")
            .and_then(|v| v.into_float())
            .unwrap_or(default.active_size);
        self.active_color = config
            .get("active_color")
            .and_then(|v| v.into_color())
            .unwrap_or(default.active_color);
        self.active_background = config
            .get("active_background")
            .map(|v| v.into_background())
            .unwrap_or(default.active_background);
        self.active_icon_border = {
            let color = config
                .get("active_border_color")
                .and_then(|s| s.into_color());
            let width = config
                .get("active_border_width")
                .and_then(|s| s.into_float());
            let radius = config
                .get("active_border_radius")
                .and_then(|s| s.into_insets().map(|i| i.into()));
            if color.is_some() || width.is_some() || radius.is_some() {
                Border {
                    color: color.unwrap_or_default(),
                    width: width.unwrap_or(1.),
                    radius: radius.unwrap_or(8_f32.into()),
                }
            } else {
                default.active_icon_border
            }
        };
        self.fallback_icon = config
            .get("fallback_icon")
            .and_then(|v| v.clone())
            .unwrap_or(default.fallback_icon);
        self.active_fallback_icon = config
            .get("active_fallback_icon")
            .and_then(|v| v.clone())
            .unwrap_or(default.active_fallback_icon);
        self.output_order = config
            .get("output_order")
            .and_then(|v| v.clone())
            .map(|v| v.split(',').map(|v| v.trim().to_string()).collect())
            .unwrap_or(default.output_order);
        config.iter().for_each(|(key, val)| {
            let Some(val) = val.clone() else {
                return;
            };
            if let [output, idx] = key.split(':').map(|i| i.trim()).collect::<Vec<&str>>()[..] {
                if let Ok(idx) = idx.parse() {
                    match self.icons.get_mut(output) {
                        Some(icons) => {
                            icons.insert(idx, val);
                        }
                        None => {
                            self.icons
                                .insert(output.to_string(), HashMap::from([(idx, val)]));
                        }
                    }
                }
            }
        });
    }
}
