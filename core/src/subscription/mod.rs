use std::{fmt::Debug, sync::Arc, time::Duration};

use iced::{
    event::{listen_with, wayland, PlatformSpecific},
    futures::future::BoxFuture,
    stream,
};
use log::{error, warn};
use tokio::{sync::mpsc, time::sleep};

use crate::{
    daemon,
    message::{Message, MessageSenderExt as _, UpdateFn},
    state::State,
};

mod reload;

impl State {
    pub fn subscribe(&self) -> iced::Subscription<Message> {
        let module_subs = iced::Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let subs = sender.read_subscriptions().await;
                let (sx, mut rx) = mpsc::channel(1);

                let config = sender.read_config().await;

                for sub in subs {
                    sub.run(&sx);
                }

                drop(sx);

                let mut updates = vec![];
                loop {
                    tokio::select! {
                        all_subs_dropped = async {
                            loop {
                                match rx.recv().await {
                                    Some(SubscriptionUpdate::Buffered(update)) => {
                                        updates.push(update);
                                    }
                                    Some(SubscriptionUpdate::Immediate(update)) => {
                                        updates.push(update);
                                        return false;
                                    }
                                    None => {
                                        warn!("All Subscription senders dropped, canceling Subscription");
                                        return true;
                                    }
                                }
                            }
                        } => if all_subs_dropped {return;},
                        _ = sleep(Duration::from_secs_f32(config.reload_interval)) => {}
                    }
                    sender
                        .send(Message::Update(std::mem::take(&mut updates)))
                        .await;
                }
            })
        });

        let reload_watcher = match self.hot_reloading() {
            true => reload::subscription(),
            false => iced::Subscription::none(),
        };

        iced::Subscription::batch([
            reload_watcher,
            // Window events
            listen_with(|event, _, _| {
                if let iced::Event::PlatformSpecific(PlatformSpecific::Wayland(
                    wayland::Event::Output(event, wl_output),
                )) = event
                {
                    Some(Message::OutputEvent {
                        event: Box::new(event),
                        wl_output,
                    })
                } else {
                    None
                }
            }),
            // IPC commands
            iced::Subscription::run(|| {
                stream::channel(1, |mut sender| async move {
                    let listener = match daemon::bind_to_ipc(&mut sender).await {
                        Ok(l) => l,
                        Err(e) => {
                            error!("Failed to bind to IPC socket: {e}");
                            return;
                        }
                    };
                    if let Err(e) = daemon::publish_ipc_commands(sender, listener).await {
                        error!("IPC publisher failed: {e}");
                    }
                })
            }),
            module_subs,
        ])
    }
}

pub enum SubscriptionUpdate {
    Buffered(UpdateFn),
    Immediate(UpdateFn),
}

impl SubscriptionUpdate {
    pub fn buffered<F>(f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Self::Buffered(UpdateFn(Arc::new(Box::new(f))))
    }

    pub fn immediate<F>(f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Self::Immediate(UpdateFn(Arc::new(Box::new(f))))
    }
}

#[allow(clippy::type_complexity)]
pub struct Subscription(
    Option<
        Box<
            dyn FnOnce(mpsc::Sender<SubscriptionUpdate>) -> BoxFuture<'static, ()>
                + Send
                + Sync
                + 'static,
        >,
    >,
);

impl Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Subscription(Option<Box<dyn FnOnce(mpsc::Sender<SubscriptionUpdate>)
            -> BoxFuture<'static, ()> + 'static>>)"
        )
    }
}

impl Subscription {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(mpsc::Sender<SubscriptionUpdate>) -> BoxFuture<'static, ()>
            + Send
            + Sync
            + 'static,
    {
        Self(Some(Box::new(f)))
    }

    pub fn none() -> Self {
        Self(None)
    }

    fn run(self, sx: &mpsc::Sender<SubscriptionUpdate>) {
        if let Some(f) = self.0 {
            let sx = sx.clone();
            tokio::spawn(f(sx));
        }
    }
}
