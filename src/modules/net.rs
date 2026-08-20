use std::collections::HashMap;

use bar_rs_derive::Builder;
use iced::{Element, widget::text};

use crate::{
    Message, NERD_FONT,
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    impl_on_click, impl_wrapper,
    listeners::net::NetListener,
};

use super::{Module, require_listener};

#[derive(Debug, Default)]
pub struct NetState {
    pub speed: f32,
    pub total: u64,
    pub icon: String,
}

fn net_view<'a>(
    state: &'a NetState,
    cfg_override: &ModuleConfigOverride,
    config: &LocalModuleConfig,
    anchor: &BarAnchor,
) -> Element<'a, Message> {
    list![
        anchor,
        iced::widget::container(
            text!("{}", state.icon)
                .fill(anchor)
                .size(cfg_override.icon_size.unwrap_or(config.icon_size))
                .color(cfg_override.icon_color.unwrap_or(config.icon_color))
                .font(NERD_FONT)
        )
        .padding(cfg_override.icon_margin.unwrap_or(config.icon_margin)),
        iced::widget::container(
            text!("{}", format_speed(state.speed))
                .fill(anchor)
                .size(cfg_override.font_size.unwrap_or(config.font_size))
                .color(cfg_override.text_color.unwrap_or(config.text_color))
        )
        .padding(cfg_override.text_margin.unwrap_or(config.text_margin)),
    ]
    .spacing(cfg_override.spacing.unwrap_or(config.spacing))
    .into()
}

fn read_config(
    state: &mut NetState,
    cfg_override: &mut ModuleConfigOverride,
    config: &HashMap<String, Option<String>>,
) {
    *cfg_override = config.into();
    state.icon = config
        .get("icon")
        .and_then(|v| v.clone())
        .unwrap_or_else(|| state.icon.clone());
}

/// Format a byte rate, e.g. `1.5 MB/s`.
fn format_speed(speed: f32) -> String {
    let (value, unit) = if speed >= 1024. * 1024. {
        (speed / (1024. * 1024.), "MB/s")
    } else if speed >= 1024. {
        (speed / 1024., "KB/s")
    } else {
        (speed, "B/s")
    };
    format!("{value:.1} {unit}")
}

#[derive(Debug, Builder)]
pub struct NetUploadMod {
    pub state: NetState,
    cfg_override: ModuleConfigOverride,
}

impl Default for NetUploadMod {
    fn default() -> Self {
        Self {
            state: NetState {
                icon: "󰕒".to_string(),
                ..Default::default()
            },
            cfg_override: Default::default(),
        }
    }
}

impl Module for NetUploadMod {
    fn name(&self) -> String {
        "net.upload".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        _popup_config: &crate::config::popup_config::PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &handlebars::Handlebars,
    ) -> Element<'_, Message> {
        net_view(&self.state, &self.cfg_override, config, anchor)
    }

    impl_wrapper!();

    fn requires(&self) -> Vec<std::any::TypeId> {
        vec![require_listener::<NetListener>()]
    }

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _templates: &mut handlebars::Handlebars,
    ) {
        read_config(&mut self.state, &mut self.cfg_override, config);
    }

    impl_on_click!();
}

#[derive(Debug, Builder)]
pub struct NetDownloadMod {
    pub state: NetState,
    cfg_override: ModuleConfigOverride,
}

impl Default for NetDownloadMod {
    fn default() -> Self {
        Self {
            state: NetState {
                icon: "󰇚".to_string(),
                ..Default::default()
            },
            cfg_override: Default::default(),
        }
    }
}

impl Module for NetDownloadMod {
    fn name(&self) -> String {
        "net.download".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        _popup_config: &crate::config::popup_config::PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &handlebars::Handlebars,
    ) -> Element<'_, Message> {
        net_view(&self.state, &self.cfg_override, config, anchor)
    }

    impl_wrapper!();

    fn requires(&self) -> Vec<std::any::TypeId> {
        vec![require_listener::<NetListener>()]
    }

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _templates: &mut handlebars::Handlebars,
    ) {
        read_config(&mut self.state, &mut self.cfg_override, config);
    }

    impl_on_click!();
}
