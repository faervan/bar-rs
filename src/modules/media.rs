use std::{collections::HashMap, process::Stdio};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::widget::{container, Text};
use iced::{futures::SinkExt, stream, widget::text, Element, Subscription};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use crate::impl_wrapper;
use crate::tooltip::ElementExt;
use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    Message, NERD_FONT,
};

use super::Module;

#[derive(Debug, Builder)]
pub struct MediaMod {
    title: String,
    artist: Option<String>,
    cfg_override: ModuleConfigOverride,
    icon: String,
    max_length: usize,
    max_title_length: usize,
}

impl Default for MediaMod {
    fn default() -> Self {
        Self {
            title: Default::default(),
            artist: None,
            cfg_override: Default::default(),
            icon: String::from(""),
            max_length: 35,
            max_title_length: 20,
        }
    }
}

impl MediaMod {
    fn update(&mut self, title: String, artist: String) {
        self.title = title;
        self.artist = match artist.as_str() == "" {
            true => None,
            false => Some(artist),
        };
    }

    fn get_active_trimmed(&self) -> String {
        let mut title = self.title.clone();
        let mut artist = self.artist.clone();
        let artist_len = artist.as_ref().map(|a| a.len()).unwrap_or(0);
        if title.len() + artist_len + 3 > self.max_length {
            if title.len() > self.max_title_length {
                title = title.chars().take(self.max_length - 3).collect();
                title.push_str("...");
            }
            if title.len() + artist_len + 3 > self.max_length {
                artist = artist.map(|a| {
                    let mut a: String = a
                        .chars()
                        .take(self.max_length - self.max_title_length - 6)
                        .collect();
                    a.push_str("...");
                    a
                });
            }
        }
        match artist {
            Some(artist) => format!("{title} - {artist}"),
            None => title,
        }
    }

    fn is_overlength(&self) -> bool {
        self.title.len() + self.artist.as_ref().map(|a| a.len()).unwrap_or(0) + 3 > self.max_length
    }

    fn get_active(&self) -> Text {
        text!(
            "{}{}",
            self.title,
            self.artist
                .as_ref()
                .map(|a| format!(" - {a}"))
                .unwrap_or_default()
        )
    }
}

impl Module for MediaMod {
    fn name(&self) -> String {
        "media".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<Message> {
        list![
            anchor,
            container(
                text(&self.icon)
                    .fill(anchor)
                    .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                    .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                    .font(NERD_FONT)
            )
            .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin)),
            container(
                text(self.get_active_trimmed())
                    .fill(anchor)
                    .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                    .color(self.cfg_override.text_color.unwrap_or(config.text_color))
            )
            .padding(self.cfg_override.text_margin.unwrap_or(config.text_margin))
            .tooltip_maybe(self.is_overlength().then_some(self.get_active().size(12))),
        ]
        .spacing(self.cfg_override.spacing.unwrap_or(config.spacing))
        .into()
    }

    impl_wrapper!();

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _templates: &mut Handlebars,
    ) {
        let default = Self::default();
        self.cfg_override = config.into();
        self.icon = config
            .get("icon")
            .and_then(|v| v.clone())
            .unwrap_or(default.icon);
        self.max_length = config
            .get("max_length")
            .and_then(|v| v.as_ref().and_then(|v| v.parse().ok()))
            .unwrap_or(default.max_length);
        self.max_title_length = config
            .get("max_title_length")
            .and_then(|v| v.as_ref().and_then(|v| v.parse().ok()))
            .unwrap_or(default.max_title_length);
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
                            let title = title.to_string();
                            let artist = artist.to_string();
                            sender
                                .send(Message::update(move |reg| {
                                    reg.get_module_mut::<MediaMod>().update(title, artist)
                                }))
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
