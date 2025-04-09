use std::{any::TypeId, collections::HashMap, fmt::Debug};

use iced::{Element, Length};

use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, MergedModuleConfig, ModuleConfigOverride},
        popup_config::{MergedPopupConfig, PopupConfig, PopupConfigOverride},
    },
    fill::FillExt,
    helpers::{ParseTemplate, SplitExt},
    modules::{Module, OnClickAction},
    Message, NERD_FONT,
};

pub trait Token {
    fn render<'a>(&'a self, context: Context<'a>, config: &Config) -> Element<'a, Message>;
}

type ToTokenRenderer = fn(&TemplateEngine, &str) -> Box<dyn Token>;

pub struct TemplateEngine {
    registry: HashMap<&'static str, ToTokenRenderer>,
    module_cfg: HashMap<TypeId, ModuleConfigOverride>,
    popup_cfg: HashMap<TypeId, PopupConfigOverride>,
    context_map: HashMap<TypeId, (GeneralContext, ExtraContext)>,
    token_map: HashMap<TypeId, Box<dyn Token>>,
    popup_token_map: HashMap<TypeId, Box<dyn Token>>,
}

impl Debug for TemplateEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TemplateEngine")
    }
}

impl TemplateEngine {
    pub fn new() -> TemplateEngine {
        TemplateEngine {
            registry: HashMap::from([
                ("text", Self::text as ToTokenRenderer),
                ("icon", Self::icon),
                ("row", Self::row),
                ("column", Self::column),
                ("container", Self::container),
                ("box", Self::container),
                ("button", Self::button),
                ("image", Self::image),
            ]),
            module_cfg: HashMap::new(),
            popup_cfg: HashMap::new(),
            context_map: HashMap::new(),
            token_map: HashMap::new(),
            popup_token_map: HashMap::new(),
        }
    }

    pub fn generate_token(&mut self, id: TypeId, module_fmt: &str, popup_fmt: Option<&str>) {
        self.token_map.insert(id, self.render_token(module_fmt));
        if let Some(fmt) = popup_fmt {
            self.popup_token_map.insert(id, self.render_token(fmt));
        }
    }

    fn render_token(&self, content: &str) -> Box<dyn Token> {
        if let Some((wrapper, cnt)) = self.parse_wrapper(content) {
            self.registry[wrapper](self, cnt)
        } else {
            Box::new(TextToken(content.to_string()))
        }
    }

    fn text(&self, content: &str) -> Box<dyn Token> {
        Box::new(TextToken(content.to_string()))
    }

    fn icon(&self, content: &str) -> Box<dyn Token> {
        Box::new(IconToken(content.to_string()))
    }

    fn row(&self, content: &str) -> Box<dyn Token> {
        Box::new(RowToken(
            content
                .split_checked(',')
                .iter()
                .map(|s| self.render_token(s.trim()))
                .collect(),
        ))
    }

    fn column(&self, content: &str) -> Box<dyn Token> {
        Box::new(ColumnToken(
            content
                .split_checked(',')
                .iter()
                .map(|s| self.render_token(s.trim()))
                .collect(),
        ))
    }

    fn container(&self, content: &str) -> Box<dyn Token> {
        let [cnt, icon_margin] = content.split_checked(',')[..] else {
            eprintln!("Insufficient amount of container arguments! container() needs 2 args");
            return Box::new(TextToken(content.to_string()));
        };
        Box::new(BoxToken {
            content: self.render_token(cnt),
            icon_margin: icon_margin.parse().unwrap(),
        })
    }

    fn button(&self, content: &str) -> Box<dyn Token> {
        let [cnt, left, center, right] = content.split_checked(',')[..] else {
            eprintln!("Insufficient amount of button arguments! button() needs 4 args");
            return Box::new(TextToken(content.to_string()));
        };
        let action = OnClickAction {
            left: (left.trim().is_empty()).then(|| left.into()),
            center: (center.trim().is_empty()).then(|| center.into()),
            right: (right.trim().is_empty()).then(|| right.into()),
        };
        Box::new(ButtonToken {
            content: self.render_token(cnt),
            action,
        })
    }

