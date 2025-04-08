use std::{
    any::{Any, TypeId},
    collections::{BTreeMap, HashMap},
    fmt::Debug,
};

/*use battery::BatteryMod;
use cpu::CpuMod;
use date::DateMod;
use disk_usage::DiskUsageMod;*/
use downcast_rs::{impl_downcast, Downcast};
//use hyprland::{window::HyprWindowMod, workspaces::HyprWorkspaceMod};
use iced::{
    theme::Palette,
    widget::{container, Container},
    Alignment, Color, Event, Theme,
};
use iced::{widget::container::Style, Element, Subscription};
/*use media::MediaMod;
use memory::MemoryMod;
use niri::{NiriWindowMod, NiriWorkspaceMod};*/
use time::TimeMod;
//use volume::VolumeMod;
//use wayfire::{WayfireWindowMod, WayfireWorkspaceMod};

use crate::{
    config::{anchor::BarAnchor, module_config::LocalModuleConfig, popup_config::PopupConfig},
    fill::FillExt,
    listeners::Listener,
    registry::Registry,
    template_engine::{ExtraContext, TemplateEngine},
    Message,
};

/*pub mod battery;
pub mod cpu;
pub mod date;
pub mod disk_usage;
pub mod hyprland;
pub mod media;
pub mod memory;
pub mod niri;
pub mod sys_tray;*/
pub mod time;
/*pub mod volume;
pub mod wayfire;*/

pub trait Module: Any + Debug + Send + Sync + Downcast {
    /// The name used to enable the Module in the config.
    fn name(&self) -> String;
    /// The context to use when rendering the module. `context` refers to the data that shall be
    /// displayed by this module.
    fn context<'a>(&'a self) -> BTreeMap<String, Box<dyn ToString + Send + Sync>>;
    /// Like context, but meant for nested rendering. This is used for example to store the
    /// indivial data of cpu cores in the cpu module. There, the outer `BTreeMap` contains a
    /// "cores" entry, which holds a different context for every core in the `Vec`.
    fn extra_context<'a>(&self) -> ExtraContext {
        BTreeMap::new()
    }
    /// Whether the module is currently active and should be shown.
    fn active(&self) -> bool {
        true
    }
    /// The module may optionally have a subscription listening for external events.
    /// See [passive-subscriptions](https://docs.iced.rs/iced/#passive-subscriptions).
    fn subscription(&self) -> Option<Subscription<Message>> {
        None
    }
    /// Modules may require shared subscriptions. Add `require_listener::<SomeListener>()`
    /// for every [Listener] this module requires.
    fn requires(&self) -> Vec<TypeId> {
        vec![]
    }
    #[allow(unused_variables)]
    /// Read configuration options from the config section of this module
    fn read_config<'a>(
        &mut self,
        config: &HashMap<String, Option<String>>,
        popup_config: &HashMap<String, Option<String>>,
        engine: &mut TemplateEngine,
    ) {
    }
    /// Using the context provided by `context()` and `extra_context()` this format defines how the
    /// module should be rendered, unless `view()` is overridden.
    fn module_format(&self) -> &str;
    /// What the module shows. This by default relies on `module_format()`.
    /// See [widgets-and-elements](https://docs.iced.rs/iced/#widgets-and-elements).
    fn module_view<'a>(
        &self,
        config: &LocalModuleConfig,
        engine: &'a TemplateEngine,
    ) -> Element<'a, Message> {
        engine.render_module(self.type_id(), config)
    }
    /// The wrapper around this module, which defines things like background color or border for
    /// this module.
    fn module_wrapper<'a>(
        &self,
        config: &'a LocalModuleConfig,
        anchor: &BarAnchor,
        engine: &'a TemplateEngine,
    ) -> Element<'a, Message> {
        let cfg = engine.get_module_config(self.type_id(), config);
        container(
            container(self.module_view(config, engine))
                .fill(anchor)
                .padding(cfg.padding)
                .style(move |_| Style {
                    background: cfg.background,
                    border: cfg.border,
                    ..Default::default()
                }),
        )
        .fill(anchor)
        .padding(cfg.margin)
        .into()
    }
    #[allow(unused_variables)]
    /// The action to perform when a on_click event occurs
    fn on_click<'a>(
        &'a self,
        event: iced::Event,
        config: &'a LocalModuleConfig,
        engine: &'a TemplateEngine,
    ) -> Option<&'a dyn Action> {
        engine
            .get_module_config(self.type_id(), config)
            .action
            .event(event)
    }
    #[allow(unused_variables, dead_code)]
    /// Using the context provided by `context()` and `extra_context()` this format defines how the
    /// module popup should be rendered, unless `popup_view()` is overridden.
    fn popup_format(&self) -> &str {
        "Missing implementation"
    }
    #[allow(unused_variables)]
    /// The `module_view` but for the popup.
    fn popup_view<'a>(
        &self,
        config: &'a PopupConfig,
        engine: &'a TemplateEngine,
    ) -> Element<'a, Message> {
        engine.render_popup(self.type_id(), config)
    }
    /// Like `module_wrapper` but for the popup.
    fn popup_wrapper<'a>(
        &self,
        config: &'a PopupConfig,
        anchor: &BarAnchor,
        engine: &'a TemplateEngine,
    ) -> Element<'a, Message> {
        let align = |elem: Container<'a, Message>| -> Container<'a, Message> {
            match anchor {
                BarAnchor::Top => elem.align_y(Alignment::Start),
                BarAnchor::Bottom => elem.align_y(Alignment::End),
                BarAnchor::Left => elem.align_x(Alignment::Start),
                BarAnchor::Right => elem.align_x(Alignment::End),
            }
        };
        align(container(self.popup_view(config, engine)).fill(anchor)).into()
    }
    /// The theme of a popup
    fn popup_theme(&self) -> Theme {
        Theme::custom(
            "Default popup theme".to_string(),
            Palette {
                background: Color::TRANSPARENT,
                text: Color::WHITE,
                primary: Color::WHITE,
                success: Color::WHITE,
                danger: Color::WHITE,
            },
        )
    }
}
impl_downcast!(Module);

