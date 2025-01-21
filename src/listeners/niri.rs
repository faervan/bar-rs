use std::{collections::HashMap, env, sync::Arc};

use bar_rs_derive::Builder;
use iced::{futures::SinkExt, stream, Subscription};
use niri_ipc::{socket::SOCKET_PATH_ENV, Event, Request};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
    sync::mpsc,
};

use crate::{
    config::ConfigEntry,
    modules::niri::{NiriWindowMod, NiriWorkspaceMod, SelectWorkspace},
    registry::Registry,
    Message, UpdateFn,
};

use super::Listener;

#[derive(Debug, Builder)]
pub struct NiriListener;

impl Listener for NiriListener {
    fn config(&self) -> Vec<ConfigEntry> {
        vec![]
    }
    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let (sx, mut rx) = mpsc::channel(1);
                sender
                    .send(Message::GetReceiver(sx, |reg| {
                        reg.get_module::<NiriWorkspaceMod>().sender.subscribe()
                    }))
                    .await
                    .unwrap();
                let mut receiver = rx.recv().await.unwrap();
                drop(rx);
                let socket_path = env::var(SOCKET_PATH_ENV).expect("No niri socket was found!");
                let mut socket = UnixStream::connect(&socket_path).await.unwrap();
                let mut buf = serde_json::to_string(&Request::EventStream).unwrap();
                socket.write_all(buf.as_bytes()).await.unwrap();
                socket.shutdown().await.unwrap();
                let mut reader = BufReader::new(socket);
                reader
                    .read_line(&mut buf)
                    .await
                    .map_err(|e| {
                        eprintln!("Failed to build an event stream with niri: {e}");
                    })
                    .ok();
                buf.clear();
                loop {
                    tokio::select! {
                        Ok(_) = reader.read_line(&mut buf) => {
                            let reply = serde_json::from_str::<Event>(&buf);
                            type F = Box<dyn FnOnce(&mut Registry) + Send + Sync>;
                            let msg: Option<F> = match reply {
                                Ok(event) => match event {
                                    Event::WorkspacesChanged { workspaces } => Some(Box::new(move |reg| {
                                        let active_ws = workspaces
                                            .iter()
                                            .find_map(|ws| ws.is_focused.then_some(ws.id));
                                        let mut workspaces: HashMap<String, Vec<niri_ipc::Workspace>> =
                                            workspaces.into_iter().fold(HashMap::new(), |mut acc, ws| {
                                                match acc
                                                    .get_mut(ws.output.as_ref().unwrap_or(&String::new()))
                                                {
                                                    Some(workspaces) => workspaces.push(ws),
                                                    None => {
                                                        acc.insert(
                                                            ws.output.clone().unwrap_or_default(),
                                                            vec![ws],
                                                        );
                                                    }
                                                }
                                                acc
                                            });
                                        for (_, workspaces) in workspaces.iter_mut() {
                                            workspaces.sort_by(|a, b| a.idx.cmp(&b.idx));
                                        }
                                        let ws_mod = reg.get_module_mut::<NiriWorkspaceMod>();
                                        ws_mod.focused = active_ws.unwrap();
                                        ws_mod.workspaces = workspaces
                                    })),
                                    Event::WorkspaceActivated { id, focused } => match focused {
                                        true => Some(Box::new(move |reg| {
                                            reg.get_module_mut::<NiriWorkspaceMod>().focused = id
                                        })),
                                        false => None,
                                    },
                                    Event::WindowsChanged { windows } => Some(Box::new(move |reg| {
                                        let window_mod = reg.get_module_mut::<NiriWindowMod>();
                                        window_mod.focused =
                                            windows.iter().find(|w| w.is_focused).map(|w| w.id);
                                        window_mod.windows = windows
                                            .into_iter()
                                            .map(|w| (w.id, (w.title, w.app_id)))
                                            .collect()
                                    })),
                                    Event::WindowFocusChanged { id } => Some(Box::new(move |reg| {
                                        reg.get_module_mut::<NiriWindowMod>().focused = id
                                    })),
                                    Event::WindowOpenedOrChanged { window } => Some(Box::new(move |reg| {
                                        let window_mod = reg.get_module_mut::<NiriWindowMod>();
                                        if window.is_focused {
                                            window_mod.focused = Some(window.id);
                                        }
                                        window_mod
                                        .windows
                                        .insert(window.id, (window.title, window.app_id));
                                    })),
                                    Event::WindowClosed { id } => Some(Box::new(move |reg| {
                                        reg.get_module_mut::<NiriWindowMod>().windows.remove(&id);
                                    })),
                                    _ => None,
                                },
                                Err(err) => {
                                    eprintln!("Failed to decode Niri IPC msg as Event: {err}");
                                    None
                                }
                                };
                                if let Some(msg) = msg {
                                    sender
                                        .send(Message::Update(Arc::new(UpdateFn(msg))))
                                        .await
                                        .unwrap();
                                }
                                buf.clear();
                            }
                        Ok(action) = receiver.recv() => {
                            if let Some(id) = action.downcast_ref::<SelectWorkspace>() {
                                let mut socket = UnixStream::connect(&socket_path).await.unwrap();
                                let buf = serde_json::to_string(&Request::Action(niri_ipc::Action::FocusWorkspace { reference: niri_ipc::WorkspaceReferenceArg::Id(*id) })).unwrap();
                                socket.write_all(buf.as_bytes()).await.unwrap();
                                socket.shutdown().await.unwrap();
                            }
                        }
                    }
                }
            })
        })
    }
}
