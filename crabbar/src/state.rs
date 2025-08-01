use core::{
    config::ConfigOptions,
    window::{Window, WindowOpenOptions},
};
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use iced::{
    event::wayland, platform_specific::shell::commands::layer_surface::destroy_layer_surface,
    stream, window::Id, Element, Task,
};
use ipc::{IpcRequest, IpcResponse};
use log::{error, info};
use smithay_client_toolkit::{
    output::OutputInfo, reexports::client::protocol::wl_output::WlOutput,
};
use tokio::time::sleep;

use crate::{config::Config, daemon, message::Message};

#[derive(Debug, Default)]
pub struct State {
    pub socket_path: PathBuf,
    pid_path: PathBuf,
    outputs: HashMap<WlOutput, Option<OutputInfo>>,
    /// If false, we have to wait for new Outputs before opening a window
    outputs_ready: bool,
    // TODO! Remove config from State
    config: Arc<Config>,
    windows: HashMap<Id, Window>,
    window_ids: HashMap<usize, Id>,
    opening_queue: VecDeque<Id>,
    /// Every opened window gets a unique ID equal to the count of windows opened beforehand
    id_count: usize,
    configurations: HashMap<String, ConfigOptions>,
}

impl State {
    pub fn new(
        socket_path: PathBuf,
        pid_path: PathBuf,
        open_window: bool,
        opts: WindowOpenOptions,
    ) -> (Self, Task<Message>) {
        let mut state = State {
            socket_path,
            pid_path,
            ..Default::default()
        };
        let task = match open_window {
            true => state.open_window(opts).0,
            false => Task::none(),
        };
        (state, task)
    }

    pub fn title(&self, _id: Id) -> String {
        "crabbar".to_string()
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        use Message::*;
        match msg {
            ReadState(f) => f.execute(self),
            FetchSubscriptions(sx) => {
                sx.send(vec![]).unwrap();
            }
            FetchConfig(sx) => sx.send(self.config.clone()).unwrap(),
            Update(updates) => {
                for updatefn in updates {
                    Arc::into_inner(updatefn.0).unwrap()()
                }
            }
            ReloadConfig => todo!(),
            OutputEvent { event, wl_output } => match *event {
                wayland::OutputEvent::Created(info_maybe) => {
                    let first_output = self.outputs.is_empty();
                    self.outputs.insert(wl_output, info_maybe);
                    if !self.outputs_ready && first_output {
                        self.outputs_ready = true;
                        return Task::stream(stream::channel(1, |_| async {
                            sleep(Duration::from_millis(500)).await;
                        }))
                        .chain(self.flush_opening_queue());
                    }
                }
                wayland::OutputEvent::InfoUpdate(info) => {
                    self.outputs.insert(wl_output, Some(info));
                }
                wayland::OutputEvent::Removed => {
                    self.outputs.remove(&wl_output);
                }
            },
            IpcCommand { request, responder } => {
                use ipc::WindowRequest::*;

                info!("Received ipc request: {request:?}");

                let mut task = Task::none();
                let response = match request {
                    IpcRequest::ListWindows => IpcResponse::WindowList(
                        self.windows
                            .values()
                            .map(|w| (w.naive_id(), w.clone()))
                            .collect(),
                    ),
                    IpcRequest::Close => {
                        info!("closing the daemon");
                        daemon::exit_cleanup(&self.socket_path, &self.pid_path);
                        task = iced::exit();
                        IpcResponse::Closing
                    }
                    IpcRequest::Window { cmd, id } => {
                        use ipc::WindowResponse;
                        match cmd {
                            Open(opts) => {
                                info!("Opening new window");
                                let naive_id;
                                (task, naive_id) = self.open_window(opts);
                                IpcResponse::Window {
                                    id: vec![naive_id],
                                    event: WindowResponse::Opened,
                                }
                            }
                            _ => {
                                if self.windows.is_empty() {
                                    IpcResponse::error("Command failed because no windows are open")
                                } else if id.is_some_and(|id| !self.window_ids.contains_key(&id)) {
                                    IpcResponse::error("No window with the specified ID is open")
                                } else {
                                    let (naive_id, window_id) = id
                                        .map_or_else(
                                            || self.window_ids.iter().last(),
                                            |id| self.window_ids.get_key_value(&id),
                                        )
                                        .map(|(k, v)| (*k, *v))
                                        .expect("Previously checked");
                                    match cmd {
                                        Close { all } => {
                                            if all {
                                                let (naive_ids, window_ids): (Vec<usize>, Vec<Id>) =
                                                    self.window_ids.drain().collect();
                                                self.windows.clear();
                                                info!("Closing all open windows ({} windows are open)", naive_ids.len());
                                                for window_id in window_ids {
                                                    task = task
                                                        .chain(destroy_layer_surface(window_id));
                                                }
                                                IpcResponse::Window {
                                                    id: naive_ids,
                                                    event: WindowResponse::Closed,
                                                }
                                            } else {
                                                self.window_ids.remove(&naive_id);
                                                self.windows.remove(&window_id);
                                                info!("Closing window with id {naive_id}");
                                                task = destroy_layer_surface(window_id);
                                                IpcResponse::Window {
                                                    id: vec![naive_id],
                                                    event: WindowResponse::Closed,
                                                }
                                            }
                                        }
                                        Reopen { all } => {
                                            if all {
                                                let (naive_ids, window_ids): (Vec<usize>, Vec<Id>) =
                                                    self.window_ids.iter().collect();
                                                info!("Reopening all open windows ({} windows are open)", naive_ids.len());
                                                for window_id in window_ids {
                                                    task = task
                                                        .chain(destroy_layer_surface(window_id))
                                                        .chain(
                                                            self.windows[&window_id]
                                                                .open(&self.outputs),
                                                        );
                                                }
                                                IpcResponse::Window {
                                                    id: naive_ids,
                                                    event: WindowResponse::Reopened,
                                                }
                                            } else {
                                                info!("Reopening window with id {naive_id}");
                                                task = destroy_layer_surface(window_id).chain(
                                                    self.windows[&window_id].open(&self.outputs),
                                                );
                                                IpcResponse::Window {
                                                    id: vec![naive_id],
                                                    event: WindowResponse::Reopened,
                                                }
                                            }
                                        }
                                        Open(_) => unreachable!(),
                                    }
                                }
                            }
                        }
                    }
                };
                if responder.send(response).is_err() {
                    error!("IPC response channel closed");
                }
                return task;
            }
        }
        Task::none()
    }

    pub fn view(&self, id: Id) -> Element<Message> {
        match self.windows.get(&id) {
            Some(window) => window.view(),
            None => "Invalid window ID".into(),
        }
    }

    fn open_window(&mut self, opts: WindowOpenOptions) -> (Task<Message>, usize) {
        let naive_id = self.id_count;
        let config = match self.configurations.get(&opts.name) {
            Some(config) => config.clone(),
            None => {
                error!("No such configuration: {}", opts.name);
                ConfigOptions::default()
            }
        };
        let window = Window::new(naive_id, opts, config);
        let mut task = Task::none();
        if self.outputs_ready {
            task = window.open(&self.outputs);
        } else {
            self.opening_queue.push_back(window.window_id());
        }
        self.window_ids.insert(naive_id, window.window_id());
        self.windows.insert(window.window_id(), window);
        self.id_count += 1;
        (task, naive_id)
    }

    fn flush_opening_queue(&mut self) -> Task<Message> {
        let mut task = Task::none();
        while let Some(id) = self.opening_queue.pop_front() {
            task = task.chain(self.windows[&id].open(&self.outputs));
        }
        task
    }
}
