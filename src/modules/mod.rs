use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

use battery::BatteryMod;
use cpu::CpuMod;
use date::DateMod;
use downcast_rs::{impl_downcast, Downcast};
use hyprland::{window::HyprWindowMod, workspaces::HyprWorkspaceMod};
use iced::{Element, Subscription};
use media::MediaMod;
use memory::MemoryMod;
use time::TimeMod;
use volume::VolumeMod;
use wayfire::{WayfireWindowMod, WayfireWorkspaceMod};

use crate::{
    config::{anchor::BarAnchor, module_config::LocalModuleConfig},
    listeners::Listener,
    registry::Registry,
    Message,
};

pub mod battery;
pub mod cpu;
pub mod date;
pub mod hyprland;
pub mod media;
pub mod memory;
pub mod sys_tray;
pub mod time;
pub mod volume;
pub mod wayfire;

pub trait Module: Any + Debug + Send + Sync + Downcast {
    /// The name used to enable the Module in the config
    fn name(&self) -> String;
    /// What the module actually shows.
    /// See [widgets-and-elements](https://docs.iced.rs/iced/#widgets-and-elements).
    fn view(&self, config: &LocalModuleConfig, anchor: &BarAnchor) -> Element<Message>;
    /// The module may optionally have a subscription listening for external events.
    /// See [passive-subscriptions](https://docs.iced.rs/iced/#passive-subscriptions).
    fn subscription(&self) -> Option<Subscription<Message>> {
        None
    }
    /// Modules may require shared subscriptions. Add `require_listener::<SomeListener>()`
    /// for every [Listener] this module requires.
    fn requires(&self) -> Vec<TypeId> {
        vec![]
    }
    #[allow(unused_variables)]
    /// Read configuration options from the config section of this module
    fn read_config(&mut self, config: &HashMap<String, Option<String>>) {}
}
impl_downcast!(Module);

pub fn require_listener<T>() -> TypeId
where
    T: Listener,
{
    TypeId::of::<T>()
}

pub fn register_modules(registry: &mut Registry) {
    registry.register_module::<CpuMod>();
    registry.register_module::<MemoryMod>();
    registry.register_module::<BatteryMod>();
    registry.register_module::<VolumeMod>();
    registry.register_module::<MediaMod>();
    registry.register_module::<DateMod>();
    registry.register_module::<TimeMod>();
    registry.register_module::<HyprWindowMod>();
    registry.register_module::<HyprWorkspaceMod>();
    registry.register_module::<WayfireWorkspaceMod>();
    registry.register_module::<WayfireWindowMod>();
}
