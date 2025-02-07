use std::collections::{BTreeMap, HashSet};
use std::{collections::HashMap, process::Stdio};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::widget::button::Style;
use iced::widget::{column, container, image, row, scrollable, Container, Text};
use iced::Length::Fill;
use iced::{futures::SinkExt, stream, widget::text, Element, Subscription};
use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use crate::button::button;
use crate::config::popup_config::{PopupConfig, PopupConfigOverride};
use crate::helpers::UnEscapeString;
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

#[derive(Debug, Builder)]
pub struct MediaMod {
    track: Option<TrackInfo>,
    img: Option<Vec<u8>>,
    active_player: Option<String>,
    cfg_override: ModuleConfigOverride,
    popup_cfg_override: PopupConfigOverride,
    icon: String,
    ctrl_icons: PlayerCtrlIcons,
    max_length: usize,
    max_title_length: usize,
    players: HashSet<String>,
    cover_width: f32,
}

#[derive(Debug)]
struct PlayerCtrlIcons {
    previous: String,
    play: String,
    pause: String,
    next: String,
}

impl Default for MediaMod {
    fn default() -> Self {
        Self {
            track: None,
            img: None,
            active_player: None,
            cfg_override: Default::default(),
            popup_cfg_override: PopupConfigOverride {
                width: Some(300),
                height: Some(450),
                ..Default::default()
            },
            icon: String::from(""),
            ctrl_icons: PlayerCtrlIcons {
                previous: String::from("󰒮"),
                play: String::from(""),
                pause: String::from(""),
                next: String::from("󰒭"),
            },
            max_length: 28,
            max_title_length: 16,
            players: HashSet::from(["spotify".to_string(), "kew".to_string()]),
            cover_width: 260.,
        }
    }
}

impl MediaMod {
    fn get_active_trimmed(&self) -> Option<String> {
        self.track.as_ref().map(|track| {
            let mut title = track.title.clone();
            let mut artist = track.artist.clone();
            if self.is_overlength() {
                if title.len() > self.max_title_length {
                    title = title.chars().take(self.max_title_length - 3).collect();
                    title.push_str("...");
                }
                if title.len() + artist.len() + 3 > self.max_length {
                    artist = artist
                        .chars()
                        .take(self.max_length - title.len() - 6)
                        .collect();
                    artist.push_str("...");
                }
            }
            match track.artist.is_empty() {
                true => title,
                false => format!("{} - {}", title, artist),
            }
        })
    }

    fn is_overlength(&self) -> bool {
        self.track
            .as_ref()
            .is_some_and(|t| t.title.len() + t.artist.len() + 3 > self.max_length)
    }

    fn new_track(&mut self, track: TrackInfo) {
        if self.players.contains(&track.player) {
            self.active_player = Some(track.player.clone());
            self.track = Some(track)
        }
    }
}

#[derive(Debug)]
struct TrackInfo {
    title: String,
    artist: String,
    album: String,
    art_url: String,
    player: String,
    art_is_local: bool,
    length: f32,
    paused: bool,
}

impl<'de> Deserialize<'de> for TrackInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: serde_json::Map<String, serde_json::Value> =
            Deserialize::deserialize(deserializer)?;

        let mut art_url = map
            .get("art_url")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let art_is_local = match art_url.strip_prefix("file://") {
            Some(file) => {
                art_url = file.to_string();
                true
            }
            None => false,
        };

        Ok(TrackInfo {
            title: map
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            artist: map
                .get("artist")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            album: map
                .get("album")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            player: map
                .get("player")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            art_url,
            art_is_local,
            length: map
                .get("length")
                .and_then(|v| v.as_f64())
                .unwrap_or_default() as f32,
            paused: map
                .get("status")
                .map(|v| !matches!(v.as_str(), Some("Playing")))
                .unwrap_or(true),
        })
    }
}

