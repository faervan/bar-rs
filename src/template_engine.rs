use std::{
    any::TypeId,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
};

use iced::Element;
use regex::Regex;

use crate::{
    config::{
        module_config::{LocalModuleConfig, MergedModuleConfig, ModuleConfigOverride},
        popup_config::{MergedPopupConfig, PopupConfig, PopupConfigOverride},
    },
    modules::Module,
    Message, NERD_FONT,
};

pub trait Token {
    fn render<'a>(&'a self, context: Context<'a>, config: &Config) -> Element<'a, Message>;
}

type Renderer = fn(&TemplateEngine, &str) -> Box<dyn Token>;

pub struct TemplateEngine {
    registry: HashMap<&'static str, Renderer>,
    module_cfg: HashMap<TypeId, ModuleConfigOverride>,
    popup_cfg: HashMap<TypeId, PopupConfigOverride>,
    context_map: HashMap<TypeId, (GeneralContext, ExtraContext)>,
    token_map: HashMap<TypeId, Box<dyn Token>>,
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
                ("text", Self::text as Renderer),
                ("icon", Self::icon),
                ("row", Self::row),
                ("column", Self::column),
            ]),
            module_cfg: HashMap::new(),
            popup_cfg: HashMap::new(),
            context_map: HashMap::new(),
            token_map: HashMap::new(),
        }
    }

    pub fn generate_token(&mut self, content: &str, id: TypeId) {
        self.token_map.insert(id, self.render_token(content));
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
                .split(',')
                .map(|s| self.render_token(s.trim()))
                .collect(),
        ))
    }

    fn column(&self, content: &str) -> Box<dyn Token> {
        Box::new(ColumnToken(
            content
                .split(',')
                .map(|s| self.render_token(s.trim()))
                .collect(),
        ))
    }

    pub fn set_module_cfg<T: Module>(&mut self, cfg: ModuleConfigOverride) {
        self.module_cfg.insert(TypeId::of::<T>(), cfg);
    }

    pub fn set_popup_cfg<T: Module>(&mut self, cfg: PopupConfigOverride) {
        self.popup_cfg.insert(TypeId::of::<T>(), cfg);
    }

    pub fn set_context<T: Module>(&mut self, general: GeneralContext, extra: ExtraContext) {
        self.context_map.insert(TypeId::of::<T>(), (general, extra));
    }

    pub fn register_renderer(&mut self, name: &'static str, renderer: Renderer) {
        self.registry.insert(name, renderer);
    }

    pub fn get_module_config<'a>(
        &'a self,
        id: TypeId,
        cfg: &'a LocalModuleConfig,
    ) -> MergedModuleConfig<'a> {
        cfg.override_cfg(self.module_cfg.get(&id).unwrap())
    }

    pub fn get_popup_config<T: Module>(&self, cfg: &PopupConfig) -> MergedPopupConfig {
        cfg.override_cfg(self.popup_cfg.get(&TypeId::of::<T>()).unwrap())
    }

    pub fn render_module<'a>(
        &'a self,
        id: TypeId,
        cfg: &LocalModuleConfig,
    ) -> Element<'a, Message> {
        if !self.token_map.contains_key(&id) {
            eprintln!("Token was not generated");
            return "Missing token".into();
        }
        if !self.context_map.contains_key(&id) {
            eprintln!("Context was not registered");
            return "Missing context".into();
        }
        self.render(
            &id,
            &Config::Module(cfg.override_cfg(self.module_cfg.get(&id).unwrap())),
        )
    }

    pub fn render_popup<'a>(&'a self, id: TypeId, cfg: &PopupConfig) -> Element<'a, Message> {
        if !self.token_map.contains_key(&id) {
            eprintln!("Token was not generated");
            return "Missing token".into();
        }
        if !self.context_map.contains_key(&id) {
            eprintln!("Context was not registered");
            return "Missing context".into();
        }
        self.render(
            &id,
            &Config::Popup(cfg.override_cfg(self.popup_cfg.get(&id).unwrap())),
        )
    }

    fn render<'a>(&'a self, id: &TypeId, config: &Config) -> Element<'a, Message> {
        self.token_map[id].render(Context::new(&self.context_map[id]), config)
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
            iced::widget::text(context.parse_text(&self.0))
                .size(config.font_size())
                .color(config.text_color()),
        )
        .padding(config.text_margin())
        .into()
    }
}

struct IconToken(String);

impl Token for IconToken {
    fn render<'a>(&'a self, context: Context<'a>, config: &Config) -> Element<'a, Message> {
        iced::widget::container(
            iced::widget::text(context.parse_text(&self.0))
                .size(config.icon_size())
                .color(config.icon_color())
                .font(NERD_FONT),
        )
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

#[derive(Clone, Copy)]
pub struct Context<'a> {
    current: Option<(&'a str, usize)>,
    general: &'a GeneralContext,
    extra: &'a ExtraContext,
}

impl<'a> Context<'a> {
    fn new((general, extra): &'a (GeneralContext, ExtraContext)) -> Self {
        Self {
            current: None,
            general,
            extra,
        }
    }

    fn with_current(mut self, key: &'a str, id: usize) -> Self {
        self.current = Some((key, id));
        self
    }

    fn get_context(&self) -> &'a GeneralContext {
        match &self.current {
            Some((key, id)) => &self.extra[*key][*id],
            None => self.general,
        }
    }

    fn parse_text(&self, template: &str) -> String {
        let regex = Regex::new(r"\{\{(.*?)\}\}").unwrap();
        regex
            .replace_all(template, |caps: &regex::Captures| {
                let key = &caps[1];
                self.get_context()
                    .get(key)
                    .map_or_else(|| format!("{{{{{}}}}}", key), |v| v.to_string())
            })
            .to_string()
    }
}

pub type GeneralContext = BTreeMap<String, Box<dyn ToString + Send + Sync>>;
pub type ExtraContext = BTreeMap<String, Vec<GeneralContext>>;

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
