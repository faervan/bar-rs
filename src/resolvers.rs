use std::{any::TypeId, env};

use crate::{
    config::Config,
    /*modules::{
        hyprland::{window::HyprWindowMod, workspaces::HyprWorkspaceMod},
        niri::{NiriWindowMod, NiriWorkspaceMod},
        wayfire::{WayfireWindowMod, WayfireWorkspaceMod},
    },*/
    registry::Registry,
};

pub fn register_resolvers(registry: &mut Registry) {
    //registry.add_resolver("window", window);
    //registry.add_resolver("workspaces", workspaces);
}

/*fn window(_config: Option<&Config>) -> Option<TypeId> {
    env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .and_then(|var| match var.as_str() {
            "niri" => Some(TypeId::of::<NiriWindowMod>()),
            "Hyprland" => Some(TypeId::of::<HyprWindowMod>()),
            "Wayfire:wlroots" => Some(TypeId::of::<WayfireWindowMod>()),
            _ => None,
        })
}

fn workspaces(_config: Option<&Config>) -> Option<TypeId> {
    env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .and_then(|var| match var.as_str() {
            "niri" => Some(TypeId::of::<NiriWorkspaceMod>()),
            "Hyprland" => Some(TypeId::of::<HyprWorkspaceMod>()),
            "Wayfire:wlroots" => Some(TypeId::of::<WayfireWorkspaceMod>()),
            _ => None,
        })
}*/