impl Module for MediaMod {
    fn name(&self) -> String {
        "media".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        popup_config: &PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<Message> {
        button(
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
                    text(self.get_active_trimmed().unwrap_or_default())
                        .fill(anchor)
                        .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                        .color(self.cfg_override.text_color.unwrap_or(config.text_color))
                )
                .padding(self.cfg_override.text_margin.unwrap_or(config.text_margin))
            ]
            .spacing(self.cfg_override.spacing.unwrap_or(config.spacing)),
        )
        .on_event_maybe_with(self.track.as_ref().map(|_| {
            Message::popup::<Self>(
                self.popup_cfg_override.width.unwrap_or(popup_config.width),
                self.popup_cfg_override
                    .height
                    .unwrap_or(popup_config.height),
                anchor,
            )
        }))
        .style(|_, _| Style::default())
        .into()
    }

    fn popup_view<'a>(
        &'a self,
        config: &'a PopupConfig,
        template: &Handlebars,
    ) -> Element<'a, Message> {
        let fmt_text = |text: Text<'a>| -> Container<'a, Message> {
            container(
                text.size(
                    self.popup_cfg_override
                        .font_size
                        .unwrap_or(config.font_size),
                )
                .color(
                    self.popup_cfg_override
                        .text_color
                        .unwrap_or(config.text_color),
                ),
            )
            .padding(
                self.popup_cfg_override
                    .text_margin
                    .unwrap_or(config.text_margin),
            )
        };
        container(match &self.track {
            Some(track) => {
                let minutes = (track.length / 60000000.).trunc();
                let icon = |icon| {
                    container(
                        text(icon)
                            .font(NERD_FONT)
                            .size(
                                self.popup_cfg_override
                                    .icon_size
                                    .unwrap_or(config.icon_size),
                            )
                            .color(
                                self.popup_cfg_override
                                    .icon_color
                                    .unwrap_or(config.icon_color),
                            ),
                    )
                    .padding(
                        self.popup_cfg_override
                            .icon_margin
                            .unwrap_or(config.icon_margin),
                    )
                };
                let cmd = |cmd| {
                    Message::command_sh(format!(
                        "playerctl {cmd}{}",
                        self.active_player
                            .as_ref()
                            .map(|p| format!(" -p {p}"))
                            .unwrap_or_default()
                    ))
                };
                let status = track
                    .paused
                    .then_some(" (paused)".to_string())
                    .unwrap_or_default();
                let length_ctx = BTreeMap::from([
                    ("minutes", minutes as u32),
                    (
                        "seconds",
                        ((track.length / 1000000.) - minutes * 60.).round() as u32,
                    ),
                ]);
                let length = template
                    .render("media_popup_length", &length_ctx)
                    .map_err(|e| eprintln!("Failed to render media popup length: {e}"))
                    .unwrap_or_default();
                let ctx = BTreeMap::from([
                    ("title", &track.title),
                    ("artist", &track.artist),
                    ("album", &track.album),
                    ("status", &status),
                    ("length", &length),
                ]);
                <iced::widget::Scrollable<'_, Message> as Into<Element<Message>>>::into(scrollable(
                    column![
                        match track.art_is_local {
                            true => <iced::widget::Image as Into<Element<Message>>>::into(
                                image(
                                    track
                                        .art_url
                                        .strip_prefix("file://")
                                        .unwrap_or(&track.art_url)
                                )
                                .width(self.cover_width)
                            ),
                            false =>
                                if let Some(bytes) = self.img.clone() {
                                    <iced::widget::Image as Into<Element<Message>>>::into(image(
                                        image::Handle::from_bytes(bytes),
                                    ))
                                } else {
                                    fmt_text(text("No cover available")).into()
                                },
                        },
                        container(
                            row![
                                button(icon(&self.ctrl_icons.previous))
                                    .on_event(cmd("previous"))
                                    .style(|_, _| Style::default()),
                                button(icon(match track.paused {
                                    true => &self.ctrl_icons.play,
                                    false => &self.ctrl_icons.pause,
                                }))
                                .on_event(cmd("play-pause"))
                                .style(|_, _| Style::default()),
                                button(icon(&self.ctrl_icons.next))
                                    .on_event(cmd("next"))
                                    .style(|_, _| Style::default()),
                            ]
                            .spacing(20)
                        )
                        .center_x(Fill),
                        fmt_text(text(
                            template
                                .render("media_popup", &ctx)
                                .map_err(|e| eprintln!("Failed to render media popup stats: {e}"))
                                .unwrap_or_default()
                        )),
                    ]
                    .spacing(self.popup_cfg_override.spacing.unwrap_or(config.spacing)),
                ))
            }
            None => fmt_text(text("No media is playing right now")).into(),
        })
        .padding(self.popup_cfg_override.padding.unwrap_or(config.padding))
        .style(|_| container::Style {
            background: Some(
                self.popup_cfg_override
                    .background
                    .unwrap_or(config.background),
            ),
            border: self.popup_cfg_override.border.unwrap_or(config.border),
            ..Default::default()
        })
        .fill_maybe(
            self.popup_cfg_override
                .fill_content_to_size
                .unwrap_or(config.fill_content_to_size),
        )
        .into()
    }

    impl_wrapper!();

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        popup_config: &HashMap<String, Option<String>>,
        templates: &mut Handlebars,
    ) {
        let default = Self::default();
        self.cfg_override = config.into();
        self.popup_cfg_override.update(popup_config);
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
        self.players = popup_config
            .get("players")
            .and_then(|v| {
                v.as_ref()
                    .map(|v| v.split(',').map(|i| i.trim().to_string()).collect())
            })
            .unwrap_or(default.players);
        self.cover_width = popup_config
            .get("cover_width")
            .and_then(|v| v.as_ref().and_then(|v| v.parse().ok()))
            .unwrap_or(default.cover_width);
        self.ctrl_icons = {
            let default = default.ctrl_icons;
            PlayerCtrlIcons {
                previous: popup_config
                    .get("icon_previous")
                    .cloned()
                    .flatten()
                    .unwrap_or(default.previous),
                play: popup_config
                    .get("icon_play")
                    .cloned()
                    .flatten()
                    .unwrap_or(default.play),
                pause: popup_config
                    .get("icon_pause")
                    .cloned()
                    .flatten()
                    .unwrap_or(default.pause),
                next: popup_config
                    .get("icon_next")
                    .cloned()
                    .flatten()
                    .unwrap_or(default.next),
            }
        };
        templates
            .register_template_string(
                "media_popup",
                popup_config.get("format").unescape().unwrap_or(
                    "{{title}}{{status}}\nin: {{album}}\nby: {{artist}}\n{{length}}".to_string(),
                ),
            )
            .unwrap_or_else(|e| eprintln!("Failed to parse battery popup time format: {e}"));
        templates
            .register_template_string(
                "media_popup_length",
                popup_config
                    .get("format_length")
                    .unescape()
                    .unwrap_or("{{minutes}}min {{seconds}}sec".to_string()),
            )
            .unwrap_or_else(|e| eprintln!("Failed to parse battery popup time format: {e}"));
    }

    impl_on_click!();

    fn subscription(&self) -> Option<iced::Subscription<Message>> {
        Some(Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let mut child = Command::new("sh")
                    .arg("-c")
                    .arg(
                        "playerctl --follow metadata --format '{\"title\": \"{{title}}\", \"artist\": \"{{artist}}\", \"album\": \"{{album}}\", \"art_url\": \"{{mpris:artUrl}}\", \"length\": {{mpris:length}}, \"status\": \"{{status}}\", \"player\": \"{{playerName}}\"}'",
                    )
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Failed to read output from playerctl");

                let stdout = child
                    .stdout
                    .take()
                    .expect("child did not have a handle to stdout");

                let mut reader = BufReader::new(stdout).lines();
                let mut last_track = String::new();

                while let Some(track) = reader
                    .next_line()
                    .await
                    .ok()
                    .flatten()
                    .and_then(|line| serde_json::from_str::<TrackInfo>(line.as_str()).ok())
                {
                    if let Some(url) = (!track.art_is_local).then_some(track.art_url.clone()) {
                        if url != last_track {
                            last_track = url.clone();
                            let mut sender = sender.clone();
                            tokio::task::spawn(async move {
                                let Ok(response) = reqwest::get(&url).await else {
                                    eprintln!("Failed to get media cover: \"{url}\"");
                                    return;
                                };
                                let Ok(bytes) = response.bytes().await else {
                                    eprintln!("Failed to get bytes from media cover: \"{url}\"");
                                    return;
                                };
                                sender
                                    .send(Message::update(move |reg| {
                                        reg.get_module_mut::<MediaMod>().img = Some(bytes.to_vec())
                                    }))
                                    .await
                                    .unwrap();
                            });
                        }
                    }
                    sender
                        .send(Message::update(move |reg| {
                            reg.get_module_mut::<MediaMod>().new_track(track)
                        }))
                        .await
                        .unwrap();
                }
            })
        }))
    }
}
