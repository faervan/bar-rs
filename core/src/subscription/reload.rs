use std::time::Duration;

use iced::{futures::executor, stream, Subscription};
use notify::{
    event::ModifyKind, Config, Error, Event, EventKind, RecommendedWatcher, RecursiveMode,
    Watcher as _,
};
use tokio::time::sleep;

use crate::message::{Message, MessageSenderExt as _};

pub fn subscription() -> Subscription<Message> {
    Subscription::run(|| {
        stream::channel(1, |mut sender| async move {
            let config_path = sender.read_with(|state| state.config_root.root()).await;

            let mut watcher = RecommendedWatcher::new(
                move |result: Result<Event, Error>| {
                    let event = result.unwrap();

                    if matches!(event.kind, EventKind::Modify(ModifyKind::Data(_))) {
                        executor::block_on(async {
                            sender.send(Message::ReloadConfig).await;
                        });
                    }
                },
                Config::default(),
            )
            .unwrap();

            watcher
                .watch(&config_path, RecursiveMode::Recursive)
                .unwrap();

            loop {
                sleep(Duration::from_secs(1)).await;
            }
        })
    })
}
