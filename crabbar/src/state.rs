use std::{collections::HashMap, sync::Arc, time::Duration};

use iced::{
    event::wayland,
    platform_specific::shell::commands::layer_surface::get_layer_surface,
    runtime::platform_specific::wayland::layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
    stream,
    window::Id,
    Element, Task,
};
use log::{error, info};
use smithay_client_toolkit::{
    output::OutputInfo, reexports::client::protocol::wl_output::WlOutput, shell::wlr_layer::Layer,
};
use tokio::time::sleep;

use crate::{config::Config, message::Message};

#[derive(Debug)]
pub struct State {
    outputs: HashMap<WlOutput, Option<OutputInfo>>,
    open: bool,
    layer_id: Id,
    config_path: String,
}

impl State {
    fn new() -> (Self, Task<Message>) {
        (
            State {
                outputs: HashMap::new(),
                open: false,
                layer_id: Id::unique(),
                // TODO!
                config_path: String::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        use Message::*;
        match msg {
            FetchSubscriptions(sx) => {
                sx.send(vec![]).unwrap();
            }
            Update(updates) => {
                for updatefn in updates {
                    Arc::into_inner(updatefn.0).unwrap()()
                }
            }
            OutputEvent { event, wl_output } => match *event {
                wayland::OutputEvent::Created(info_maybe) => {
                    self.outputs.insert(wl_output, info_maybe);
                    if !self.open {
                        self.open = true;
                        return Task::stream(stream::channel(1, |_| async {
                            sleep(Duration::from_millis(500)).await;
                        }))
                        .chain(self.open());
                    }
                }
                wayland::OutputEvent::InfoUpdate(info) => {
                    self.outputs.insert(wl_output, Some(info));
                }
                wayland::OutputEvent::Removed => {
                    self.outputs.remove(&wl_output);
                }
            },
            IpcCommand(cmd) => info!("Received ipc cmd: {cmd:?}"),
            _ => todo!(),
        }
        Task::none()
    }

    fn view(&self, _: Id) -> Element<Message> {
        "hello".into()
    }

    fn open(&self) -> Task<Message> {
        let (output, info) = self
            .outputs
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
            namespace: "crabbar".to_string(),
            output,
            // margin: (&self.config.style.margin).into(),
            id: self.layer_id,
            ..Default::default()
        })
    }
}
