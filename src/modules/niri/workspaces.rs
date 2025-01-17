use std::{any::TypeId, collections::HashMap};

use bar_rs_derive::Builder;
use iced::{
    widget::{rich_text, span},
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
    list,
    listeners::niri::NiriListener,
    modules::{require_listener, Module},
    Message, NERD_FONT,
};

#[derive(Debug, Builder)]
pub struct NiriWorkspaceMod {
    pub workspaces: HashMap<String, Vec<Workspace>>,
    pub focused: u64,
    cfg_override: ModuleConfigOverride,
    active_color: Color,
    active_background: Color,
    // Output, (idx, icon)
    icons: HashMap<String, HashMap<u8, String>>,
    fallback_icon: String,
    output_order: Vec<String>,
}

impl Default for NiriWorkspaceMod {
    fn default() -> Self {
        Self {
            workspaces: HashMap::new(),
            focused: 0,
            cfg_override: Default::default(),
            active_color: Color::BLACK,
            active_background: Color::WHITE.scale_alpha(0.5),
            icons: HashMap::new(),
            fallback_icon: String::new(),
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
                .filter_map(|o| self.workspaces.get(o).map(|ws| (o, ws)))
                .flat_map(f)
                .collect::<Vec<Element<Message>>>(),
        }
    }
}

impl Module for NiriWorkspaceMod {
    fn name(&self) -> String {
        "niri.workspaces".to_string()
    }

    fn view(&self, config: &LocalModuleConfig, anchor: &BarAnchor) -> iced::Element<Message> {
        list(
            anchor,
            self.sort_by_outputs(|(output, workspaces)| {
                workspaces.iter().map(|ws| {
                    let mut span = span(
                        self.icons
                            .get(&output.to_lowercase())
                            .and_then(|icons| icons.get(&ws.idx))
                            .unwrap_or(&self.fallback_icon),
                    )
                    .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                    .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                    .padding(Padding {
                        top: 0.,
                        bottom: 0.,
                        right: 12.,
                        left: 5.,
                    })
                    .font(NERD_FONT);
                    if ws.id == self.focused {
                        span = span
                            .background(Background::Color(self.active_background))
                            .border(Border::default().rounded(8))
                            .color(self.active_color);
                    }
                    rich_text![span].fill(anchor).into()
                })
            }),
        )
        .padding(Padding {
            top: 0.,
            bottom: 0.,
            right: 12.,
            left: 5.,
        })
        .spacing(self.cfg_override.spacing.unwrap_or(config.spacing))
        .into()
    }

    fn requires(&self) -> Vec<TypeId> {
        vec![require_listener::<NiriListener>()]
    }

    fn read_config(&mut self, config: &HashMap<String, Option<String>>) {
        let default = Self::default();
        self.cfg_override = config.into();
        self.active_color = config
            .get("active_color")
            .and_then(|v| v.into_color())
            .unwrap_or(default.active_color);
        self.active_background = config
            .get("active_background")
            .and_then(|v| v.into_color())
            .unwrap_or(default.active_background);
        self.fallback_icon = config
            .get("fallback_icon")
            .and_then(|v| v.clone())
            .unwrap_or(default.fallback_icon);
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
