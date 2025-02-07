/// Literally 100% copypasta from https://github.com/iced-rs/iced/blob/master/widget/src/button.rs
use iced::core::widget::tree;
use iced::core::{keyboard, overlay, renderer, touch};
use iced::{
    core::{
        event, layout, mouse,
        widget::{Operation, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    id::Id,
    widget::button::{Catalog, Status, Style, StyleFn},
    Element, Event, Length, Padding, Rectangle, Size,
};
use iced::{Background, Color, Vector};

type EventHandlerFn<'a, Message> = Box<
    dyn Fn(
            iced::Event,
            iced::core::Layout,
            iced::mouse::Cursor,
            &mut dyn iced::core::Clipboard,
            &Rectangle,
        ) -> Message
        + 'a,
>;

enum ButtonEventHandler<'a, Message>
where
    Message: Clone,
{
    Message(Message),
    F(EventHandlerFn<'a, Message>),
    FMaybe(EventHandlerFn<'a, Option<Message>>),
}

impl<Message> ButtonEventHandler<'_, Message>
where
    Message: Clone,
{
    fn get(
        &self,
        event: iced::Event,
        layout: iced::core::Layout,
        cursor: iced::mouse::Cursor,
        clipboard: &mut dyn iced::core::Clipboard,
        viewport: &Rectangle,
    ) -> Option<Message> {
        match self {
            ButtonEventHandler::Message(msg) => Some(msg.clone()),
            ButtonEventHandler::F(f) => Some(f(event, layout, cursor, clipboard, viewport)),
            ButtonEventHandler::FMaybe(f) => f(event, layout, cursor, clipboard, viewport),
        }
    }
}

pub struct Button<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::core::Renderer,
    Theme: Catalog,
    Message: Clone,
{
    content: Element<'a, Message, Theme, Renderer>,
    on_event: Option<ButtonEventHandler<'a, Message>>,
    id: Id,
    width: Length,
    height: Length,
    padding: Padding,
    clip: bool,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Button<'a, Message, Theme, Renderer>
where
    Renderer: iced::core::Renderer,
    Theme: Catalog,
    Message: Clone,
{
    /// Creates a new [`Button`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Button {
            content,
            id: Id::unique(),
            on_event: None,
            width: size.width.fluid(),
            height: size.height.fluid(),
            padding: Padding::ZERO,
            clip: false,
            class: Theme::default(),
        }
    }

    /// Defines the on_event action of the [`Button`]
    pub fn on_event(mut self, msg: Message) -> Self {
        self.on_event = Some(ButtonEventHandler::Message(msg));
        self
    }

    /// Defines the on_event action of the [`Button`], if Some
    pub fn on_event_maybe(mut self, msg: Option<Message>) -> Self {
        if let Some(msg) = msg {
            self.on_event = Some(ButtonEventHandler::Message(msg));
        }
        self
    }

    /// Determines the on_event action of the [`Button`] using a closure
    pub fn on_event_with<F>(mut self, f: F) -> Self
    where
        F: Fn(
                iced::Event,
                iced::core::Layout,
                iced::mouse::Cursor,
                &mut dyn iced::core::Clipboard,
                &Rectangle,
            ) -> Message
            + 'a,
    {
        self.on_event = Some(ButtonEventHandler::F(Box::new(f)));
        self
    }

    /// Determines the on_event action of the [`Button`] with a closure, if Some
    pub fn on_event_maybe_with<F>(self, f: Option<F>) -> Self
    where
        F: Fn(
                iced::Event,
                iced::core::Layout,
                iced::mouse::Cursor,
                &mut dyn iced::core::Clipboard,
                &Rectangle,
            ) -> Message
            + 'a,
    {
        if let Some(f) = f {
            self.on_event_with(f)
        } else {
            self
        }
    }

    /// Determines the on_event action of the [`Button`] using a closure which might return a Message
    pub fn on_event_try<F>(mut self, f: F) -> Self
    where
        F: Fn(
                iced::Event,
                iced::core::Layout,
                iced::mouse::Cursor,
                &mut dyn iced::core::Clipboard,
                &Rectangle,
            ) -> Option<Message>
            + 'a,
    {
        self.on_event = Some(ButtonEventHandler::FMaybe(Box::new(f)));
        self
    }

    /// Sets the width of the [`Button`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Button`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the [`Button`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets whether the contents of the [`Button`] should be clipped on
    /// overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Sets the style of the [`Button`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the [`Id`] of the [`Button`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    is_hovered: bool,
    is_pressed: bool,
    is_focused: bool,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Button<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::core::Renderer,
    Theme: Catalog,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) {
            return event::Status::Captured;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle))
            | Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if self.on_event.is_some() {
                    let bounds = layout.bounds();

                    if cursor.is_over(bounds) {
                        let state = tree.state.downcast_mut::<State>();

                        state.is_pressed = true;

                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle))
            | Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(on_press) = self.on_event.as_ref() {
                    let state = tree.state.downcast_mut::<State>();

                    if state.is_pressed {
                        state.is_pressed = false;

                        let bounds = layout.bounds();

                        if cursor.is_over(bounds) {
                            if let Some(msg) =
                                on_press.get(event, layout, cursor, clipboard, viewport)
                            {
                                shell.publish(msg);
                            }
                        }

                        return event::Status::Captured;
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { ref key, .. }) => {
                if let Some(on_press) = self.on_event.as_ref() {
                    let state = tree.state.downcast_mut::<State>();
                    if state.is_focused
                        && matches!(key, keyboard::Key::Named(keyboard::key::Named::Enter))
                    {
                        state.is_pressed = true;
                        if let Some(msg) = on_press.get(event, layout, cursor, clipboard, viewport)
                        {
                            shell.publish(msg);
                        }
                        return event::Status::Captured;
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. })
            | Event::Mouse(mouse::Event::CursorLeft) => {
                let state = tree.state.downcast_mut::<State>();
                state.is_hovered = false;
                state.is_pressed = false;
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let is_mouse_over = cursor.is_over(bounds);

        let status = if self.on_event.is_none() {
            Status::Disabled
        } else if is_mouse_over {
            let state = tree.state.downcast_ref::<State>();

            if state.is_pressed {
                Status::Pressed
            } else {
                Status::Hovered
            }
        } else {
            Status::Active
        };

        let style = theme.style(&self.class, status);

        if style.background.is_some() || style.border.width > 0.0 || style.shadow.color.a > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                },
                style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        let viewport = if self.clip {
            bounds.intersection(viewport).unwrap_or(*viewport)
        } else {
            *viewport
        };

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
                icon_color: style.icon_color.unwrap_or(renderer_style.icon_color),
                scale_factor: renderer_style.scale_factor,
            },
            content_layout,
            cursor,
            &viewport,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_mouse_over = cursor.is_over(layout.bounds());

        if is_mouse_over && self.on_event.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }

    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

impl<'a, Message, Theme, Renderer> From<Button<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: iced::core::Renderer + 'a,
{
    fn from(button: Button<'a, Message, Theme, Renderer>) -> Self {
        Self::new(button)
    }
}

pub fn button<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: iced::core::Renderer,
    Message: Clone,
{
    Button::new(content)
}
