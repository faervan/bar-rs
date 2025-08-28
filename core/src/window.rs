use std::collections::HashMap;

use clap::{Args, Subcommand};
use iced::{
    platform_specific::shell::commands::layer_surface::{destroy_layer_surface, get_layer_surface},
    runtime::platform_specific::wayland::layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
    widget::Row,
    window::Id,
    Element, Task,
};
use log::info;
use serde::{Deserialize, Serialize};
use smithay_client_toolkit::{
    output::OutputInfo, reexports::client::protocol::wl_output::WlOutput, shell::wlr_layer::Layer,
};

use crate::{
    config::{
        style::{ContainerStyle, ContainerStyleOverride},
        theme::{Theme, ThemeOverride},
        window::MonitorSelection,
        ConfigOptionOverride, ConfigOptions,
    },
    helpers::task_constructor::TaskConstructor,
    ipc::WindowResponse,
    message::Message,
    registry::Registry,
    state::State,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Window {
    naive_id: usize,
    #[serde(skip, default = "id_default")]
    window_id: Id,
    runtime_options: WindowRuntimeOptions,
    config: ConfigOptions,
    theme: Theme,
    style: ContainerStyle,
}

#[derive(Args, Debug, Clone, Deserialize, Serialize)]
/// Configurations options that apply only to this specific window instance. They are applied by
/// CLI arguments at window creation and can be changed at runtime through the IPC.
pub struct WindowRuntimeOptions {
    #[arg(default_value = "crabbar")]
    /// Name of the configuration to use
    pub name: String,

    #[command(flatten)]
    pub config: ConfigOptionOverride,

    #[command(flatten)]
    pub theme: ThemeOverride,

    #[command(flatten)]
    pub style: ContainerStyleOverride,
}

impl WindowRuntimeOptions {
    pub fn new(name: String) -> Self {
        Self {
            name,
            config: ConfigOptionOverride::default(),
            theme: ThemeOverride::default(),
            style: ContainerStyleOverride::default(),
        }
    }
}

fn id_default() -> Id {
    Id::NONE
}

#[derive(Subcommand, Debug, Deserialize, Serialize)]
pub enum WindowCommand {
    /// Print the current configuration
    GetConfig,
    /// Print the current theme variables
    GetTheme,
    /// Print the current styles
    GetStyle,
    /// Override configuration settings
    SetConfig {
        #[arg(short, long)]
        /// Reopen the window to ensure all settings are applied
        reopen: bool,
        #[command(flatten)]
        cfg: ConfigOptionOverride,
    },
    /// Override theme variables
    SetTheme(ThemeOverride),
    /// Override style settings
    SetStyle(ContainerStyleOverride),
}

impl Window {
    pub fn new(
        id: usize,
        runtime_options: WindowRuntimeOptions,
        config: ConfigOptions,
        theme: Theme,
        style: ContainerStyle,
    ) -> Self {
        Self {
            naive_id: id,
            window_id: Id::unique(),
            runtime_options,
            config,
            theme,
            style,
        }
    }

    pub fn view<'a>(&'a self, registry: &'a Registry) -> Element<'a, Message> {
        registry
            .get_modules(self.config.modules.all())
            .fold(Row::new(), |row, (variant_name, module)| {
                row.push(module.view(variant_name, &self.config.window.anchor, &HashMap::new()))
            })
            .into()
        // iced::widget::text!("This is window {}", self.naive_id).into()
        // iced::widget::container(iced::widget::text("This is window 󰈹").color(iced::color!(0x0f0)))
        //     .style(|_| iced::widget::container::Style {
        //         icon_color: Some(iced::color!(0xf00)),
        //         text_color: Some(iced::color!(0x00f)),
        //         ..Default::default()
        //     })
        //     .into()
    }

    pub fn handle_ipc(&mut self, cmd: WindowCommand) -> (WindowResponse, TaskConstructor<State>) {
        use WindowCommand::*;
        let mut task = TaskConstructor::new();
        let response = match cmd {
            GetConfig => WindowResponse::Config(self.config.clone()),
            GetTheme => WindowResponse::Theme(self.theme.clone()),
            GetStyle => WindowResponse::Style(self.style.clone()),
            SetConfig { reopen, cfg } => {
                if reopen {
                    let window_id = self.window_id;
                    task.chain(move |state: &State| state.reopen_window(&window_id));
                }
                self.config.merge_opt(cfg);
                WindowResponse::ConfigApplied
            }
            SetTheme(theme) => {
                self.theme.merge_opt(theme);
                WindowResponse::ThemeApplied
            }
            SetStyle(style) => {
                self.style.merge_opt(style);
                WindowResponse::StyleApplied
            }
        };
        (response, task)
    }

    pub fn open(&self, outputs: &HashMap<WlOutput, Option<OutputInfo>>) -> Task<Message> {
        info!("opening window with id {}", self.naive_id);
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

    pub fn reopen(&self, outputs: &HashMap<WlOutput, Option<OutputInfo>>) -> Task<Message> {
        info!("Reopening window with id {}", self.naive_id);
        destroy_layer_surface(self.window_id).chain(self.open(outputs))
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
