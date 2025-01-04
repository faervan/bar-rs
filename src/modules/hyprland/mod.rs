use hyprland::{data::Monitor, keyword::Keyword, shared::HyprDataActive};

use crate::BAR_HEIGHT;

pub mod window;
pub mod workspaces;

pub fn get_monitor_name() -> String {
    Monitor::get_active()
        .map(|m| m.name)
        .unwrap_or("eDP-1".to_string())
}

pub fn reserve_bar_space(monitor: &String) {
    Keyword::set("monitor", format!("{monitor}, addreserved, {BAR_HEIGHT}, 0, 0, 0"))
        .expect("Failed to set reserved space using hyprctl");
}
