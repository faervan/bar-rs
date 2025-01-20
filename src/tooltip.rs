use iced::widget::container;
use iced::widget::tooltip::Position;
use iced::{
    widget::{container::Style, Tooltip},
    Element,
};
use iced::{Background, Border, Color, Theme};

pub trait ElementExt<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced::core::text::Renderer + 'a,
{
    fn tooltip(
        self,
        tooltip: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Tooltip<'a, Message, Theme, Renderer>;
    fn tooltip_maybe(
        self,
        tooltip: Option<impl Into<Element<'a, Message, Theme, Renderer>>>,
    ) -> Element<'a, Message, Theme, Renderer>;
}

impl<'a, Message, Renderer, Elem> ElementExt<'a, Message, Renderer> for Elem
where
    Message: 'a,
    Renderer: iced::core::text::Renderer + 'a,
    Elem: Into<Element<'a, Message, Theme, Renderer>>,
{
    fn tooltip(
        self,
        tooltip: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Tooltip<'a, Message, Theme, Renderer> {
        iced::widget::tooltip(
            self,
            container(tooltip).padding([2, 10]).style(|_| Style {
                text_color: Some(Color::WHITE),
                background: Some(Background::Color(Color::BLACK)),
                border: Border {
                    color: Color::WHITE,
                    width: 1.,
                    radius: 5_f32.into(),
                },
                ..Default::default()
            }),
            Position::Bottom,
        )
    }
    fn tooltip_maybe(
        self,
        tooltip: Option<impl Into<Element<'a, Message, Theme, Renderer>>>,
    ) -> Element<'a, Message, Theme, Renderer> {
        match tooltip {
            Some(t) => self.tooltip(t).into(),
            None => self.into(),
        }
    }
}
