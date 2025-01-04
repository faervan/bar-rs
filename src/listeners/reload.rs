use std::{env, path::PathBuf, time::Duration};

use bar_rs_derive::Builder;
use iced::{futures::{executor, SinkExt}, stream, Subscription};
use notify::{event::{ModifyKind, RemoveKind}, Config, Error, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::time::sleep;

use crate::{config::{get_config, ConfigEntry}, Message};

use super::Listener;

#[derive(Debug, Builder)]
pub struct ReloadListener;

impl Listener for ReloadListener {
    fn config(&self) -> Vec<ConfigEntry> {
        vec![
            ConfigEntry::new("general", "hot_reloading", true)
        ]
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(||
            stream::channel(1, |mut sender| async move {
                let config_path = get_config(&mut sender).await.0;
                let config_pathx = config_path.clone();

                let mut watcher = RecommendedWatcher::new(
                    move |result: Result<Event, Error>| {
                        let event = result.unwrap();

                        if event.paths.contains(&config_pathx) {
                            if matches!(event.kind, EventKind::Modify(ModifyKind::Data(_)))
                                || matches!(event.kind, EventKind::Remove(RemoveKind::File))
                            {
                                executor::block_on(async {
                                    sender.send(Message::ReloadConfig)
                                        .await
                                        .unwrap_or_else(|err| {
                                            eprintln!("Trying to request config reload failed with err: {err}");
                                        });
                                });
                            }
                        }
                    },
                    Config::default()
                ).unwrap();

                watcher.watch(
                    &config_path.parent()
                        .unwrap_or(&default_config_path()),
                    RecursiveMode::Recursive
                ).unwrap();

                loop {
                    sleep(Duration::from_secs(1)).await;
                }
            })
        )
    }
}

fn default_config_path() -> PathBuf {
    format!("{}/.config/bar-rs",
        env::var("HOME").expect("Env $HOME is not set?")
    ).into()
}
