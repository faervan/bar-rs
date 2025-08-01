use core::config::GlobalConfig;
use std::{fmt::Debug, sync::Arc};

use iced::{
    event::wayland::OutputEvent,
    futures::{channel::mpsc, SinkExt as _},
};
use ipc::{IpcRequest, IpcResponse};
use smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput;
use tokio::sync::oneshot;

use crate::{state::State, subscription::Subscription};

pub struct UpdateFn(pub Arc<Box<dyn Fn() + Send + Sync>>);

impl Debug for UpdateFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UpdateFn(Arc<dyn FnOnce()>)")
    }
}

pub struct ReadFn<T>(Arc<Box<dyn FnOnce(&T) + Send + Sync>>);
impl<T> Debug for ReadFn<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReadFn<{}>", std::any::type_name::<T>())
    }
}
impl<T> ReadFn<T> {
    pub fn execute(self, t: &T) {
        Arc::into_inner(self.0).unwrap()(t)
    }
}

#[derive(Debug)]
pub enum Message {
    ReadState(ReadFn<State>),
    Update(Vec<UpdateFn>),
    ReloadConfig,
    OutputEvent {
        event: Box<OutputEvent>,
        wl_output: WlOutput,
    },
    IpcCommand {
        request: IpcRequest,
        responder: oneshot::Sender<IpcResponse>,
    },
}

impl Message {
    pub fn read_state<F>(f: F) -> Self
    where
        F: FnOnce(&State) + Send + Sync + 'static,
    {
        Self::ReadState(ReadFn(Arc::new(Box::new(f))))
    }
}

pub trait MessageSenderExt {
    async fn read_config(&mut self) -> Arc<GlobalConfig>;
    async fn read_subscriptions(&mut self) -> Vec<Subscription>;
}

impl MessageSenderExt for mpsc::Sender<Message> {
    async fn read_config(&mut self) -> Arc<GlobalConfig> {
        let (sx, rx) = oneshot::channel();
        self.send(Message::read_state(move |state| {
            sx.send(state.config.clone()).unwrap()
        }))
        .await
        .unwrap();
        rx.await.unwrap()
    }
    async fn read_subscriptions(&mut self) -> Vec<Subscription> {
        // TODO!
        vec![]
    }
}