pub trait Action: Any + Debug + Send + Sync + Downcast {
    fn as_message(&self) -> Message;
}
impl_downcast!(Action);

impl<T: ToString> From<T> for Box<dyn Action> {
    fn from(value: T) -> Box<dyn Action> {
        Box::new(CommandAction(value.to_string()))
    }
}

#[derive(Debug)]
pub struct CommandAction(String);

impl Action for CommandAction {
    fn as_message(&self) -> Message {
        Message::command_sh(&self.0)
    }
}

#[derive(Debug, Default)]
pub struct OnClickAction {
    pub left: Option<Box<dyn Action>>,
    pub center: Option<Box<dyn Action>>,
    pub right: Option<Box<dyn Action>>,
}

impl OnClickAction {
    pub fn event(&self, event: Event) -> Option<&dyn Action> {
        match event {
            Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                self.left.as_deref()
            }
            Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Middle)) => {
                self.center.as_deref()
            }
            Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Right)) => {
                self.right.as_deref()
            }
            _ => None,
        }
    }
}

pub fn require_listener<T>() -> TypeId
where
    T: Listener,
{
    TypeId::of::<T>()
}

pub fn register_modules(registry: &mut Registry) {
    /*registry.register_module::<CpuMod>();
    registry.register_module::<MemoryMod>();
    registry.register_module::<BatteryMod>();
    registry.register_module::<VolumeMod>();
    registry.register_module::<MediaMod>();
    registry.register_module::<DateMod>();*/
    registry.register_module::<TimeMod>();
    /*registry.register_module::<DiskUsageMod>();
    registry.register_module::<HyprWindowMod>();
    registry.register_module::<HyprWorkspaceMod>();
    registry.register_module::<WayfireWorkspaceMod>();
    registry.register_module::<WayfireWindowMod>();
    registry.register_module::<NiriWorkspaceMod>();
    registry.register_module::<NiriWindowMod>();*/
}