    fn image(&self, content: &str) -> Box<dyn Token> {
        let [path, width, height] = content.split_checked(',')[..] else {
            eprintln!("Insufficient amount of image arguments! image() needs 3 args");
            return Box::new(TextToken(content.to_string()));
        };
        Box::new(ImageToken {
            path: path.to_string(),
            width: width
                .parse::<f32>()
                .map(Length::Fixed)
                .unwrap_or(Length::Fill),
            height: height
                .parse::<f32>()
                .map(Length::Fixed)
                .unwrap_or(Length::Fill),
        })
    }

    pub fn set_module_cfg(&mut self, id: TypeId, cfg: ModuleConfigOverride) {
        self.module_cfg.insert(id, cfg);
    }

    pub fn set_popup_cfg(&mut self, id: TypeId, cfg: PopupConfigOverride) {
        self.popup_cfg.insert(id, cfg);
    }

    pub fn set_context(&mut self, id: TypeId, general: GeneralContext, extra: ExtraContext) {
        self.context_map.insert(id, (general, extra));
    }

    pub fn register_renderer(&mut self, name: &'static str, renderer: ToTokenRenderer) {
        self.registry.insert(name, renderer);
    }

    pub fn get_module_config<'a>(
        &'a self,
        id: TypeId,
        cfg: &'a LocalModuleConfig,
    ) -> MergedModuleConfig<'a> {
        cfg.override_cfg(self.module_cfg.get(&id).unwrap())
    }

    pub fn get_popup_config(&self, id: TypeId, cfg: &PopupConfig) -> MergedPopupConfig {
        cfg.override_cfg(self.popup_cfg.get(&id).unwrap())
    }

    pub fn render_module<'a>(
        &'a self,
        id: TypeId,
        cfg: &LocalModuleConfig,
        anchor: &'a BarAnchor,
    ) -> Element<'a, Message> {
        if !self.token_map.contains_key(&id) {
            eprintln!("Token was not generated");
            return "Missing token".into();
        }
        if !self.context_map.contains_key(&id) {
            eprintln!("Context was not registered");
            return "Missing context".into();
        }
        if !self.module_cfg.contains_key(&id) {
            eprintln!("Module config was not registered");
            return "Missing module config".into();
        }
        self.render(
            &id,
            &Config::Module(cfg.override_cfg(&self.module_cfg[&id])),
            anchor,
        )
    }

    pub fn render_popup<'a>(
        &'a self,
        id: TypeId,
        cfg: &PopupConfig,
        anchor: &'a BarAnchor,
    ) -> Element<'a, Message> {
        if !self.token_map.contains_key(&id) {
            eprintln!("Token was not generated");
            return "Missing token".into();
        }
        if !self.context_map.contains_key(&id) {
            eprintln!("Context was not registered");
            return "Missing context".into();
        }
        if !self.popup_cfg.contains_key(&id) {
            eprintln!("Popup config was not registered");
            return "Missing popup config".into();
        }
        self.render(
            &id,
            &Config::Popup(cfg.override_cfg(&self.popup_cfg[&id])),
            anchor,
        )
    }

    fn render<'a>(
        &'a self,
        id: &TypeId,
        config: &Config,
        anchor: &'a BarAnchor,
    ) -> Element<'a, Message> {
        match config.is_popup() {
            true => &self.popup_token_map[id],
            false => &self.token_map[id],
        }
        .render(Context::new(&self.context_map[id], anchor), config)
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
                (chars.next().is_some() && self.registry.contains_key(&z1))
                    .then_some((z1, chars.as_str()))
            })
    }
}

struct TextToken(String);

impl Token for TextToken {
    fn render<'a>(&'a self, context: Context<'a>, config: &Config) -> Element<'a, Message> {
        iced::widget::container(
            iced::widget::text(context.get().parse_template(&self.0))
                .fill(context.anchor)
                .size(config.font_size())
                .color(config.text_color()),
        )
        .fill(context.anchor)
        .padding(config.text_margin())
        .into()
    }
}

struct IconToken(String);

impl Token for IconToken {
    fn render<'a>(&'a self, context: Context<'a>, config: &Config) -> Element<'a, Message> {
        iced::widget::container(
            iced::widget::text(context.get().parse_template(&self.0))
                .fill(context.anchor)
                .size(config.icon_size())
                .color(config.icon_color())
                .font(NERD_FONT),
        )
        .fill(context.anchor)
        .padding(config.icon_margin())
        .into()
    }
}

