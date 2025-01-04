use std::{any::TypeId, time::Duration};

use bar_rs_derive::Builder;
use hyprland::{data::{Workspace, Workspaces}, shared::{HyprData, HyprDataActive, HyprDataVec}};
use iced::{widget::{row, text::{Rich, Span}}, Background, Border, Color, Length::Fill, Padding};
use tokio::time::sleep;

use crate::{listeners::hyprland::HyprListener, modules::{require_listener, Module}, Message, NERD_FONT};

#[derive(Debug, Default, Builder)]
pub struct HyprWorkspaceMod {
    pub active: usize,
    // (Name, Fullscreen state)
    pub open: Vec<(String, bool)>,
}

impl Module for HyprWorkspaceMod {
    fn id(&self) -> String {
        "hyprland.workspaces".to_string()
    }

    fn view(&self) -> iced::Element<Message> {
        row(
            self.open
                .iter()
                .enumerate()
                .map(|(id, (ws, _))| {
                    let mut span = Span::new(ws)
                        .size(20)
                        .padding(Padding {top: -3., bottom: 0., right: 10., left: 5.})
                        .font(NERD_FONT);
                    if id == self.active {
                        span = span
                            .background(Background::Color(Color::WHITE).scale_alpha(0.5))
                            .border(Border::default().rounded(8))
                            .color(Color::BLACK);
                    }
                    Rich::with_spans([span])
                        .center()
                        .height(Fill)
                        .into()
                })
        ).spacing(15).into()
    }

    fn requires(&self) -> Vec<TypeId> {
        vec![
            require_listener::<HyprListener>()
        ]
    }
}

impl From<(Workspaces, usize)> for HyprWorkspaceMod {
    fn from(value: (Workspaces, usize)) -> Self {
        let mut workspaces = Self::default();
        let mut list = value.0.to_vec();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list.iter()
            .for_each(
                |ws| workspaces.open.push((
                        ws.name.clone(),
                        ws.fullscreen
                    ))
            );
        workspaces.active = list.iter()
            .position(|ws| ws.id as usize == value.1)
            .unwrap_or(0);
        workspaces
    }
}

pub async fn get_workspaces(active: Option<i32>) -> HyprWorkspaceMod {
    // Sleep a bit, to reduce the probability that a nonexisting ws is still reported active
    sleep(Duration::from_millis(10)).await;
    let Ok(workspaces) = Workspaces::get_async().await else {
        eprintln!("[hyprland.workspaces] Failed to get Workspaces!");
        return HyprWorkspaceMod::default();
    };
    HyprWorkspaceMod::from((
        workspaces,
        active.unwrap_or(
            Workspace::get_active_async().await
                .map(|ws| ws.id)
                .unwrap_or(0)
        ) as usize,
    ))
}
