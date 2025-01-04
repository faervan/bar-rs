use std::process::Command;

use bar_rs_derive::Builder;
use iced::{widget::{row, text}, Length::Fill};

use crate::NERD_FONT;

use super::Module;

#[derive(Debug, Builder)]
pub struct MemoryMod;

impl Module for MemoryMod {
    fn id(&self) -> String {
        "memory".to_string()
    }

    fn view(&self) -> iced::Element<crate::Message> {
        let usage = Command::new("sh")
            .arg("-c")
            .arg("free | grep Mem | awk '{printf \"%.0f\", $3/$2 * 100.0}'")
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
            .unwrap_or_else(|e| {
                eprintln!("Failed to get memory usage. err: {e}");
                "0".to_string()
            })
            .parse()
            .unwrap_or_else(|e| {
                eprintln!("Failed to parse memory usage (output from free), e: {e}");
                999
            });

        row![
            text!("󰍛")
                .center().height(Fill).size(20).font(NERD_FONT),
            text![
                "{}%", usage
            ].center().height(Fill)
        ].spacing(10).into()
    }
}
