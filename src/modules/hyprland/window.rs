use std::any::TypeId;

use bar_rs_derive::Builder;
use iced::{futures::{channel::mpsc::Sender, SinkExt}, widget::{row, text}, Length::Fill};

use crate::{listeners::hyprland::HyprListener, modules::{require_listener, Module}, Message};

#[derive(Debug, Default, Builder)]
pub struct HyprWindowMod(Option<String>);

impl Module for HyprWindowMod {
    fn id(&self) -> String {
        "hyprland.window".to_string()
    }

    fn view(&self) -> iced::Element<Message> {
        row![
            text![
                "{}",
                self.0.as_ref()
                    .unwrap_or(&"".to_string())
            ].center().height(Fill)
        ].into()
    }

    fn requires(&self) -> Vec<TypeId> {
        vec![
            require_listener::<HyprListener>()
        ]
    }
}

pub async fn update_window(sender: &mut Sender<Message>, window: Option<String>) {
    sender.send(Message::update(Box::new(
            move |reg| reg.get_module_mut::<HyprWindowMod>().0 = window
        )))
        .await
        .unwrap_or_else(|err| {
            eprintln!("Trying to send workspaces failed with err: {err}");
        });
}
