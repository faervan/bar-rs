use std::process::Stdio;

use iced::{futures::{SinkExt, Stream}, stream};
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command};

use crate::Message;

const MAX_LENGTH: usize = 35;
const MAX_TITLE_LENGTH: usize = 20;

#[derive(Default, Debug, Clone)]
pub struct MediaStats {
    pub title: String,
    pub artist: Option<String>,
    pub icon: &'static str,
}


pub fn playerctl() -> impl Stream<Item = Message> {
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
                    sender.send(Message::Media(MediaStats {
                            title,
                            artist: match artist.as_str() == "" {
                                true => None,
                                false => Some(artist)
                            },
                            icon: ""
                        }))
                        .await
                        .unwrap_or_else(|err| {
                            eprintln!("Trying to send cpu_usage failed with err: {err}");
                        });
                }
            }
        }
    })
}
