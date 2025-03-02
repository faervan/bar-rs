use std::collections::{BTreeMap, HashMap};

use iced::{Element, Length::Fill};
use regex::Regex;

use crate::{
    config::{module_config::MergedModuleConfig, popup_config::MergedPopupConfig},
    helpers::SplitExt,
    modules::OnClickAction,
    Message,
};

type Registry<'a, S> = HashMap<
    &'a str,
    fn(
        engine: &'a TemplateEngine<'a, S>,
        context: &BTreeMap<&'a str, S>,
        content: &'a str,
        is_popup: bool,
    ) -> Element<'a, Message>,
>;

#[derive(Debug)]
pub struct TemplateEngine<'a, S: ToString> {
    registry: Registry<'a, S>,
    module_cfg: MergedModuleConfig<'a>,
    popup_cfg: MergedPopupConfig,
}

impl<'a, S: ToString> TemplateEngine<'a, S> {
    pub fn new(
        module_cfg: MergedModuleConfig<'a>,
        popup_cfg: MergedPopupConfig,
    ) -> TemplateEngine<'a, S> {
        TemplateEngine {
            registry: HashMap::from([
                (
                    "text",
                    Self::text
                        as fn(
                            &'a TemplateEngine<'a, S>,
                            &BTreeMap<&'a str, S>,
                            &'a str,
                            bool,
                        ) -> Element<'a, Message>,
                ),
                ("icon", Self::icon),
                ("row", Self::row),
                ("column", Self::column),
                ("button", Self::button),
                ("image", Self::image),
            ]),
            module_cfg,
            popup_cfg,
        }
    }

    pub fn update_module_cfg(&mut self, module_cfg: MergedModuleConfig<'a>) {
        self.module_cfg = module_cfg;
    }

    pub fn update_popup_cfg(&mut self, popup_cfg: MergedPopupConfig) {
        self.popup_cfg = popup_cfg;
    }

    pub fn render_module(
        &'a self,
        context: &BTreeMap<&'a str, S>,
        content: &'a str,
    ) -> Element<'a, Message> {
        if let Some((wrapper, cnt)) = self.parse_wrapper(content) {
            self.registry[wrapper](self, context, cnt, false)
        } else {
            self.text(context, content, false)
        }
    }

    pub fn render_popup(
        &'a self,
        context: &BTreeMap<&'a str, S>,
        content: &'a str,
    ) -> Element<'a, Message> {
        if let Some((wrapper, cnt)) = self.parse_wrapper(content) {
            self.registry[wrapper](self, context, cnt, true)
        } else {
            self.text(context, content, true)
        }
    }

    fn parse_wrapper(&self, content: &'a str) -> Option<(&str, &str)> {
        let i1 = content.find('(');
        let i2 = content.find(')');
        i1.and_then(|i1| i2.map(|i2| (i1, i2)))
            .map(|(i1, i2)| {
                let x = content.split_at(i1);
                (x.0, x.1.split_at(i2 - i1).0)
            })
            .and_then(|(z1, z2)| {
                let mut chars = z2.chars();
                (chars.next().is_some() && self.registry.contains_key(&z1))
                    .then_some((z1, chars.as_str()))
            })
    }

    fn row(
        &'a self,
        context: &BTreeMap<&'a str, S>,
        content: &'a str,
        is_popup: bool,
    ) -> Element<'a, Message> {
        iced::widget::row(
            content
                .split_checked(',')
                .into_iter()
                .map(|s| match is_popup {
                    true => self.render_popup(context, s),
                    false => self.render_module(context, s),
                }),
        )
        .spacing(match is_popup {
            true => self.popup_cfg.spacing,
            false => self.module_cfg.spacing,
        })
        .into()
    }

    fn column(
        &'a self,
        context: &BTreeMap<&'a str, S>,
        content: &'a str,
        is_popup: bool,
    ) -> Element<'a, Message> {
        iced::widget::column(
            content
                .split_checked(',')
                .into_iter()
                .map(|s| match is_popup {
                    true => self.render_popup(context, s),
                    false => self.render_module(context, s),
                }),
        )
        .spacing(match is_popup {
            true => self.popup_cfg.spacing,
            false => self.module_cfg.spacing,
        })
        .into()
    }

    fn text(
        &self,
        context: &BTreeMap<&'a str, S>,
        content: &'a str,
        is_popup: bool,
    ) -> Element<'a, Message> {
        iced::widget::container(match is_popup {
            true => iced::widget::text(parse_text(content, context))
                .size(self.popup_cfg.font_size)
                .color(self.popup_cfg.text_color),
            false => iced::widget::text(parse_text(content, context))
                .size(self.module_cfg.font_size)
                .color(self.module_cfg.text_color),
        })
        .padding(match is_popup {
            true => self.popup_cfg.text_margin,
            false => self.module_cfg.text_margin,
        })
        .into()
    }

    fn icon(
        &self,
        context: &BTreeMap<&'a str, S>,
        content: &'a str,
        is_popup: bool,
    ) -> Element<'a, Message> {
        iced::widget::container(match is_popup {
            true => iced::widget::text(parse_text(content, context))
                .size(self.popup_cfg.icon_size)
                .color(self.popup_cfg.icon_color),
            false => iced::widget::text(parse_text(content, context))
                .size(self.module_cfg.icon_size)
                .color(self.module_cfg.icon_color),
        })
        .padding(match is_popup {
            true => self.popup_cfg.icon_margin,
            false => self.module_cfg.icon_margin,
        })
        .into()
    }

    fn button(
        &'a self,
        context: &BTreeMap<&'a str, S>,
        content: &'a str,
        is_popup: bool,
    ) -> Element<'a, Message> {
        let [cnt, left, center, right] = content.split_checked(',')[..] else {
            eprintln!("Insufficient amount of button arguments! button() needs 4 args");
            return self.text(context, content, is_popup);
        };
        let action = OnClickAction {
            left: (left.trim().is_empty()).then(|| left.into()),
            center: (center.trim().is_empty()).then(|| center.into()),
            right: (right.trim().is_empty()).then(|| right.into()),
        };
        crate::button::button(match is_popup {
            true => self.render_popup(context, cnt.trim()),
            false => self.render_module(context, cnt.trim()),
        })
        .on_event_try(move |evt, _, _, _, _| action.event(evt).map(|action| action.as_message()))
        .style(|_, _| iced::widget::button::Style::default())
        .into()
    }

    fn image(
        &'a self,
        context: &BTreeMap<&'a str, S>,
        content: &'a str,
        is_popup: bool,
    ) -> Element<'a, Message> {
        let [path, width, height] = content.split_checked(',')[..] else {
            eprintln!("Insufficient amount of image arguments! image() needs 3 args");
            return self.text(context, content, is_popup);
        };
        iced::widget::image(path)
            .width(
                width
                    .parse::<f32>()
                    .map(iced::Length::Fixed)
                    .unwrap_or(Fill),
            )
            .height(
                height
                    .parse::<f32>()
                    .map(iced::Length::Fixed)
                    .unwrap_or(Fill),
            )
            .into()
    }
}

fn parse_text<'a, S: ToString>(
    template: &'a str,
    ctx: &BTreeMap<&str, S>,
) -> std::borrow::Cow<'a, str> {
    let regex = Regex::new(r"\{\{(.*?)\}\}").unwrap();
    regex.replace_all(template, |caps: &regex::Captures| {
        let key = &caps[1];
        ctx.get(key)
            .map_or_else(|| format!("{{{{{}}}}}", key), |v| v.to_string())
    })
}
