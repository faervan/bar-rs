use std::{process::Stdio, sync::Arc};

use iced::{futures::SinkExt, stream, widget::{row, text}, Length::Fill, Subscription};
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command};

use crate::{Message, NERD_FONT};

use super::Module;

const MAX_LENGTH: usize = 35;
const MAX_TITLE_LENGTH: usize = 20;

#[derive(Default, Debug)]
pub struct MediaMod {
    title: String,
    artist: Option<String>,
    icon: &'static str,
}

impl Module for MediaMod {
    fn id(&self) -> String {
        "media".to_string()
    }

    fn view(&self) -> iced::Element<Message> {
        row![
            text!("{}", self.icon)
                .center().height(Fill).size(20).font(NERD_FONT),
            text![
                "{}{}",
                self.title,
                self.artist.as_ref()
                    .map(|name| format!(" - {name}"))
                    .unwrap_or("".to_string())
            ].center().height(Fill)
        ].spacing(15).into()
    }

    fn subscription(&self) -> Option<iced::Subscription<Message>> {
        Some(Subscription::run(||
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
                        if title != "" {
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
                            sender.send(Message::UpdateModule {
                                    id: "media".to_string(),
                                    data: Arc::new(MediaMod {
                                        title,
                                        artist: match artist.as_str() == "" {
                                            true => None,
                                            false => Some(artist)
                                        },
                                        icon: ""
                                    })
                                })
                                .await
                                .unwrap_or_else(|err| {
                                    eprintln!("Trying to send cpu_usage failed with err: {err}");
                                });
                        }
                    }
                }
            })
        ))
    }
}
