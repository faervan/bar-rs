use chrono::Local;
use iced::{widget::{row, text}, Length::Fill};

use crate::NERD_FONT;

use super::Module;

#[derive(Debug)]
pub struct TimeMod;

impl Module for TimeMod {
    fn id(&self) -> String {
        "time".to_string()
    }

    fn view(&self) -> iced::Element<crate::Message> {
        let time = Local::now();
        row![
            text!("")
                .center().height(Fill).size(20).font(NERD_FONT),
            text![
                " {}", time.format("%a, %d. %b  ")
            ].center().height(Fill),
            text!("")
                .center().height(Fill).size(25).font(NERD_FONT),
            text![
                " {}", time.format("%H:%M")
            ].center().height(Fill),
        ].spacing(10).into()
    }
    
}
