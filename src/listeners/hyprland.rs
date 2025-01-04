use std::time::Duration;

use bar_rs_derive::Builder;
use hyprland::{data::{Client, Monitor}, event_listener::AsyncEventListener, shared::{HyprDataActive, HyprDataActiveOptional}};
use iced::{futures::SinkExt, stream, window, Subscription, Task};
use tokio::time::sleep;

use crate::{config::{get_config, ConfigEntry}, modules::hyprland::{reserve_bar_space, window::update_window, workspaces::{get_workspaces, HyprWorkspaceMod}}, Bar, Message};

use super::Listener;

#[derive(Debug, Default, Builder)]
pub struct HyprListener {
    active_monitor: String,
    fullscreen: bool,
}

impl Listener for HyprListener {
    fn config(&self) -> Vec<ConfigEntry> {
        vec![
            ConfigEntry::new("general", "close_on_fullscreen", true)
        ]
    }
    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(||
            stream::channel(1, |mut sender| async move {

                let config = get_config(&mut sender).await.1;

                let monitor = Monitor::get_active_async().await.unwrap().name;
                let workspaces = get_workspaces(None).await;
                sender.send(Message::update(Box::new(
                        move |reg| {
                            let hypr = reg.get_listener_mut::<HyprListener>();
                            hypr.active_monitor = monitor;
                            hypr.fullscreen = workspaces.open[workspaces.active].1;
                            *reg.get_module_mut::<HyprWorkspaceMod>() = workspaces;
                        }
                    )))
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
                        update_window(
                            &mut sender,
                            data.map(|name| match name.title.len() > 25 {
                                true => format!(
                                    "{}...",
                                    &name.title
                                        .chars()
                                        .take(22)
                                        .collect::<String>()
                                ),
                                false => name.title
                            })
                        ).await;
                    })
                });

                let senderx = sender.clone();
                listener.add_active_monitor_changed_handler(move |data| {
                    let mut sender = senderx.clone();
                    Box::pin(async move {
                        sender.send(Message::update(Box::new(
                                move |reg| reg.get_listener_mut::<HyprListener>().active_monitor = data.monitor_name
                            )))
                            .await
                            .unwrap_or_else(|err| {
                                eprintln!("Trying to send fullscreen toggle event failed with err: {err}");
                            });
                    })
                });

                let senderx = sender.clone();
                listener.add_workspace_changed_handler(move |data| {
                    let mut sender = senderx.clone();
                    Box::pin(async move {
                        let workspaces = get_workspaces(Some(data.id)).await;
                        sender.send(Message::update(Box::new(
                                move |reg| *reg.get_module_mut::<HyprWorkspaceMod>() = workspaces
                            )))
                            .await
                            .unwrap_or_else(|err| {
                                eprintln!("Trying to send workspaces failed with err: {err}");
                            });
                        sender.send(Message::Perform(
                                |reg, config, open| {
                                    let monitor_focused = reg.get_listener::<HyprListener>().active_monitor == config.monitor;
                                    let workspaces = reg.get_module::<HyprWorkspaceMod>();
                                    if monitor_focused {
                                        match (workspaces.open[workspaces.active].1, *open) {
                                            (false, false) => {
                                                *open = !*open;
                                                Bar::open_window().1.map(|_| Message::WindowOpened)
                                            }
                                            (true, true) => {
                                                *open = !*open;
                                                window::get_latest().and_then(window::close)
                                            }
                                            _ => Task::none()
                                        }
                                    } else {
                                        Task::none()
                                    }
                                }
                            ))
                            .await
                            .unwrap_or_else(|err| {
                                eprintln!("Trying to send fullscreen toggle event failed with err: {err}");
                            });
                    })
                });

                listener.add_fullscreen_state_changed_handler(move |_| {
                    let mut sender = sender.clone();
                    Box::pin(async move {
                        // Need this sleep to prevent wayland crashes: "Tried to add event to destroyed queue"
                        sleep(Duration::from_millis(30)).await;
                        sender.send(Message::Perform(
                                |reg, config, open| {
                                    let monitor_focused = reg.get_listener::<HyprListener>().active_monitor == config.monitor;
                                    let workspaces = reg.get_module_mut::<HyprWorkspaceMod>();
                                    workspaces.open[workspaces.active].1 = !workspaces.open[workspaces.active].1;
                                    if monitor_focused && config.close_on_fullscreen {
                                        *open = !*open;
                                        match !workspaces.open[workspaces.active].1 {
                                            true => Bar::open_window().1.map(|_| Message::WindowOpened),
                                            false => window::get_latest().and_then(window::close)
                                        }
                                    } else {
                                        Task::none()
                                    }
                                }
                            ))
                            .await
                            .unwrap_or_else(|err| {
                                eprintln!("Trying to send fullscreen toggle event failed with err: {err}");
                            });
                    })
                });

                listener.add_config_reloaded_handler(move || {
                    let monitor = config.monitor.clone();
                    Box::pin(async move {
                        reserve_bar_space(&monitor)
                    })
                });

                listener.start_listener_async().await
                    .expect("Failed to listen for hyprland events");
            })
        )
    }
}
