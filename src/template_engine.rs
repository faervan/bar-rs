use std::collections::{BTreeMap, HashMap};

use handlebars::Handlebars;
use iced::Element;
use serde::Serialize;

use crate::{
    config::{module_config::MergedModuleConfig, popup_config::MergedPopupConfig},
    Message,
};

type Registry<'a, S: Serialize> = HashMap<
    &'a str,
    fn(
        engine: &TemplateEngine<'a, S>,
        context: &Handlebars<'a>,
        template: Option<(&'a str, BTreeMap<&'a str, S>)>,
        cfg: &MergedModuleConfig<'a>,
        popup_cfg: &MergedPopupConfig<'a>,
        content: String,
    ) -> Element<'a, Message>,
>;

#[derive(Debug)]
pub struct TemplateEngine<'a, S: Serialize> {
    registry: Registry<'a, S>,
}

impl<'a, S: Serialize> TemplateEngine<'a, S> {
    pub fn module() -> TemplateEngine<'a, S> {
        TemplateEngine {
            registry: HashMap::from([
                (
                    "text",
                    Self::module_text
                        as fn(
                            &TemplateEngine<'a, S>,
                            &Handlebars<'a>,
                            Option<(&'a str, BTreeMap<&'a str, S>)>,
                            &MergedModuleConfig<'a>,
                            &MergedPopupConfig<'a>,
                            String,
                        ) -> Element<'a, Message>,
                ),
                ("row", Self::row),
            ]),
        }
    }

    fn module_text(
        &self,
        context: &Handlebars<'a>,
        template: Option<(&'a str, BTreeMap<&'a str, S>)>,
        cfg: &MergedModuleConfig,
        _popup_cfg: &MergedPopupConfig,
        txt: String,
    ) -> Element<'a, Message> {
        iced::widget::text(template.map_or_else(
            || txt,
            |(name, ctx)| context.render(name, &ctx).unwrap_or_default(),
        ))
        .size(cfg.font_size)
        .color(cfg.text_color)
        .into()
    }

    fn new() -> TemplateEngine<'a, S> {
        TemplateEngine {
            registry: HashMap::from([
                (
                    "text",
                    Self::text
                        as fn(
                            &TemplateEngine<'a, S>,
                            &Handlebars<'a>,
                            Option<(&'a str, BTreeMap<&'a str, S>)>,
                            &MergedModuleConfig<'a>,
                            &MergedPopupConfig<'a>,
                            String,
                        ) -> Element<'a, Message>,
                ),
                ("row", Self::row),
            ]),
        }
    }

    fn render(
        &self,
        context: &Handlebars<'a>,
        template: Option<(&'a str, BTreeMap<&'a str, S>)>,
        cfg: &MergedModuleConfig<'a>,
        popup_cfg: &MergedPopupConfig<'a>,
        content: String,
    ) -> Element<'a, Message> {
        let i1 = content.find('(');
        let i2 = content.find(')');
        if let Some((wrapper, cnt)) = i1
            .and_then(|i1| i2.map(|i2| (i1, i2)))
            .map(|(i1, i2)| {
                let x = content.split_at(i1);
                (x.0, x.1.split_at(i2 - i1).0)
            })
            .and_then(|(z1, z2)| {
                let mut chars = z2.chars();
                (chars.next().is_some() && self.registry.contains_key(&z1))
                    .then_some((z1, chars.as_str()))
            })
        {
            self.registry[wrapper](self, context, template, cfg, popup_cfg, cnt.to_string())
        } else {
            self.text(context, template, cfg, popup_cfg, content)
        }
    }

    fn row(
        &self,
        context: &Handlebars<'a>,
        template: Option<(&'a str, BTreeMap<&'a str, S>)>,
        cfg: &MergedModuleConfig<'a>,
        popup_cfg: &MergedPopupConfig<'a>,
        content: String,
    ) -> Element<'a, Message> {
        iced::widget::row(
            content
                .split(',')
                .map(|s| self.render(context, template, cfg, popup_cfg, s.trim().to_string())),
        )
        .into()
    }

    fn text(
        &self,
        context: &Handlebars<'a>,
        template: Option<(&'a str, BTreeMap<&'a str, S>)>,
        cfg: &MergedModuleConfig,
        popup_cfg: &MergedPopupConfig,
        txt: String,
    ) -> Element<'a, Message> {
        iced::widget::text(txt).into()
    }
}
