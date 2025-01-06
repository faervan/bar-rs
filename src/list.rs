use iced::{
    widget::{column, row, Column, Row},
    Element, Padding, Pixels,
};

use crate::config::anchor::BarAnchor;

pub enum List<'a, Message, Theme, Renderer> {
    Row(Row<'a, Message, Theme, Renderer>),
    Column(Column<'a, Message, Theme, Renderer>),
}

impl<'a, Message, Theme, Renderer> List<'a, Message, Theme, Renderer>
where
    Renderer: iced::core::Renderer,
{
    pub fn spacing(self, amount: impl Into<Pixels>) -> List<'a, Message, Theme, Renderer> {
        match self {
            List::Row(row) => List::Row(row.spacing(amount)),
            List::Column(col) => List::Column(col.spacing(amount)),
        }
    }

    pub fn padding<P>(self, padding: P) -> List<'a, Message, Theme, Renderer>
    where
        P: Into<Padding>,
    {
        match self {
            List::Row(row) => List::Row(row.padding(padding)),
            List::Column(col) => List::Column(col.padding(padding)),
        }
    }
}

pub fn list<'a, Message, Theme, Renderer>(
    anchor: &BarAnchor,
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> List<'a, Message, Theme, Renderer>
where
    Renderer: iced::core::Renderer,
{
    match anchor.vertical() {
        true => List::Column(column(children)),
        false => List::Row(row(children)),
    }
}

impl<'a, Message, Theme, Renderer> From<List<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::core::Renderer + 'a,
{
    fn from(list: List<'a, Message, Theme, Renderer>) -> Self {
        match list {
            List::Row(row) => Self::new(row),
            List::Column(col) => Self::new(col),
        }
    }
}
