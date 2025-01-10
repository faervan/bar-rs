use iced::{
    widget::{column, row, Column, Container, Row},
    Alignment, Element, Padding, Pixels,
};

use crate::config::anchor::BarAnchor;

pub trait DynamicAlign {
    fn align(self, anchor: &BarAnchor, alignment: Alignment) -> Self;
}

impl<Message> DynamicAlign for Container<'_, Message> {
    fn align(self, anchor: &BarAnchor, alignment: Alignment) -> Self {
        match anchor.vertical() {
            true => self.align_y(alignment),
            false => self.align_x(alignment),
        }
    }
}

pub enum List<'a, Message, Theme, Renderer> {
    Row(Row<'a, Message, Theme, Renderer>),
    Column(Column<'a, Message, Theme, Renderer>),
}

impl<'a, Message, Theme, Renderer> List<'a, Message, Theme, Renderer>
where
    Renderer: iced::core::Renderer,
{
    pub fn new(anchor: &BarAnchor) -> List<'a, Message, Theme, Renderer> {
        match anchor.vertical() {
            true => List::Column(Column::new()),
            false => List::Row(Row::new()),
        }
    }

    pub fn with_children(
        anchor: &BarAnchor,
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        match anchor.vertical() {
            true => List::Column(Column::with_children(children)),
            false => List::Row(Row::with_children(children)),
        }
    }

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

macro_rules! list {
    ($anchor:expr) => (
        $crate::list::List::new($anchor)
    );
    ($anchor:expr, $($x:expr),+ $(,)?) => (
        $crate::list::List::with_children($anchor, [$(iced::core::Element::from($x)),+])
    );
}
