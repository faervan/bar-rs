use std::{error::Error, time::Duration};

use bar_rs_derive::Builder;
use iced::{
    futures::{channel::mpsc::Sender, SinkExt},
    stream, Subscription,
};
use serde_json::Value;
use tokio::time::sleep;
use wayfire_rs::ipc::WayfireSocket;

use crate::{
    modules::wayfire::{WayfireWindowMod, WayfireWorkspaceMod},
    Message,
};

use super::Listener;

#[derive(Debug, Builder)]
pub struct WayfireListener;

async fn send_first_values(
    socket: &mut WayfireSocket,
    sender: &mut Sender<Message>,
) -> Result<(), Box<dyn Error>> {
    let title = socket.get_focused_view().await.ok().map(|v| v.title);
    let workspace = socket.get_focused_output().await?.workspace;
    sender
        .send(Message::update(move |reg| {
            reg.get_module_mut::<WayfireWindowMod>().set_active(title);
            reg.get_module_mut::<WayfireWorkspaceMod>().active = (workspace.x, workspace.y);
        }))
        .await?;
    Ok(())
}

impl Listener for WayfireListener {
    fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let Ok(mut socket) = WayfireSocket::connect().await else {
                    eprintln!("Failed to connect to wayfire socket");
                    return;
                };

                send_first_values(&mut socket, &mut sender)
                    .await
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to send initial wayfire module data: {e}")
                    });

                socket
                    .watch(Some(vec![
                        "wset-workspace-changed".to_string(),
                        "view-focused".to_string(),
                        "view-title-changed".to_string(),
                        "view-unmapped".to_string(),
                    ]))
                    .await
                    .expect("Failed to watch wayfire socket (but we're connected already)!");

                let mut active_window = None;

                while let Ok(Value::Object(msg)) = socket.read_message().await {
                    match msg.get("event") {
                        Some(Value::String(val)) if val == "wset-workspace-changed" => {
                            let Some(Value::Object(obj)) = msg.get("new-workspace") else {
                                continue;
                            };

                            // serde_json::Value::Object => (i64, i64)
                            if let Some((x, y)) = obj.get("x").and_then(|x| {
                                x.as_i64().and_then(|x| {
                                    obj.get("y").and_then(|y| y.as_i64().map(|y| (x, y)))
                                })
                            }) {
                                // With this wayfire will send an additional msg, see the None
                                // match arm... No idea why tho
                                sleep(Duration::from_millis(150)).await;
                                let title = socket.get_focused_view().await.ok().map(|v| v.title);
                                active_window = title.clone();
                                sender
                                    .send(Message::update(move |reg| {
                                        reg.get_module_mut::<WayfireWorkspaceMod>().active = (x, y);
                                        reg.get_module_mut::<WayfireWindowMod>().set_active(title)
                                    }))
                                    .await
                                    .unwrap();
                            }
                        }

                        Some(Value::String(val))
                            if val == "view-focused" || val == "view-title-changed" =>
                        {
                            let Some(Value::String(title)) = msg
                                .get("view")
                                .and_then(|v| v.as_object())
                                .and_then(|o| o.get("title").map(|t| t.to_owned()))
                            else {
                                continue;
                            };
                            match Some(&title) == active_window.as_ref() {
                                true => continue,
                                false => active_window = Some(title.clone()),
                            }
                            sender
                                .send(Message::update(move |reg| {
                                    reg.get_module_mut::<WayfireWindowMod>()
                                        .set_active(Some(title))
                                }))
                                .await
                                .unwrap();
                        }

                        // That sure seems useless, but we need the view-unmapped events that
                        // somehow end up in the None match arm
                        Some(Value::String(val)) if val == "view-unmapped" => {}

                        None => {
                            if let Some("ok") = msg.get("result").and_then(|r| r.as_str()) {
                                let Some(title) = msg.get("info").map(|info| {
                                    if info.is_null() {
                                        return None;
                                    }
                                    info.as_object()
                                        .and_then(|obj| obj.get("title"))
                                        .and_then(|t| t.as_str())
                                        .map(|s| s.to_string())
                                }) else {
                                    continue;
                                };
                                match title == active_window {
                                    true => continue,
                                    false => active_window = title.clone(),
                                }
                                sender
                                    .send(Message::update(move |reg| {
                                        reg.get_module_mut::<WayfireWindowMod>().set_active(title)
                                    }))
                                    .await
                                    .unwrap();
                            };
                        }

                        _ => eprintln!("got unknown event from wayfire ipc: {msg:#?}"),
                    }
                }

                eprintln!("Failed to read messages from the Wayfire socket!");
            })
        })
    }
}
