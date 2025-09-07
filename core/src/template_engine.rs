use std::{collections::HashMap, fmt::Debug};

use smithay_client_toolkit::shell::wlr_layer::Anchor;

use crate::{config::style::ContainerStyle, message::Message, module::Context, Element};

pub trait Token<Message: Sized>: Send + Sync {
    fn render<'a>(
        &'a self,
        context: &Context,
        anchor: &Anchor,
        style: &ContainerStyle,
    ) -> Element<'a>;
}

type ToTokenRenderer<Message> = fn(&TemplateEngine, &str) -> Box<dyn Token<Message>>;

pub struct TemplateEngine {
    token_registry: HashMap<&'static str, ToTokenRenderer<Message>>,
}

impl Debug for TemplateEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TemplateEngine")
    }
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            token_registry: HashMap::from([("text", Self::text as ToTokenRenderer<Message>)]),
        }
    }

    pub fn render_token(&self, content: &str) -> Box<dyn Token<Message>> {
        if let Some((wrapper, cnt)) = self.parse_wrapper(content) {
            self.token_registry[wrapper](self, cnt)
        } else {
            Box::new(TextToken(content.to_string()))
        }
    }

    fn parse_wrapper<'a>(&self, content: &'a str) -> Option<(&'a str, &'a str)> {
        let i1 = content.find('(');
        let i2 = content.rfind(')');
        i1.and_then(|i1| i2.map(|i2| (i1, i2)))
            .map(|(i1, i2)| {
                let x = content.split_at(i1);
                (x.0, x.1.split_at(i2 - i1).0)
            })
            .and_then(|(z1, z2)| {
                let mut chars = z2.chars();
                (chars.next().is_some() && self.token_registry.contains_key(&z1))
                    .then_some((z1, chars.as_str()))
            })
    }

    fn text(&self, content: &str) -> Box<dyn Token<Message>> {
        Box::new(TextToken(String::from(content)))
    }
}

struct TextToken(String);

impl<Message: 'static> Token<Message> for TextToken {
    fn render<'a>(
        &'a self,
        context: &Context,
        _anchor: &Anchor,
        style: &ContainerStyle,
    ) -> Element<'a> {
        let style = style.class("text");
        iced::widget::container(iced::widget::text(&self.0).size(style.font_size))
            .padding(style.margin)
            .into()
    }
}
