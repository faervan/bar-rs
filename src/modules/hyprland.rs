use std::time::Duration;

use hyprland::{data::{Client, Monitor, Workspace, Workspaces}, event_listener::AsyncEventListener, keyword::Keyword, shared::{HyprData, HyprDataActive, HyprDataActiveOptional, HyprDataVec}};
use iced::{futures::{channel::mpsc::Sender, SinkExt, Stream}, stream};
use tokio::{sync::mpsc, time::sleep};

use crate::{Message, BAR_HEIGHT};

#[derive(Debug, Default, Clone)]
pub struct OpenWorkspaces {
    pub active: usize,
    pub open: Vec<String>,
}

impl From<(Workspaces, usize)> for OpenWorkspaces {
    fn from(value: (Workspaces, usize)) -> Self {
        let mut workspaces = OpenWorkspaces::default();
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

pub fn hyprland_events() -> impl Stream<Item = Message> {
    stream::channel(1, |mut sender| async move {

        let (sx, mut rx) = mpsc::channel(1);
        sender.send(Message::GetConfig(sx))
            .await
            .unwrap_or_else(|err| {
                eprintln!("Trying to request config failed with err: {err}");
            });
        let Some(config) = rx.recv().await else {
            panic!("Something went wrong! No config was returned");
        };

        update_workspaces(&mut sender, None).await;
        if let Ok(Some(window)) = Client::get_active() {
            sender.send(Message::Window(Some(window.title)))
                .await
                .unwrap_or_else(|err| {
                    eprintln!("Trying to send workspaces failed with err: {err}");
                });
        }

        let mut listener = AsyncEventListener::new();

        let sender1 = sender.clone();
        listener.add_active_window_changed_handler(move |data| {
            let mut sender = sender1.clone();
            Box::pin(async move {
                sender
                    .send(Message::Window(data.map(|name| match name.title.len() > 25 {
                        true => format!(
                            "{}...",
                            &name.title
                                .chars()
                                .take(22)
                                .collect::<String>()
                        ),
                        false => name.title
                    })))
                    .await
                    .unwrap();
            })
        });

        listener.add_workspace_changed_handler(move |data| {
            let mut sender = sender.clone();
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

        listener.start_listener_async().await
            .expect("Failed to listen for hyprland events");
    })
}

async fn update_workspaces(sender: &mut Sender<Message>, active: Option<i32>) {
    // Sleep a bit, to reduce the probability that a nonexisting ws is still reported active
    sleep(Duration::from_millis(10)).await;
    if let Ok(workspaces) = Workspaces::get() {
        sender.send(Message::Workspaces(
            OpenWorkspaces::from((
                workspaces,
                active.unwrap_or(
                    Workspace::get_active()
                        .map(|ws| ws.id)
                        .unwrap_or(0)
                ) as usize,
            )
        )))
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
