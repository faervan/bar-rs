use std::{fmt::Debug, sync::Arc};

use crate::{
    config::GlobalConfig,
    ipc::{IpcRequest, IpcResponse},
};
use iced::{
    event::wayland::OutputEvent,
    futures::{channel::mpsc, Sink},
};
use smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput;
use tokio::sync::oneshot;

use crate::{state::State, subscription::Subscription};

pub struct UpdateFn(pub Arc<Box<dyn Fn() + Send + Sync>>);

impl Debug for UpdateFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UpdateFn(Arc<dyn FnOnce()>)")
    }
}

#[allow(clippy::type_complexity)]
/// Capturing closure that is executed with read access to [T]
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
    OutputsReady,
    IpcCommand {
        request: IpcRequest,
        responder: oneshot::Sender<IpcResponse>,
    },
}

impl Message {
    /// Can be used to capture a [oneshot::Sender] and return a value based on the current [State].
    /// See [MessageSenderExt]
    pub fn read_state<F>(f: F) -> Self
    where
        F: FnOnce(&State) + Send + Sync + 'static,
    {
        Self::ReadState(ReadFn(Arc::new(Box::new(f))))
    }
}

pub trait MessageSenderExt<M>
where
    Self: Sink<M> + Unpin,
    <Self as Sink<M>>::Error: Debug,
{
    async fn send(&mut self, msg: M) {
        iced::futures::SinkExt::send(self, msg)
            .await
            .unwrap_or_else(|e| log::error!("Internal error: Message could not be send: {e:#?}"));
    }
    /// Execute the given closure with read access to the [State], then return its output
    async fn read_with<OUT, F>(&mut self, f: F) -> OUT
    where
        F: FnOnce(&State) -> OUT + Send + Sync + 'static,
        OUT: Debug + Send + Sync + 'static;
    async fn read_config(&mut self) -> Arc<GlobalConfig>;
    async fn read_subscriptions(&mut self) -> Vec<Subscription>;
}

impl MessageSenderExt<Message> for mpsc::Sender<Message> {
    async fn read_with<OUT, F>(&mut self, f: F) -> OUT
    where
        F: FnOnce(&State) -> OUT + Send + Sync + 'static,
        OUT: Debug + Send + Sync + 'static,
    {
        let (sx, rx) = oneshot::channel();
        self.send(Message::read_state(move |state| sx.send(f(state)).unwrap()))
            .await;
        rx.await.unwrap()
    }
    async fn read_config(&mut self) -> Arc<GlobalConfig> {
        self.read_with(|state| state.config.clone()).await
    }
    async fn read_subscriptions(&mut self) -> Vec<Subscription> {
        // TODO!
        self.read_with(|_state| {
            let subs = vec![];
            subs
        })
        .await
    }
}
