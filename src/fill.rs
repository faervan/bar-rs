use iced::{
    widget::{text::Rich, Container, Text},
    Length::Fill,
};

use crate::config::anchor::BarAnchor;

pub trait FillExt {
    fn fill(self, anchor: &BarAnchor) -> Self;
}

impl FillExt for Text<'_> {
    fn fill(self, anchor: &BarAnchor) -> Self {
        match anchor.vertical() {
            true => self.width(Fill),
            false => self.height(Fill),
        }
        .center()
    }
}

impl<Link> FillExt for Rich<'_, Link>
where
    Link: Clone,
{
    fn fill(self, anchor: &BarAnchor) -> Self {
        match anchor.vertical() {
            true => self.center(),
            false => self.height(Fill),
        }
    }
}

impl<Message> FillExt for Container<'_, Message> {
    fn fill(self, anchor: &BarAnchor) -> Self {
        match anchor.vertical() {
            true => self.height(Fill),
            false => self.width(Fill),
        }
    }
}
