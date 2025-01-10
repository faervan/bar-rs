use std::{collections::HashMap, process::Stdio};

use bar_rs_derive::Builder;
use iced::{
    futures::SinkExt,
    stream,
    widget::{row, text},
    Length::Fill,
    Subscription,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use crate::{
    config::module_config::{LocalModuleConfig, ModuleConfigOverride},
    Message, NERD_FONT,
};

use super::Module;

#[derive(Default, Debug, Builder)]
pub struct VolumeMod {
    level: u16,
    icon: &'static str,
    cfg_override: ModuleConfigOverride,
}

impl Module for VolumeMod {
    fn id(&self) -> String {
        "volume".to_string()
    }

    fn view(&self, config: &LocalModuleConfig) -> iced::Element<Message> {
        row![
            text!("{}", self.icon)
                .center()
                .height(Fill)
                .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                .font(NERD_FONT),
            text!["{}%", self.level,]
                .center()
                .height(Fill)
                .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                .color(self.cfg_override.text_color.unwrap_or(config.text_color)),
        ]
        .spacing(10)
        .into()
    }

    fn read_config(&mut self, config: &HashMap<String, Option<String>>) {
        self.cfg_override = config.into();
    }

    fn subscription(&self) -> Option<iced::Subscription<Message>> {
        Some(Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let volume = || {
                    Message::update(move |reg| {
                        let vmod = reg.get_module_mut::<VolumeMod>();
                        let volume = get_volume();
                        vmod.level = volume.0;
                        vmod.icon = volume.1;
                    })
                };

                sender.send(volume()).await.unwrap_or_else(|err| {
                    eprintln!("Trying to send volume failed with err: {err}");
                });

                let mut child = Command::new("sh")
                    .arg("-c")
                    .arg("pactl subscribe")
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Failed to spawn pactl to monitor volume changes");

                let stdout = child
                    .stdout
                    .take()
                    .expect("child did not have a handle to stdout");

                let mut reader = BufReader::new(stdout).lines();

                while let Some(line) = reader.next_line().await.unwrap() {
                    if line.contains("'change' on sink") {
                        sender.send(volume()).await.unwrap_or_else(|err| {
                            eprintln!("Trying to send volume failed with err: {err}");
                        });
                    }
                }
            })
        }))
    }
}

fn get_volume() -> (u16, &'static str) {
    let volume = String::from_utf8(
        std::process::Command::new("sh")
            .arg("-c")
            .arg("wpctl get-volume @DEFAULT_AUDIO_SINK@")
            .output()
            .expect("Couldn't get volume from wpctl")
            .stdout,
    )
    .expect("Couldn't convert output from wpctl to String");
    let mut volume = volume
        .as_str()
        .strip_prefix("Volume: ")
        .unwrap_or_else(|| {
            eprintln!(
                "Failed to get volume from wpctl, tried: `wpctl get-volume @DEFAULT_AUDIO_SINK@`"
            );
            "0"
        })
        .trim();
    let mut muted = false;
    if let Some(x) = volume.strip_suffix(" [MUTED]") {
        volume = x;
        muted = true;
    }
    let volume = volume.parse::<f32>().unwrap();
    let volume = (volume * 100.) as u16;
    (
        volume,
        match muted {
            true => "󰖁",
            false => match volume {
                n if n >= 50 => "󰕾",
                n if n >= 25 => "󰖀",
                _ => "󰕿",
            },
        },
    )
}
