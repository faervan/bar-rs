use std::{any::TypeId, collections::HashMap};

use bar_rs_derive::Builder;
use iced::{
    widget::{rich_text, span},
    Background, Border, Color, Padding,
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
            self.workspaces
                .iter()
                .map(|(output, workspaces)| {
                    workspaces.iter().map(|ws| {
                        let mut span = span(
                            self.icons
                                .get(output)
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
                })
                .flatten(),
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
        self.cfg_override = config.into();
        if let Some(color) = config.get("active_color").and_then(|v| v.into_color()) {
            self.active_color = color;
        }
        if let Some(color) = config.get("active_background").and_then(|v| v.into_color()) {
            self.active_background = color;
        }
        if let Some(icon) = config.get("fallback_icon").and_then(|v| v.clone()) {
            self.fallback_icon = icon;
        }
        println!("reading config");
        config.iter().for_each(|(key, val)| {
            println!("key: {key:?}, val: {val:?}");
            let Some(val) = val.as_ref().map(|v| v.clone()) else {
                return;
            };
            println!("1");
            if let [output, idx] = key.split(':').map(|i| i.trim()).collect::<Vec<&str>>()[..] {
                println!("2");
                if let Ok(idx) = idx.parse() {
                    println!("3");
                    match self.icons.get_mut(output) {
                        Some(icons) => {
                            println!("4a");
                            icons.insert(idx, val);
                        }
                        None => {
                            println!("4b");
                            self.icons
                                .insert(output.to_string(), HashMap::from([(idx, val)]));
                        }
                    }
                }
            }
        });
        println!("icons: {:#?}", self.icons);
    }
}
