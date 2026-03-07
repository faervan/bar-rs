use bar_rs_derive::Builder;
use hyprland::{data::Client, event_listener::AsyncEventListener, shared::HyprDataActiveOptional};
use iced::{
    Subscription,
    futures::{SinkExt, channel::mpsc::Sender},
    stream,
};

use crate::{
    Message,
    config::ConfigEntry,
    modules::hyprland::{
        window::update_window,
        workspaces::{HyprWorkspaceMod, get_workspaces},
    },
};

use super::Listener;

#[derive(Debug, Builder)]
pub struct HyprListener;

impl Listener for HyprListener {
    fn config(&self) -> Vec<ConfigEntry> {
        vec![]
    }
    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(|| {
            stream::channel(1, |mut sender: Sender<Message>| async move {
                let workspaces = get_workspaces(None).await;
                sender
                    .send(Message::update(move |reg| {
                        let ws = reg.get_module_mut::<HyprWorkspaceMod>();
                        ws.active = workspaces.0;
                        ws.open = workspaces.1;
                    }))
                    .await
                    .unwrap_or_else(|err| {
                        eprintln!("Trying to send workspaces failed with err: {err}");
                    });
                if let Ok(window) = Client::get_active_async().await {
                    update_window(&mut sender, window.map(|w| w.title)).await;
                }

                let mut listener = AsyncEventListener::new();

                let senderx = sender.clone();
                listener.add_active_window_changed_handler(move |data| {
                    let mut sender = senderx.clone();
                    Box::pin(async move {
                        update_window(&mut sender, data.map(|window| window.title)).await;
                    })
                });

                let senderx = sender.clone();
                listener.add_workspace_changed_handler(move |data| {
                    let mut sender = senderx.clone();
                    Box::pin(async move {
                        let workspaces = get_workspaces(Some(data.id)).await;
                        sender
                            .send(Message::update(move |reg| {
                                let ws = reg.get_module_mut::<HyprWorkspaceMod>();
                                ws.active = workspaces.0;
                                ws.open = workspaces.1;
                            }))
                            .await
                            .unwrap_or_else(|err| {
                                eprintln!("Trying to send workspaces failed with err: {err}");
                            });
                    })
                });

                listener
                    .start_listener_async()
                    .await
                    .expect("Failed to listen for hyprland events");
            })
        })
    }
}
