use std::{any::TypeId, collections::HashMap, time::Duration};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use hyprland::{
    data::{Workspace, Workspaces},
    shared::{HyprData, HyprDataActive, HyprDataVec},
};
use iced::{
    widget::{container, rich_text, span},
    Background, Border, Color, Element, Padding,
};
use tokio::time::sleep;

use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
        parse::StringExt,
        popup_config::PopupConfig,
    },
    fill::FillExt,
    impl_on_click, impl_wrapper,
    list::list,
    listeners::hyprland::HyprListener,
    modules::{require_listener, Module},
    Message, NERD_FONT,
};

#[derive(Debug, Builder)]
pub struct HyprWorkspaceMod {
    pub active: usize,
    // (Name, Fullscreen state)
    pub open: Vec<(String, bool)>,
    cfg_override: ModuleConfigOverride,
    icon_padding: Padding,
    icon_background: Option<Background>,
    icon_border: Border,
    active_padding: Option<Padding>,
    active_size: f32,
    active_color: Color,
    active_background: Option<Background>,
    active_icon_border: Border,
}

impl Default for HyprWorkspaceMod {
    fn default() -> Self {
        Self {
            active: 0,
            open: vec![],
            cfg_override: ModuleConfigOverride::default(),
            icon_padding: Padding::default(),
            icon_background: None,
            icon_border: Border::default(),
            active_padding: None,
            active_size: 20.,
            active_color: Color::WHITE,
            active_background: None,
            active_icon_border: Border::default().rounded(8),
        }
    }
}

impl Module for HyprWorkspaceMod {
    fn name(&self) -> String {
        "hyprland.workspaces".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        _popup_config: &PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<Message> {
        list(
            anchor,
            self.open.iter().enumerate().map(|(id, (ws, _))| {
                let mut span = span(ws)
                    .padding(self.icon_padding)
                    .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                    .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                    .background_maybe(self.icon_background)
                    .border(self.icon_border)
                    .font(NERD_FONT);
                if id == self.active {
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
            }),
        )
        .padding(self.cfg_override.padding.unwrap_or(config.padding))
        .spacing(self.cfg_override.spacing.unwrap_or(config.spacing))
        .into()
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
    }

    impl_on_click!();
}

pub async fn get_workspaces(active: Option<i32>) -> (usize, Vec<(String, bool)>) {
    // Sleep a bit, to reduce the probability that a nonexisting ws is still reported active
    sleep(Duration::from_millis(10)).await;
    let Ok(workspaces) = Workspaces::get_async().await else {
        eprintln!("[hyprland.workspaces] Failed to get Workspaces!");
        return (0, vec![]);
    };
    let mut open = workspaces.to_vec();
    open.sort_by(|a, b| a.id.cmp(&b.id));
    (
        open.iter()
            .position(|ws| {
                ws.id
                    == active
                        .unwrap_or_else(|| Workspace::get_active().map(|ws| ws.id).unwrap_or(0))
            })
            .unwrap_or(0),
        open.into_iter()
            .map(|ws| (ws.name, ws.fullscreen))
            .collect(),
    )
}
