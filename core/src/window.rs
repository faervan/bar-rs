use std::collections::HashMap;

use clap::{Args, Subcommand};
use iced::{
    platform_specific::shell::commands::layer_surface::get_layer_surface,
    runtime::platform_specific::wayland::layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
    window::Id,
    Element, Task,
};
use serde::{Deserialize, Serialize};
use smithay_client_toolkit::{
    output::OutputInfo, reexports::client::protocol::wl_output::WlOutput, shell::wlr_layer::Layer,
};

use crate::config::{
    theme::{Theme, ThemeOverride},
    window::MonitorSelection,
    ConfigOptionOverride, ConfigOptions,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Window {
    naive_id: usize,
    #[serde(skip, default = "id_default")]
    window_id: Id,
    open_options: WindowOpenOptions,
    config: ConfigOptions,
    theme: Theme,
}

#[derive(Args, Debug, Clone, Deserialize, Serialize)]
pub struct WindowOpenOptions {
    #[arg(default_value = "crabbar")]
    /// Name of the configuration to use
    pub name: String,

    #[command(flatten)]
    pub config: ConfigOptionOverride,

    #[command(flatten)]
    pub theme: ThemeOverride,
}

fn id_default() -> Id {
    Id::NONE
}

#[derive(Subcommand, Debug, Deserialize, Serialize)]
pub enum WindowCommand {}

impl Window {
    pub fn new(
        id: usize,
        open_options: WindowOpenOptions,
        config: ConfigOptions,
        theme: Theme,
    ) -> Self {
        Self {
            naive_id: id,
            window_id: Id::unique(),
            open_options,
            config,
            theme,
        }
    }

    pub fn view<'a, Message: 'a>(&'a self) -> Element<'a, Message> {
        iced::widget::text!("This is window {}", self.naive_id).into()
        // iced::widget::container(iced::widget::text("This is window 󰈹").color(iced::color!(0x0f0)))
        //     .style(|_| iced::widget::container::Style {
        //         icon_color: Some(iced::color!(0xf00)),
        //         text_color: Some(iced::color!(0x00f)),
        //         ..Default::default()
        //     })
        //     .into()
    }

    pub fn handle_ipc<Message>(&mut self, cmd: WindowCommand) -> Task<Message> {
        use WindowCommand::*;
        match cmd {}
    }

    pub fn open<Message>(&self, outputs: &HashMap<WlOutput, Option<OutputInfo>>) -> Task<Message> {
        log::info!("opening window with id {}", self.naive_id);
        log::info!("outputs: {outputs:#?}");
        let (output, info) = match &self.config.window.monitor {
            MonitorSelection::All => (IcedOutput::All, None),
            MonitorSelection::Active => (IcedOutput::Active, None),
            MonitorSelection::Name(name) => outputs
                .iter()
                .find(|(_, info)| {
                    info.as_ref()
                        .is_some_and(|info| info.name.as_ref() == Some(name))
                })
                .map(|(o, info)| (IcedOutput::Output(o.clone()), info.as_ref()))
                .unwrap_or_else(|| {
                    log::error!("No output with name {name} could be found!");
                    (IcedOutput::Active, None)
                }),
        };

        let (x, y) = info
            .as_ref()
            .and_then(|i| i.logical_size.map(|(x, y)| (x as u32, y as u32)))
            .unwrap_or((1920, 1080));

        use toml_example::TomlExample;
        log::debug!("default config:\n{}", ConfigOptions::toml_example());

        get_layer_surface(SctkLayerSurfaceSettings {
            layer: Layer::Top,
            keyboard_interactivity: self.config.window.keyboard_focus,
            // exclusive_zone: self.config.exclusive_zone(),
            // size: self.config.dimension(x, y),
            size: Some((
                Some(self.config.window.width),
                Some(self.config.window.height),
            )),
            anchor: self.config.window.anchor,
            namespace: format!("crabbar{}", self.naive_id),
            exclusive_zone: 30,
            output,
            // margin: (&self.config.style.margin).into(),
            id: self.window_id,
            ..Default::default()
        })
    }

    pub fn window_id(&self) -> Id {
        self.window_id
    }

    pub fn naive_id(&self) -> usize {
        self.naive_id
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::custom(self.config.theme.clone(), (&self.theme).into())
    }
}
