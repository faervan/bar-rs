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

const MAX_LENGTH: usize = 35;
const MAX_TITLE_LENGTH: usize = 20;

#[derive(Default, Debug, Builder)]
pub struct MediaMod {
    title: String,
    artist: Option<String>,
    cfg_override: ModuleConfigOverride,
}

impl Module for MediaMod {
    fn id(&self) -> String {
        "media".to_string()
    }

    fn view(&self, config: &LocalModuleConfig) -> iced::Element<Message> {
        row![
            text!("")
                .center()
                .height(Fill)
                .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                .font(NERD_FONT),
            text![
                "{}{}",
                self.title,
                self.artist
                    .as_ref()
                    .map(|name| format!(" - {name}"))
                    .unwrap_or("".to_string())
            ]
            .center()
            .height(Fill)
            .size(self.cfg_override.font_size.unwrap_or(config.font_size))
            .color(self.cfg_override.text_color.unwrap_or(config.text_color)),
        ]
        .spacing(15)
        .into()
    }

    fn read_config(&mut self, config: &HashMap<String, Option<String>>) {
        self.cfg_override = config.into();
    }

    fn subscription(&self) -> Option<iced::Subscription<Message>> {
        Some(Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let mut child = Command::new("sh")
                    .arg("-c")
                    .arg("playerctl --follow metadata --format '{{title}}¾{{artist}}'")
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Failed to read output from playerctl");

                let stdout = child
                    .stdout
                    .take()
                    .expect("child did not have a handle to stdout");

                let mut reader = BufReader::new(stdout).lines();

                while let Some(line) = reader.next_line().await.unwrap() {
                    if let Some((mut title, artist)) = line.split_once('¾') {
                        title = title.trim();
                        if !title.is_empty() {
                            let mut title = title.to_string();
                            let mut artist = artist.to_string();
                            if title.len() + artist.len() + 3 > MAX_LENGTH {
                                if title.len() > MAX_TITLE_LENGTH {
                                    title.truncate(MAX_TITLE_LENGTH - 3);
                                    title.push_str("...");
                                }
                                if title.len() + artist.len() + 3 > MAX_LENGTH {
                                    artist.truncate(MAX_LENGTH - MAX_TITLE_LENGTH - 6);
                                    artist.push_str("...");
                                }
                            }
                            sender
                                .send(Message::update(Box::new(move |reg| {
                                    let media = reg.get_module_mut::<MediaMod>();
                                    media.title = title;
                                    media.artist = match artist.as_str() == "" {
                                        true => None,
                                        false => Some(artist),
                                    };
                                })))
                                .await
                                .unwrap_or_else(|err| {
                                    eprintln!("Trying to send cpu_usage failed with err: {err}");
                                });
                        }
                    }
                }
            })
        }))
    }
}
