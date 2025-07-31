use std::collections::HashMap;

use clap::Subcommand;
use iced::{
    Element, Task,
    platform_specific::shell::commands::layer_surface::get_layer_surface,
    runtime::platform_specific::wayland::layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
    window::Id,
};
use serde::{Deserialize, Serialize};
use smithay_client_toolkit::{
    output::OutputInfo, reexports::client::protocol::wl_output::WlOutput, shell::wlr_layer::Layer,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Window {
    naive_id: usize,
    #[serde(skip, default = "id_default")]
    window_id: Id,
}

fn id_default() -> Id {
    Id::NONE
}

#[derive(Subcommand, Debug, Deserialize, Serialize)]
pub enum WindowCommand {}

impl Window {
    pub fn new(id: usize) -> Self {
        Self {
            naive_id: id,
            window_id: Id::unique(),
        }
    }

    pub fn view<Message>(&self) -> Element<Message> {
        iced::widget::text!("This is window {}", self.naive_id).into()
    }

    pub fn handle_ipc<Message>(&mut self, cmd: WindowCommand) -> Task<Message> {
        use WindowCommand::*;
        match cmd {}
    }

    pub fn open<Message>(&self, outputs: &HashMap<WlOutput, Option<OutputInfo>>) -> Task<Message> {
        log::info!("opening window with id {}", self.naive_id);
        let (output, info) = outputs
            .iter()
            .find(|(_, info)| {
                true
                // info.as_ref()
                //     .is_some_and(|info| info.name == self.config.monitor)
            })
            .map(|(o, info)| (IcedOutput::Output(o.clone()), info))
            .unwrap_or_else(|| {
                // if let Some(m) = self.config.monitor.as_ref() {
                //     error!("No output with name {m} could be found!");
                // }
                (IcedOutput::Active, &None)
            });

        let (x, y) = info
            .as_ref()
            .and_then(|i| i.logical_size.map(|(x, y)| (x as u32, y as u32)))
            .unwrap_or((1920, 1080));

        get_layer_surface(SctkLayerSurfaceSettings {
            layer: Layer::Top,
            // keyboard_interactivity: (&self.config.kb_focus).into(),
            // anchor: (&self.config.style.anchor).into(),
            // exclusive_zone: self.config.exclusive_zone(),
            // size: self.config.dimension(x, y),
            size: Some((Some(1920), Some(30))),
            // size: Some((None, Some(50))),
            anchor: smithay_client_toolkit::shell::wlr_layer::Anchor::TOP,
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
}
