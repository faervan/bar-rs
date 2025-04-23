use std::{fmt::Debug, sync::Arc};

use iced::futures::{channel::mpsc, SinkExt};
use tokio::sync::oneshot;

use crate::{config::Config, subscription::Subscription};

pub struct UpdateFn(pub Arc<Box<dyn Fn() + Send + Sync>>);

impl Debug for UpdateFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UpdateFn(Arc<dyn FnOnce()>)")
    }
}

#[derive(Debug)]
pub enum Message {
    FetchSubscriptions(oneshot::Sender<Vec<Subscription>>),
    FetchConfig(oneshot::Sender<Arc<Config>>),
    Update(Vec<UpdateFn>),
}

pub async fn get_config(sender: &mut mpsc::Sender<Message>) -> Arc<Config> {
    let (sx, rx) = oneshot::channel();
    sender.send(Message::FetchConfig(sx)).await.unwrap();
    rx.await.unwrap()
}
