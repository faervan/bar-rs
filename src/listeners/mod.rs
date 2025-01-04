use std::{any::Any, fmt::Debug};

use downcast_rs::{impl_downcast, Downcast};
use hyprland::HyprListener;
use iced::Subscription;
use reload::ReloadListener;

use crate::{config::ConfigEntry, registry::Registry, Message};

pub mod hyprland;
mod reload;

pub trait Listener: Any + Debug + Send + Sync + Downcast {
    fn config(&self) -> Vec<ConfigEntry> {
        vec![]
    }
    fn subscription(&self) -> Subscription<Message>;
}
impl_downcast!(Listener);

pub fn register_listeners(registry: &mut Registry) {
    registry.register_listener::<HyprListener>();
    registry.register_listener::<ReloadListener>();
}
