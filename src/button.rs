use iced::{Element, Length, Padding, Rectangle};


pub struct Button<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> where
    Renderer: iced::core::Renderer,
    Theme: Catalog, {
    content: Element<'a, Message, Theme, Renderer>,
    on_event: Option<Box<dyn Fn(iced::Event, iced::core::Layout, iced::mouse::Cursor, &mut dyn iced::core::Clipboard, &Rectangle) -> Message>>,
    width: Length,
    height: Length,
    padding: Padding,
    class: iced::Theme::Class<'a>,
}
