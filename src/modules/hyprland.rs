use std::{sync::Arc, time::Duration};

use hyprland::{data::{Client, Monitor, Workspace, Workspaces}, event_listener::AsyncEventListener, keyword::Keyword, shared::{HyprData, HyprDataActive, HyprDataActiveOptional, HyprDataVec}};
use iced::{futures::{channel::mpsc::Sender, SinkExt}, stream, widget::{row, text, text::{Rich, Span}}, Background, Border, Color, Length::Fill, Padding, Subscription};
use tokio::time::sleep;

use crate::{config::get_config, Message, BAR_HEIGHT, NERD_FONT};

use super::Module;

#[derive(Debug, Default)]
pub struct HyprWindowMod(Option<String>);

impl Module for HyprWindowMod {
    fn id(&self) -> String {
        "hyprland.window".to_string()
    }

    fn view(&self) -> iced::Element<Message> {
        row![
            text![
                "{}",
                self.0.as_ref()
                    .unwrap_or(&"".to_string())
            ].center().height(Fill)
        ].into()
    }
}

#[derive(Debug, Default)]
pub struct HyprWorkspaceMod {
    active: usize,
    open: Vec<String>,
}

impl Module for HyprWorkspaceMod {
    fn id(&self) -> String {
        "hyprland.workspaces".to_string()
    }

    fn view(&self) -> iced::Element<Message> {
        row(
            self.open
                .iter()
                .enumerate()
                .map(|(id, ws)| {
                    let mut span = Span::new(ws)
                        .size(20)
                        .padding(Padding {top: -3., bottom: 0., right: 10., left: 5.})
                        .font(NERD_FONT);
                    if id == self.active {
                        span = span
                            .background(Background::Color(Color::WHITE).scale_alpha(0.5))
                            .border(Border::default().rounded(8))
                            .color(Color::BLACK);
                    }
                    Rich::with_spans([span])
                        .center()
                        .height(Fill)
                        .into()
                })
        ).spacing(15).into()
    }

    fn subscription(&self) -> Option<iced::Subscription<Message>> {
        Some(Subscription::run(||
            stream::channel(1, |mut sender| async move {

                let config = get_config(&mut sender).await.1;

                update_workspaces(&mut sender, None).await;
                if let Ok(window) = Client::get_active() {
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
                listener.add_workspace_changed_handler(move |data| {
                    let mut sender = senderx.clone();
                    Box::pin(async move {
                        update_workspaces(&mut sender, Some(data.id)).await;
                    })
                });

                listener.add_config_reloaded_handler(move || {
                    let monitor = config.monitor.clone();
                    Box::pin(async move {
                        reserve_bar_space(&monitor)
                    })
                });

                listener.add_fullscreen_state_changed_handler(move |_| {
                    let mut sender = sender.clone();
                    Box::pin(async move {
                        sender.send(Message::ToggleWindow)
                            .await
                            .unwrap_or_else(|err| {
                                eprintln!("Trying to send fullscreen toggle event failed with err: {err}");
                            });
                    })
                });

                listener.start_listener_async().await
                    .expect("Failed to listen for hyprland events");
            })
        ))
    }
}

impl From<(Workspaces, usize)> for HyprWorkspaceMod {
    fn from(value: (Workspaces, usize)) -> Self {
        let mut workspaces = Self::default();
        let mut list = value.0.to_vec();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list.iter()
            .for_each(
                |ws| workspaces.open.push(ws.name.clone())
            );
        workspaces.active = list.iter()
            .position(|ws| ws.id as usize == value.1)
            .unwrap_or(0);
        workspaces
    }
}

async fn update_window(sender: &mut Sender<Message>, window: Option<String>) {
    sender.send(Message::UpdateModule {
            id: "hyprland.window".to_string(),
            data: Arc::new(HyprWindowMod(window))
        })
        .await
        .unwrap_or_else(|err| {
            eprintln!("Trying to send workspaces failed with err: {err}");
        });
}

async fn update_workspaces(sender: &mut Sender<Message>, active: Option<i32>) {
    // Sleep a bit, to reduce the probability that a nonexisting ws is still reported active
    sleep(Duration::from_millis(10)).await;
    if let Ok(workspaces) = Workspaces::get() {
        sender.send(Message::UpdateModule {
            id: "hyprland.workspaces".to_string(),
            data: Arc::new(HyprWorkspaceMod::from((
                workspaces,
                active.unwrap_or(
                    Workspace::get_active()
                        .map(|ws| ws.id)
                        .unwrap_or(0)
                ) as usize,
            )))
        })
        .await
        .unwrap_or_else(|err| {
            eprintln!("Trying to send workspaces failed with err: {err}");
        });
    }
}

pub fn get_monitor_name() -> String {
    Monitor::get_active()
        .map(|m| m.name)
        .unwrap_or("eDP-1".to_string())
}

pub fn reserve_bar_space(monitor: &String) {
    Keyword::set("monitor", format!("{monitor}, addreserved, {BAR_HEIGHT}, 0, 0, 0"))
        .expect("Failed to set reserved space using hyprctl");
}
