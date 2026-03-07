use iced::{
    Alignment::Center,
    Length::Fill,
    widget::{
        Container, Text,
        text::{Catalog, Rich},
    },
};

use crate::config::anchor::BarAnchor;

pub trait FillExt {
    fn fill(self, anchor: &BarAnchor) -> Self;
    fn fillx(self, vertical: bool) -> Self;
    fn fill_maybe(self, fill: bool) -> Self;
}

impl FillExt for Text<'_> {
    fn fill(self, anchor: &BarAnchor) -> Self {
        self.fillx(anchor.vertical())
    }
    fn fillx(self, vertical: bool) -> Self {
        match vertical {
            true => self.width(Fill),
            false => self.height(Fill),
        }
        .center()
    }
    fn fill_maybe(self, fill: bool) -> Self {
        match fill {
            true => self.height(Fill).width(Fill),
            false => self,
        }
    }
}

impl<'a, Link, Message, Theme, Renderer> FillExt for Rich<'a, Link, Message, Theme, Renderer>
where
    Link: Clone,
    Theme: Catalog,
    Renderer: iced::core::text::Renderer + 'a,
{
    fn fill(self, anchor: &BarAnchor) -> Self {
        self.fillx(anchor.vertical())
    }
    fn fillx(self, vertical: bool) -> Self {
        match vertical {
            true => self.center(),
            false => self.height(Fill).align_y(Center),
        }
    }
    fn fill_maybe(self, fill: bool) -> Self {
        match fill {
            true => self.height(Fill).width(Fill),
            false => self,
        }
    }
}

impl<Message> FillExt for Container<'_, Message> {
    fn fill(self, anchor: &BarAnchor) -> Self {
        self.fillx(anchor.vertical())
    }
    fn fillx(self, vertical: bool) -> Self {
        match vertical {
            true => self.width(Fill),
            false => self.height(Fill),
        }
    }
    fn fill_maybe(self, fill: bool) -> Self {
        match fill {
            true => self.height(Fill).width(Fill),
            false => self,
        }
    }
}

impl<Message> FillExt for iced::widget::button::Button<'_, Message>
where
    Message: Clone,
{
    fn fill(self, anchor: &BarAnchor) -> Self {
        self.fillx(anchor.vertical())
    }
    fn fillx(self, vertical: bool) -> Self {
        match vertical {
            true => self.width(Fill),
            false => self.height(Fill),
        }
    }
    fn fill_maybe(self, fill: bool) -> Self {
        match fill {
            true => self.height(Fill).width(Fill),
            false => self,
        }
    }
}

impl<Message> FillExt for crate::button::Button<'_, Message>
where
    Message: Clone,
{
    fn fill(self, anchor: &BarAnchor) -> Self {
        self.fillx(anchor.vertical())
    }
    fn fillx(self, vertical: bool) -> Self {
        match vertical {
            true => self.width(Fill),
            false => self.height(Fill),
        }
    }
    fn fill_maybe(self, fill: bool) -> Self {
        match fill {
            true => self.height(Fill).width(Fill),
            false => self,
        }
    }
}
