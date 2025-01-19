use iced::widget::container;
use iced::widget::tooltip::Position;
use iced::{
    widget::{container::Style, Tooltip},
    Element,
};
use iced::{Background, Border, Color, Theme};

pub fn tooltip<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    tooltip: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Tooltip<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced::core::text::Renderer + 'a,
{
    iced::widget::tooltip(
        content,
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
        Position::FollowCursor,
    )
}
