use std::{env, path::PathBuf, time::Duration};

use iced::{futures::{executor, SinkExt}, stream, Subscription};
use notify::{event::ModifyKind, Config, Error, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::time::sleep;

use crate::Message;

use super::get_config;

pub fn config_changed() -> Subscription<Message> {
    Subscription::run(||
        stream::channel(1, |mut sender| async move {
            let config_path = get_config(&mut sender).await.0;
            let config_path2 = config_path.clone();

            let mut watcher = RecommendedWatcher::new(
                move |result: Result<Event, Error>| {
                    let event = result.unwrap();

                    if event.paths.contains(&config_path2) {
                        if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
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
                Config::default().with_poll_interval(Duration::from_secs(2)).with_compare_contents(true)
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

fn default_config_path() -> PathBuf {
    format!("{}/.config/bar-rs",
        env::var("HOME").expect("Env $HOME is not set?")
    ).into()
}
