use std::{collections::HashMap, fmt::Debug, sync::Arc};

use battery::BatteryMod;
use cpu::CpuMod;
use hyprland::{HyprWindowMod, HyprWorkspaceMod};
use iced::{Element, Subscription};
use media::MediaMod;
use memory::MemoryMod;
use time::TimeMod;
use volume::VolumeMod;

use crate::Message;

pub mod time;
pub mod cpu;
pub mod memory;
pub mod battery;
pub mod media;
pub mod volume;
pub mod hyprland;
pub mod sys_tray;

pub trait Module: Send + Sync + Debug {
    fn id(&self) -> String;
    fn view(&self) -> Element<Message>;
    fn subscription(&self) -> Option<Subscription<Message>> {
        None
    }
}

pub fn all_modules() -> HashMap<String, Arc<dyn Module>> {
    let modules: Vec<Arc<dyn Module>> = vec![
        Arc::new(HyprWorkspaceMod::default()),
        Arc::new(HyprWindowMod::default()),
        Arc::new(TimeMod),
        Arc::new(MediaMod::default()),
        Arc::new(VolumeMod::default()),
        Arc::new(BatteryMod::default()),
        Arc::new(CpuMod::default()),
        Arc::new(MemoryMod),
    ];
    modules.into_iter()
        .map(|module| (
            module.id(),
            module
        ))
        .collect()
}
