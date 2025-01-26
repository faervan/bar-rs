use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

use battery::BatteryMod;
use cpu::CpuMod;
use date::DateMod;
use downcast_rs::{impl_downcast, Downcast};
use handlebars::Handlebars;
use hyprland::{window::HyprWindowMod, workspaces::HyprWorkspaceMod};
use iced::{
    theme::Palette,
    widget::{container, Container},
    Alignment, Color, Theme,
};
use iced::{widget::container::Style, Element, Subscription};
use media::MediaMod;
use memory::MemoryMod;
use niri::{NiriWindowMod, NiriWorkspaceMod};
use time::TimeMod;
use volume::VolumeMod;
use wayfire::{WayfireWindowMod, WayfireWorkspaceMod};

use crate::{
    config::{anchor::BarAnchor, module_config::LocalModuleConfig},
    fill::FillExt,
    listeners::Listener,
    registry::Registry,
    Message,
};

pub mod battery;
pub mod cpu;
pub mod date;
pub mod hyprland;
pub mod media;
pub mod memory;
pub mod niri;
pub mod sys_tray;
pub mod time;
pub mod volume;
pub mod wayfire;

pub trait Module: Any + Debug + Send + Sync + Downcast {
    /// The name used to enable the Module in the config.
    fn name(&self) -> String;
    /// What the module actually shows.
    /// See [widgets-and-elements](https://docs.iced.rs/iced/#widgets-and-elements).
    fn view(
        &self,
        config: &LocalModuleConfig,
        anchor: &BarAnchor,
        template: &Handlebars,
    ) -> Element<Message>;
    /// The wrapper around this module, which defines things like background color or border for
    /// this module.
    fn wrapper<'a>(
        &'a self,
        config: &'a LocalModuleConfig,
        content: Element<'a, Message>,
        anchor: &BarAnchor,
    ) -> Element<'a, Message> {
        container(
            container(content)
                .fill(anchor)
                .padding(config.padding)
                .style(|_| Style {
                    background: config.background,
                    border: config.border,
                    ..Default::default()
                }),
        )
        .fill(anchor)
        .padding(config.margin)
        .into()
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
    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        templates: &mut Handlebars,
    ) {
    }
    #[allow(unused_variables, dead_code)]
    /// Handle an action (likely produced by a user interaction).
    fn handle_action(&mut self, action: Box<dyn Action>) {}
    /// The view of a popup
    fn popup_view(&self) -> Element<Message> {
        "Missing implementation".into()
    }
    /// The wrapper around a popup
    fn popup_wrapper<'a>(&'a self, anchor: &BarAnchor) -> Element<'a, Message> {
        let align = |elem: Container<'a, Message>| -> Container<'a, Message> {
            match anchor {
                BarAnchor::Top => elem.align_y(Alignment::Start),
                BarAnchor::Bottom => elem.align_y(Alignment::End),
                BarAnchor::Left => elem.align_x(Alignment::Start),
                BarAnchor::Right => elem.align_x(Alignment::End),
            }
        };
        align(container(self.popup_view()).fill(anchor)).into()
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

pub trait Action: Any + Debug + Send + Sync + Downcast {}
impl_downcast!(Action);

pub fn require_listener<T>() -> TypeId
where
    T: Listener,
{
    TypeId::of::<T>()
}

pub fn register_modules(registry: &mut Registry) {
    registry.register_module::<CpuMod>();
    registry.register_module::<MemoryMod>();
    registry.register_module::<BatteryMod>();
    registry.register_module::<VolumeMod>();
    registry.register_module::<MediaMod>();
    registry.register_module::<DateMod>();
    registry.register_module::<TimeMod>();
    registry.register_module::<HyprWindowMod>();
    registry.register_module::<HyprWorkspaceMod>();
    registry.register_module::<WayfireWorkspaceMod>();
    registry.register_module::<WayfireWindowMod>();
    registry.register_module::<NiriWorkspaceMod>();
    registry.register_module::<NiriWindowMod>();
}

#[macro_export]
macro_rules! impl_wrapper {
    () => {
        fn wrapper<'a>(
            &'a self,
            config: &'a LocalModuleConfig,
            content: Element<'a, Message>,
            anchor: &BarAnchor,
        ) -> Element<'a, Message> {
            iced::widget::container(
                iced::widget::container(content)
                    .fill(anchor)
                    .padding(self.cfg_override.padding.unwrap_or(config.padding))
                    .style(|_| iced::widget::container::Style {
                        background: self.cfg_override.background.unwrap_or(config.background),
                        border: self.cfg_override.border.unwrap_or(config.border),
                        ..Default::default()
                    }),
            )
            .fill(anchor)
            .padding(self.cfg_override.margin.unwrap_or(config.margin))
            .into()
        }
    };
}