struct RowToken(Vec<Box<dyn Token>>);

impl Token for RowToken {
    fn render<'a>(&'a self, context: Context<'a>, config: &Config) -> Element<'a, Message> {
        iced::widget::row(self.0.iter().map(|t| t.render(context, config)))
            .spacing(config.spacing())
            .into()
    }
}

struct ColumnToken(Vec<Box<dyn Token>>);

impl Token for ColumnToken {
    fn render<'a>(&'a self, context: Context<'a>, config: &Config) -> Element<'a, Message> {
        iced::widget::column(self.0.iter().map(|t| t.render(context, config)))
            .spacing(config.spacing())
            .into()
    }
}

struct BoxToken {
    content: Box<dyn Token>,
    icon_margin: bool,
}

impl Token for BoxToken {
    fn render<'a>(&'a self, context: Context<'a>, config: &Config) -> Element<'a, Message> {
        iced::widget::container(self.content.render(context, config))
            .fill(context.anchor)
            .padding(match self.icon_margin {
                true => config.icon_margin(),
                false => config.text_margin(),
            })
            .into()
    }
}

struct ButtonToken {
    content: Box<dyn Token>,
    action: OnClickAction,
}

impl Token for ButtonToken {
    fn render<'a>(&'a self, context: Context<'a>, config: &Config) -> Element<'a, Message> {
        crate::button::button(self.content.render(context, config))
            .on_event_try(move |evt, _, _, _, _| {
                self.action.event(evt).map(|action| action.as_message())
            })
            .style(|_, _| iced::widget::button::Style::default())
            .into()
    }
}
struct ImageToken {
    path: String,
    width: Length,
    height: Length,
}

impl Token for ImageToken {
    fn render<'a>(&'a self, _context: Context<'a>, _config: &Config) -> Element<'a, Message> {
        iced::widget::image(&self.path)
            .width(self.width)
            .height(self.height)
            .into()
    }
}

#[derive(Clone, Copy)]
pub struct Context<'a> {
    current: Option<(&'a str, usize)>,
    general: &'a GeneralContext,
    extra: &'a ExtraContext,
    anchor: &'a BarAnchor,
}

impl<'a> Context<'a> {
    fn new((general, extra): &'a (GeneralContext, ExtraContext), anchor: &'a BarAnchor) -> Self {
        Self {
            current: None,
            general,
            extra,
            anchor,
        }
    }

    fn with_current(mut self, key: &'a str, id: usize) -> Self {
        self.current = Some((key, id));
        self
    }

    fn get(&self) -> &'a GeneralContext {
        match &self.current {
            Some((key, id)) => &self.extra[*key][*id],
            None => self.general,
        }
    }
}

pub type GeneralContext = HashMap<String, Box<dyn ToString + Send + Sync>>;
pub type ExtraContext = HashMap<String, Vec<GeneralContext>>;

pub enum Config<'a> {
    Module(MergedModuleConfig<'a>),
    Popup(MergedPopupConfig),
}

impl<'a> Config<'a> {
    fn is_popup(&self) -> bool {
        match self {
            Config::Module(_) => false,
            Config::Popup(_) => true,
        }
    }

    fn text_margin(&self) -> iced::Padding {
        match self {
            Config::Module(m) => m.text_margin,
            Config::Popup(p) => p.text_margin,
        }
    }

    fn icon_margin(&self) -> iced::Padding {
        match self {
            Config::Module(m) => m.icon_margin,
            Config::Popup(p) => p.icon_margin,
        }
    }

    fn text_color(&self) -> iced::Color {
        match self {
            Config::Module(m) => m.text_color,
            Config::Popup(p) => p.text_color,
        }
    }

    fn icon_color(&self) -> iced::Color {
        match self {
            Config::Module(m) => m.icon_color,
            Config::Popup(p) => p.icon_color,
        }
    }

    fn font_size(&self) -> f32 {
        match self {
            Config::Module(m) => m.font_size,
            Config::Popup(p) => p.font_size,
        }
    }

    fn icon_size(&self) -> f32 {
        match self {
            Config::Module(m) => m.icon_size,
            Config::Popup(p) => p.icon_size,
        }
    }

    fn spacing(&self) -> f32 {
        match self {
            Config::Module(m) => m.spacing,
            Config::Popup(p) => p.spacing,
        }
    }
}
