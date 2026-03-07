/// Literally 100% copy pasta from https://github.com/iced-rs/iced/blob/master/widget/src/button.rs
use iced::core::widget::tree;
use iced::core::{keyboard, overlay, renderer, touch};
use iced::widget::button::DEFAULT_PADDING;
use iced::{Background, Color, Vector, window};
use iced::{
    Element, Event, Length, Padding, Rectangle, Size,
    core::{
        Clipboard, Layout, Shell, Widget, layout, mouse,
        widget::{Operation, Tree},
    },
    id::Id,
    widget::button::{Catalog, Status, Style, StyleFn},
};

type EventHandlerFn<'a, Message> = Box<
    dyn Fn(
            &iced::Event,
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
    Direct(Message),
    Closure(Box<dyn Fn() -> Message + 'a>),
    F(EventHandlerFn<'a, Message>),
    FMaybe(EventHandlerFn<'a, Option<Message>>),
}

impl<Message> ButtonEventHandler<'_, Message>
where
    Message: Clone,
{
    fn get(
        &self,
        event: &iced::Event,
        layout: iced::core::Layout,
        cursor: iced::mouse::Cursor,
        clipboard: &mut dyn iced::core::Clipboard,
        viewport: &Rectangle,
    ) -> Option<Message> {
        match self {
            ButtonEventHandler::Direct(msg) => Some(msg.clone()),
            ButtonEventHandler::Closure(f) => Some(f()),
            ButtonEventHandler::F(f) => Some(f(event, layout, cursor, clipboard, viewport)),
            ButtonEventHandler::FMaybe(f) => f(event, layout, cursor, clipboard, viewport),
        }
    }
}

pub struct Button<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Message: Clone,
    Renderer: iced::core::Renderer,
    Theme: Catalog,
{
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<ButtonEventHandler<'a, Message>>,
    id: Id,
    width: Length,
    height: Length,
    padding: Padding,
    clip: bool,
    class: Theme::Class<'a>,
    status: Option<Status>,
}

impl<'a, Message, Theme, Renderer> Button<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::core::Renderer,
    Theme: Catalog,
{
    /// Creates a new [`Button`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Button {
            content,
            id: Id::unique(),
            on_press: None,
            width: size.width.fluid(),
            height: size.height.fluid(),
            padding: DEFAULT_PADDING,
            clip: false,
            class: Theme::default(),
            status: None,
        }
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

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// Unless `on_press` is called, the [`Button`] will be disabled.
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(ButtonEventHandler::Direct(on_press));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// This is analogous to [`Button::on_press`], but using a closure to produce
    /// the message.
    ///
    /// This closure will only be called when the [`Button`] is actually pressed and,
    /// therefore, this method is useful to reduce overhead if creating the resulting
    /// message is slow.
    pub fn on_press_with(mut self, on_press: impl Fn() -> Message + 'a) -> Self {
        self.on_press = Some(ButtonEventHandler::Closure(Box::new(on_press)));
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed,
    /// if `Some`.
    ///
    /// If `None`, the [`Button`] will be disabled.
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press = on_press.map(ButtonEventHandler::Direct);
        self
    }

    /// Determines the `on_press` action of the [`Button`] using a closure
    pub fn on_press_with_context<F>(mut self, f: F) -> Self
    where
        F: Fn(
                &iced::Event,
                iced::core::Layout,
                iced::mouse::Cursor,
                &mut dyn iced::core::Clipboard,
                &Rectangle,
            ) -> Message
            + 'a,
    {
        self.on_press = Some(ButtonEventHandler::F(Box::new(f)));
        self
    }

    /// Determines the `on_press` action of the [`Button`] with a closure, if Some
    pub fn on_press_with_context_maybe<F>(self, f: Option<F>) -> Self
    where
        F: Fn(
                &iced::Event,
                iced::core::Layout,
                iced::mouse::Cursor,
                &mut dyn iced::core::Clipboard,
                &Rectangle,
            ) -> Message
            + 'a,
    {
        if let Some(f) = f {
            self.on_press_with_context(f)
        } else {
            self
        }
    }

    /// Determines the `on_press` action of the [`Button`] using a closure which might return a Message
    pub fn on_press_with_context_try<F>(mut self, f: F) -> Self
    where
        F: Fn(
                &iced::Event,
                iced::core::Layout,
                iced::mouse::Cursor,
                &mut dyn iced::core::Clipboard,
                &Rectangle,
            ) -> Option<Message>
            + 'a,
    {
        self.on_press = Some(ButtonEventHandler::FMaybe(Box::new(f)));
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
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout
                    .children()
                    .next()
                    .unwrap()
                    .with_virtual_offset(layout.virtual_offset()),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout
                .children()
                .next()
                .unwrap()
                .with_virtual_offset(layout.virtual_offset()),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if self.on_press.is_some() =>
            {
                let bounds = layout.bounds();

                if cursor.is_over(bounds) {
                    let state = tree.state.downcast_mut::<State>();

                    state.is_pressed = true;

                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(on_press) = &self.on_press {
                    let state = tree.state.downcast_mut::<State>();

                    if state.is_pressed {
                        state.is_pressed = false;

                        let bounds = layout.bounds();

                        if cursor.is_over(bounds)
                            && let Some(message) =
                                on_press.get(event, layout, cursor, clipboard, viewport)
                        {
                            shell.publish(message);
                        }

                        shell.capture_event();
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                if let Some(on_press) = self.on_press.as_ref() {
                    let state = tree.state.downcast_mut::<State>();
                    if state.is_focused
                        && matches!(key, keyboard::Key::Named(keyboard::key::Named::Enter))
                        && let Some(message) =
                            on_press.get(event, layout, cursor, clipboard, viewport)
                    {
                        state.is_pressed = true;
                        shell.publish(message);
                        shell.capture_event();
                        return;
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

        let current_status = if self.on_press.is_none() {
            Status::Disabled
        } else if cursor.is_over(layout.bounds()) {
            let state = tree.state.downcast_ref::<State>();

            if state.is_pressed {
                Status::Pressed
            } else {
                Status::Hovered
            }
        } else {
            Status::Active
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.status = Some(current_status);
        } else if self.status.is_some_and(|status| status != current_status) {
            shell.request_redraw();
        }
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
        let content_layout = layout
            .children()
            .next()
            .unwrap()
            .with_virtual_offset(layout.virtual_offset());
        let style = theme.style(&self.class, self.status.unwrap_or(Status::Disabled));

        if style.background.is_some() || style.border.width > 0.0 || style.shadow.color.a > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    snap: style.snap,
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

        if is_mouse_over && self.on_press.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout
                .children()
                .next()
                .unwrap()
                .with_virtual_offset(layout.virtual_offset()),
            renderer,
            viewport,
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
