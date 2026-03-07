use std::{any::Any, fmt::Debug};

use downcast_rs::{Downcast, impl_downcast};
use hyprland::HyprListener;
use iced::Subscription;
use niri::NiriListener;
use reload::ReloadListener;
use wayfire::WayfireListener;

use crate::{Message, config::ConfigEntry, registry::Registry};

pub mod hyprland;
pub mod niri;
mod reload;
pub mod wayfire;

pub trait Listener: Any + Debug + Send + Sync + Downcast {
    fn config(&self) -> Vec<ConfigEntry> {
        vec![]
    }
    fn subscription(&self) -> Subscription<Message>;
}
impl_downcast!(Listener);

pub fn register_listeners(registry: &mut Registry) {
    registry.register_listener::<HyprListener>();
    registry.register_listener::<WayfireListener>();
    registry.register_listener::<NiriListener>();
    registry.register_listener::<ReloadListener>();
}
