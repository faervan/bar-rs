use std::{
    any::TypeId,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
};

use iced::{Element, Length::Fill};
use regex::Regex;

use crate::{
    config::{
        module_config::{LocalModuleConfig, MergedModuleConfig, ModuleConfigOverride},
        popup_config::{MergedPopupConfig, PopupConfig, PopupConfigOverride},
    },
    helpers::SplitExt,
    modules::{Module, OnClickAction},
    Message, ENGINE, NERD_FONT,
};

type Renderer = fn(
    engine: &'static TemplateEngine,
    id: &TypeId,
    context: Option<(&String, usize)>,
    content: String,
    cfg: &Config,
) -> Element<'static, Message>;

pub struct TemplateEngine {
    registry: HashMap<&'static str, Renderer>,
    module_cfg: HashMap<TypeId, ModuleConfigOverride>,
    popup_cfg: HashMap<TypeId, PopupConfigOverride>,
    context_map: HashMap<TypeId, (GeneralContext, ExtraContext)>,
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
                ("button", Self::button),
                ("image", Self::image),
                ("items", Self::items),
            ]),
            module_cfg: HashMap::new(),
            popup_cfg: HashMap::new(),
            context_map: HashMap::new(),
        }
    }

    pub fn from_static() -> &'static mut Self {
        unsafe { ENGINE.get_mut().unwrap() }
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
        &'static self,
        id: TypeId,
        cfg: &'a LocalModuleConfig,
    ) -> MergedModuleConfig<'a> {
        cfg.override_cfg(self.module_cfg.get(&id).unwrap())
    }

    pub fn get_popup_config<T: Module>(&self, cfg: &PopupConfig) -> MergedPopupConfig {
        cfg.override_cfg(self.popup_cfg.get(&TypeId::of::<T>()).unwrap())
    }

    pub fn render_module(
        &'static self,
        id: TypeId,
        content: String,
        cfg: &LocalModuleConfig,
    ) -> Element<'static, Message> {
        if !self.context_map.contains_key(&id) {
            eprintln!("Context was not registered");
            return "Missing context".into();
        }
        self.render(
            &id,
            None,
            content,
            &Config::Module(cfg.override_cfg(self.module_cfg.get(&id).unwrap())),
        )
    }

    pub fn render_popup(
        &'static self,
        id: TypeId,
        content: String,
        cfg: &PopupConfig,
    ) -> Element<'static, Message> {
        if !self.context_map.contains_key(&id) {
            eprintln!("Context was not registered");
            return "Missing context".into();
        }
        self.render(
            &id,
            None,
            content,
            &Config::Popup(cfg.override_cfg(self.popup_cfg.get(&id).unwrap())),
        )
    }

    fn render(
        &'static self,
        id: &TypeId,
        context: Option<(&String, usize)>,
        content: String,
        cfg: &Config,
    ) -> Element<'static, Message> {
        if let Some((wrapper, cnt)) = self.parse_wrapper(&content) {
            self.registry[wrapper](self, id, context, cnt, cfg)
        } else {
            self.text(id, context, content, cfg)
        }
    }

    fn parse_wrapper<'a>(&self, content: &'a String) -> Option<(&'a str, String)> {
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
                    .then_some((z1, chars.as_str().to_string()))
            })
    }

    fn row(
        &'static self,
        id: &TypeId,
        context: Option<(&String, usize)>,
        content: String,
        cfg: &Config,
    ) -> Element<'static, Message> {
        iced::widget::row(
            content
                .split_checked(',')
                .into_iter()
                .map(|s| self.render(id, context, s.to_string(), cfg)),
        )
        .spacing(cfg.spacing())
        .into()
    }

    fn column(
        &'static self,
        id: &TypeId,
        context: Option<(&String, usize)>,
        content: String,
        cfg: &Config,
    ) -> Element<'static, Message> {
        iced::widget::column(
            content
                .split_checked(',')
                .into_iter()
                .map(|s| self.render(id, context, s.to_string(), cfg)),
        )
        .spacing(cfg.spacing())
        .into()
    }

    fn text(
        &'static self,
        id: &TypeId,
        context: Option<(&String, usize)>,
        content: String,
        cfg: &Config,
    ) -> Element<'static, Message> {
        iced::widget::container(
            iced::widget::text(parse_text(&content, self.get_context(id, context)))
                .size(cfg.font_size())
                .color(cfg.text_color()),
        )
        .padding(cfg.text_margin())
        .into()
    }

    fn icon(
        &'static self,
        id: &TypeId,
        context: Option<(&String, usize)>,
        content: String,
        cfg: &Config,
    ) -> Element<'static, Message> {
        iced::widget::container(
            iced::widget::text(parse_text(&content, self.get_context(id, context)))
                .size(cfg.icon_size())
                .color(cfg.icon_color())
                .font(NERD_FONT),
        )
        .padding(cfg.text_margin())
        .into()
    }

    fn button(
        &'static self,
        id: &TypeId,
        context: Option<(&String, usize)>,
        content: String,
        cfg: &Config,
    ) -> Element<'static, Message> {
        let [cnt, left, center, right] = content.split_checked(',')[..] else {
            eprintln!("Insufficient amount of button arguments! button() needs 4 args");
            return self.text(id, context, content, cfg);
        };
        let action = OnClickAction {
            left: (!left.trim().is_empty())
                .then(|| parse_text(left, self.get_context(id, context)).into()),
            center: (!center.trim().is_empty())
                .then(|| parse_text(center, self.get_context(id, context)).into()),
            right: (!right.trim().is_empty())
                .then(|| parse_text(right, self.get_context(id, context)).into()),
        };
        crate::button::button(self.render(id, context, cnt.to_string(), cfg))
            .on_event_try(move |evt, _, _, _, _| {
                action.event(evt).map(|action| action.as_message())
            })
            .style(|_, _| iced::widget::button::Style::default())
            .into()
    }

    fn image(
        &'static self,
        id: &TypeId,
        context: Option<(&String, usize)>,
        content: String,
        cfg: &Config,
    ) -> Element<'static, Message> {
        let [path, width, height] = content.split_checked(',')[..] else {
            eprintln!("Insufficient amount of image arguments! image() needs 3 args");
            return self.text(id, context, content, cfg);
        };
        let ctx = self.get_context(id, context);
        iced::widget::image(parse_text(path, ctx).to_string())
            .width(
                parse_text(width, ctx)
                    .parse::<f32>()
                    .map(iced::Length::Fixed)
                    .unwrap_or(Fill),
            )
            .height(
                parse_text(height, ctx)
                    .parse::<f32>()
                    .map(iced::Length::Fixed)
                    .unwrap_or(Fill),
            )
            .into()
    }

    fn items(
        &'static self,
        id: &TypeId,
        context: Option<(&String, usize)>,
        content: String,
        cfg: &Config,
    ) -> Element<'static, Message> {
        let [subset, chain_method, format] = content.splitn(3, ',').collect::<Vec<&str>>()[..]
        else {
            eprintln!("Insufficient amount of arguments! items() needs 3 args");
            return self.text(id, context, content, cfg);
        };
        let chain_method = parse_text(chain_method, self.get_context(id, context));
        let children = self.context_map[id]
            .1
            .get(subset)
            .expect("This context is not available")
            .iter()
            .enumerate()
            .map(|(i, _)| self.render(id, Some((&subset.to_string(), i)), format.to_string(), cfg));
        match chain_method.trim() {
            "row" => iced::widget::row(children).spacing(cfg.spacing()).into(),
            "column" => iced::widget::column(children).spacing(cfg.spacing()).into(),
            _ => "Unsupported chain method".into(),
        }
    }

    fn get_context<'a>(
        &'static self,
        id: &TypeId,
        context: Option<(&String, usize)>,
    ) -> &'a GeneralContext {
        match context {
            Some((key, i)) => &self.context_map[id].1[key][i],
            None => &self.context_map[id].0,
        }
    }
}

fn parse_text<'a>(template: &str, context: &GeneralContext) -> String {
    let regex = Regex::new(r"\{\{(.*?)\}\}").unwrap();
    regex
        .replace_all(template, |caps: &regex::Captures| {
            let key = &caps[1];
            context
                .get(key)
                .map_or_else(|| format!("{{{{{}}}}}", key), |v| v.to_string())
        })
        .to_string()
}

#[derive(Clone, Copy)]
struct Context<'a> {
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
}

type GeneralContext = BTreeMap<String, Box<dyn ToString + Send + Sync>>;
pub type ExtraContext = BTreeMap<String, Vec<GeneralContext>>;

enum Config<'a> {
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
