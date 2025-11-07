use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use bar_rs_derive::Builder;
use bluer::Adapter;
use handlebars::Handlebars;
use iced::widget::button::Style;
use iced::widget::container;
use iced::{futures::SinkExt, stream, widget::text, Element, Subscription};
use tokio::{io, time::sleep};

use crate::button::button;
use crate::config::popup_config::PopupConfig;
use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    Message, NERD_FONT,
};
use crate::{impl_on_click, impl_wrapper};

use super::Module;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Device {
    icon: &'static str,
    name: String,
}

#[derive(Clone, Debug)]
struct Controller {
    is_powered: bool,
    connected_devices: HashSet<Device>,
}

impl Controller {
    async fn get_all_devices(adapter: &Adapter) -> Result<HashSet<Device>, io::Error> {
        let mut connected_devices = HashSet::new();

        let connected_devices_addresses = adapter.device_addresses().await?;
        for addr in connected_devices_addresses {
            let device = adapter.device(addr)?;

            let icon = match device.icon().await?.unwrap_or("None".to_string()).as_ref() {
                "audio-card" => "󰓃",
                "audio-input-microphone" => "",
                "audio-headphones" | "audio-headset" => "󰋋",
                "battery" => "󰂀",
                "camera-photo" => "󰻛",
                "computer" => "",
                "input-keyboard" => "󰌌",
                "input-mouse" => "󰍽",
                "input-gaming" => "󰊴",
                "phone" => "󰏲",
                "None" => "",
                _ => "",
            };
            if device.is_connected().await? {
                connected_devices.insert(Device {
                    icon,
                    name: device.alias().await?,
                });
            }
        }
        Ok(connected_devices)
    }

    async fn from_adaper(adapter: Adapter) -> Result<Controller, io::Error> {
        let is_powered = adapter.is_powered().await?;
        Ok(Controller {
            is_powered,
            connected_devices: if is_powered {
                Controller::get_all_devices(&adapter).await?
            } else {
                HashSet::default()
            },
        })
    }
}

#[derive(Debug, Builder)]
pub struct BluetoothMod {
    controllers: Vec<Controller>,
    cfg_override: ModuleConfigOverride,
}

impl Default for BluetoothMod {
    fn default() -> Self {
        Self {
            controllers: Vec::new(),
            cfg_override: Default::default(),
        }
    }
}

impl BluetoothMod {
    fn status_icon(&self) -> &'static str {
        if self.controllers.iter().any(|c| c.is_powered) {
            ""
        } else {
            ""
        }
    }
    fn connected_devices(&self) -> HashSet<&Device> {
        let mut devices = HashSet::new();
        for c in self.controllers.iter() {
            devices.extend(&c.connected_devices);
        }
        devices
    }
}

impl Module for BluetoothMod {
    fn name(&self) -> String {
        "bluetooth".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        _popup_config: &PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<Message> {
        let connected_devices = self.connected_devices();
        let (bt_icons, bt_text) = match connected_devices.len() {
            0 => (self.status_icon().to_string(), None),
            // show name if only one connected device
            1 => {
                let device = connected_devices.iter().next().unwrap();
                (device.icon.to_string(), Some(&device.name))
            }
            // show icons for connected bluetooth devices
            _ => (
                connected_devices
                    .iter()
                    .fold(String::new(), |mut acc, elem| {
                        acc.push_str(elem.icon);
                        acc
                    }),
                None,
            ),
        };
        let list = if let Some(bt_text) = bt_text {
            list![
                anchor,
                container(
                    text(bt_icons)
                        .fill(anchor)
                        .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                        .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                        .font(NERD_FONT)
                )
                .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin)),
                container(
                    text(bt_text)
                        .fill(anchor)
                        .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                        .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                )
                .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin))
            ]
        } else {
            list![
                anchor,
                container(
                    text(bt_icons)
                        .fill(anchor)
                        .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                        .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                        .font(NERD_FONT)
                )
                .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin))
            ]
        };

        button(list.spacing(self.cfg_override.spacing.unwrap_or(config.spacing)))
            .style(|_, _| Style::default())
            .into()
    }

    impl_wrapper!();

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _templates: &mut Handlebars,
    ) {
        self.cfg_override = config.into();
    }

    impl_on_click!();

    fn subscription(&self) -> Option<iced::Subscription<Message>> {
        Some(Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                if let Ok(session) = bluer::Session::new().await {
                    loop {
                        let mut controllers: Vec<Controller> = Vec::new();
                        let Ok(adapter_names) = session.adapter_names().await else {
                            return;
                        };
                        for adapter_name in adapter_names {
                            // swallow any io errors for fetch adaper informations,
                            // because it will be retred and frequently fetch in a loop
                            if let Ok(adapter) = session.adapter(&adapter_name) {
                                if let Ok(controller) = Controller::from_adaper(adapter).await {
                                    controllers.push(controller);
                                }
                            }
                        }
                        if sender
                            .send(Message::update(move |reg| {
                                let m = reg.get_module_mut::<BluetoothMod>();
                                m.controllers = controllers
                            }))
                            .await
                            .is_err()
                        {
                            return;
                        }
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            })
        }))
    }
}
