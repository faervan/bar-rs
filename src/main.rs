use std::{
    any::{Any, TypeId},
    fmt::Debug,
    path::PathBuf,
    process::{Command, exit},
    sync::Arc,
    time::Duration,
};

use config::{Config, EnabledModules, Thrice, anchor::BarAnchor, get_config_dir, read_config};
use fill::FillExt;
use handlebars::Handlebars;
use iced::{
    Alignment, Color, Element, Font, Rectangle, Subscription, Task, Theme, daemon,
    event::wayland::OutputEvent,
    platform_specific::shell::commands::{
        layer_surface::{Layer, destroy_layer_surface, get_layer_surface},
        popup::{destroy_popup, get_popup},
    },
    runtime::platform_specific::wayland::{
        layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
        popup::{SctkPopupSettings, SctkPositioner},
    },
    theme::Palette,
    widget::{container, stack},
    window::Id,
};
use list::{DynamicAlign, list};
use listeners::register_listeners;
use modules::{Module, empty::EmptyModule, register_modules};
use registry::Registry;
use resolvers::register_resolvers;
use smithay_client_toolkit::{
    output::OutputInfo, reexports::client::protocol::wl_output::WlOutput,
};
use tokio::{
    sync::{broadcast, mpsc},
    time::sleep,
};

mod config;
#[macro_use]
mod list;
mod button;
mod fill;
mod helpers;
mod listeners;
mod modules;
mod registry;
mod resolvers;
mod tooltip;

const NERD_FONT: Font = Font::with_name("3270 Nerd Font");

fn main() -> iced::Result {
    daemon(Bar::new, Bar::update, Bar::view)
        .title("Bar")
        .theme(Bar::theme)
        .font(include_bytes!("../assets/3270/3270NerdFont-Regular.ttf"))
        .subscription(|state| {
            if state.open {
                Subscription::batch([
                    iced::event::listen_with(|event, _, _| {
                        if let iced::Event::PlatformSpecific(
                            iced::event::PlatformSpecific::Wayland(
                                iced::event::wayland::Event::Output(event, wl_output),
                            ),
                        ) = event
                        {
                            Some(Message::OutputEvent {
                                event: Box::new(event),
                                wl_output,
                            })
                        } else {
                            None
                        }
                    }),
                    Subscription::batch({
                        state
                            .registry
                            .get_modules(state.config.enabled_modules.get_all(), &state.config)
                            .filter(|m| state.config.enabled_modules.contains(&m.name()))
                            .filter_map(|m| m.subscription())
                            .chain(
                                state
                                    .registry
                                    .get_listeners(&state.config.enabled_listeners)
                                    .map(|l| l.subscription()),
                            )
                    }),
                ])
            } else {
                Subscription::none()
            }
        })
        .run()
}

pub struct UpdateFn(Box<dyn FnOnce(&mut Registry) + Send + Sync>);
impl Debug for UpdateFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UpdateFn(Box<dyn FnOnce(&mut Registry) + Send + Sync>) can't be displayed"
        )
    }
}
pub struct ActionFn(Box<dyn FnOnce(&Registry) + Send + Sync>);
impl Debug for ActionFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ActionFn(Box<dyn FnOnce(&Registry) + Send + Sync>) can't be displayed"
        )
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Popup {
        type_id: TypeId,
        dimension: Rectangle<i32>,
    },
    Update(Arc<UpdateFn>),
    Action(Arc<ActionFn>),
    GetConfig(mpsc::Sender<(Arc<PathBuf>, Arc<Config>)>),
    GetReceiver(
        mpsc::Sender<broadcast::Receiver<Arc<dyn Any + Send + Sync>>>,
        fn(&Registry) -> broadcast::Receiver<Arc<dyn Any + Send + Sync>>,
    ),
    Spawn(Arc<Command>),
    ReloadConfig,
    LoadRegistry,
    OutputEvent {
        event: Box<OutputEvent>,
        wl_output: WlOutput,
    },
    OutputsReady,
}

impl Message {
    fn update<F>(f: F) -> Self
    where
        F: FnOnce(&mut Registry) + Send + Sync + 'static,
    {
        Message::Update(Arc::new(UpdateFn(Box::new(f))))
    }
    fn action<F>(f: F) -> Self
    where
        F: FnOnce(&Registry) + Send + Sync + 'static,
    {
        Message::Action(Arc::new(ActionFn(Box::new(f))))
    }
    #[allow(dead_code)]
    fn command<I, S>(command: S, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let mut cmd = Command::new(command);
        cmd.args(args);
        Message::Spawn(Arc::new(cmd))
    }
    fn command_sh<S>(arg: S) -> Self
    where
        S: AsRef<std::ffi::OsStr>,
    {
        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        cmd.arg(arg);
        Message::Spawn(Arc::new(cmd))
    }
    fn popup<'a, T>(
        width: i32,
        height: i32,
        anchor: &BarAnchor,
    ) -> impl Fn(
        &iced::Event,
        iced::core::Layout,
        iced::mouse::Cursor,
        &mut dyn iced::core::Clipboard,
        &Rectangle,
    ) -> Message
    + 'a
    where
        T: Module,
    {
        let anchor = *anchor;
        move |_: &iced::Event,
              layout: iced::core::Layout,
              _: iced::mouse::Cursor,
              _: &mut dyn iced::core::Clipboard,
              _: &Rectangle| {
            let bounds = layout.bounds();
            let position = layout.position();
            let x = match anchor {
                BarAnchor::Left => bounds.width as i32,
                BarAnchor::Right => -width,
                _ => position.x as i32,
            };
            let y = match anchor {
                BarAnchor::Top => bounds.height as i32,
                BarAnchor::Bottom => -height,
                _ => position.y as i32,
            };
            Message::Popup {
                type_id: TypeId::of::<T>(),
                dimension: Rectangle {
                    x,
                    y,
                    width,
                    height,
                },
            }
        }
    }
}

pub type Outputs = Vec<(WlOutput, Option<OutputInfo>)>;

#[derive(Debug)]
struct Bar<'a> {
    config_file: Arc<PathBuf>,
    config: Arc<Config>,
    registry: Registry,
    outputs: Outputs,
    /// If false, we have to wait for new Outputs before opening a window
    outputs_ready: bool,
    layer_id: Id,
    open: bool,
    popup: Option<(TypeId, Id)>,
    templates: Handlebars<'a>,
}

impl Bar<'_> {
    fn new() -> (Self, Task<Message>) {
        let mut registry = Registry::default();
        register_modules(&mut registry);
        register_listeners(&mut registry);
        register_resolvers(&mut registry);

        let mut templates = Handlebars::new();

        let config_file = get_config_dir();
        let config = read_config(&config_file, &mut registry, &mut templates);

        ctrlc::set_handler(|| {
            println!("Received exit signal...Exiting");
            exit(0);
        })
        .unwrap();

        let bar = Self {
            config_file: config_file.into(),
            config: config.into(),
            registry,
            outputs: Default::default(),
            outputs_ready: false,
            layer_id: Id::unique(),
            open: true,
            popup: None,
            templates,
        };

        (bar, Task::none())
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Popup { type_id, dimension } => {
                let settings = |id| SctkPopupSettings {
                    parent: self.layer_id,
                    id,
                    positioner: SctkPositioner {
                        size: Some((dimension.width as u32, dimension.height as u32)),
                        anchor_rect: Rectangle {
                            x: dimension.x,
                            y: dimension.y,
                            width: dimension.width,
                            height: dimension.height,
                        },
                        ..Default::default()
                    },
                    parent_size: None,
                    grab: true,
                    close_with_children: true,
                    input_zone: None,
                };
                return match self.popup {
                    None => {
                        let id = Id::unique();
                        self.popup = Some((type_id, id));
                        get_popup(settings(id))
                    }
                    Some((old_ty_id, id)) => match old_ty_id == type_id {
                        true => {
                            self.popup = None;
                            destroy_popup(id)
                        }
                        false => {
                            self.popup = Some((type_id, id));
                            destroy_popup(id).chain(get_popup(settings(id)))
                        }
                    },
                };
            }
            Message::Update(task) => {
                Arc::into_inner(task).unwrap().0(&mut self.registry);
            }
            Message::Action(task) => {
                Arc::into_inner(task).unwrap().0(&self.registry);
            }
            Message::GetConfig(sx) => sx
                .try_send((self.config_file.clone(), self.config.clone()))
                .unwrap(),
            Message::GetReceiver(sx, f) => sx.try_send(f(&self.registry)).unwrap(),
            Message::Spawn(cmd) => {
                Arc::into_inner(cmd)
                    .unwrap()
                    .spawn()
                    .inspect_err(|e| eprintln!("Failed to spawn command: {e}"))
                    .ok();
            }
            Message::ReloadConfig => {
                println!(
                    "Reloading config from {}",
                    self.config_file.to_string_lossy()
                );
                self.config =
                    read_config(&self.config_file, &mut self.registry, &mut self.templates).into();
                if self.config.hard_reload {
                    self.open = false;
                    return destroy_layer_surface(self.layer_id)
                        .chain(self.open())
                        .chain(Task::done(Message::LoadRegistry));
                }
            }
            Message::LoadRegistry => {
                self.registry = Registry::default();
                register_modules(&mut self.registry);
                register_listeners(&mut self.registry);
                register_resolvers(&mut self.registry);
                self.config =
                    read_config(&self.config_file, &mut self.registry, &mut self.templates).into();
                self.open = true;
            }
            Message::OutputsReady => {
                self.outputs_ready = true;
                return self.open();
            }
            Message::OutputEvent { event, wl_output } => match *event {
                OutputEvent::Created(info_maybe) => {
                    let first_output = self.outputs.is_empty();
                    self.outputs.push((wl_output, info_maybe));
                    if !self.outputs_ready && first_output {
                        return Task::future(async {
                            sleep(Duration::from_millis(500)).await;
                            Message::OutputsReady
                        });
                    }
                }
                OutputEvent::InfoUpdate(info) => {
                    if let Some((_, info_maybe)) =
                        self.outputs.iter_mut().find(|(wlo, _)| wlo == &wl_output)
                    {
                        *info_maybe = Some(info);
                    }
                }
                OutputEvent::Removed => {
                    let pos = self.outputs.iter().position(|(wlo, _)| wlo == &wl_output);
                    if let Some(pos) = pos {
                        self.outputs.remove(pos);
                    }
                }
            },
        }
        Task::none()
    }

    fn view(&self, window_id: Id) -> Element<'_, Message> {
        if window_id == self.layer_id {
            self.bar_view()
        } else if let Some(mod_id) = self
            .popup
            .and_then(|(m_id, p_id)| (p_id == window_id).then_some(m_id))
        {
            self.registry.get_module_by_id(mod_id).popup_wrapper(
                &self.config.popup_config,
                &self.config.anchor,
                &self.templates,
            )
        } else {
            "Internal error".into()
        }
    }

    fn bar_view(&self) -> Element<'_, Message> {
        let anchor = &self.config.anchor;
        let make_list = |spacing: fn(&Thrice<f32>) -> f32,
                         field: fn(&EnabledModules) -> &Vec<String>| {
            let modules = self
                .registry
                .get_modules(field(&self.config.enabled_modules).iter(), &self.config)
                .filter(|&m| m.active())
                .map(|m| {
                    m.wrapper(
                        &self.config.module_config.local,
                        m.view(
                            &self.config.module_config.local,
                            &self.config.popup_config,
                            anchor,
                            &self.templates,
                        ),
                        anchor,
                    )
                })
                .collect::<Vec<_>>();
            let content = if modules.is_empty() {
                vec![self.registry.get_module::<EmptyModule>().wrapper(
                    &self.config.module_config.local,
                    "".into(),
                    anchor,
                )]
            } else {
                modules
            };
            container(
                list(anchor, content).spacing(spacing(&self.config.module_config.global.spacing)),
            )
            .fillx(!anchor.vertical())
        };
        let left = make_list(|s| s.left, |m| &m.left);
        let center = make_list(|s| s.center, |m| &m.center);
        let right = make_list(|s| s.right, |m| &m.right);
        container(stack!(
            center.align(anchor, Alignment::Center),
            list(
                anchor,
                [(left, Alignment::Start), (right, Alignment::End)]
                    .map(|(e, align)| e.align(anchor, align).into())
            )
        ))
        .padding(self.config.module_config.global.padding)
        .into()
    }

    fn open(&self) -> Task<Message> {
        let (output, info) = match &self.config.monitor {
            Some(name) => self
                .outputs
                .iter()
                .find(|(_, info)| {
                    info.as_ref()
                        .is_some_and(|info| info.name.as_ref() == Some(name))
                })
                .map(|(o, info)| (IcedOutput::Output(o.clone()), info.as_ref()))
                .unwrap_or_else(|| {
                    eprintln!("No output with name {name} could be found!");
                    (IcedOutput::Active, None)
                }),
            None => (IcedOutput::Active, None),
        };

        let (x, y) = info
            .as_ref()
            .and_then(|i| i.logical_size.map(|(x, y)| (x as u32, y as u32)))
            .unwrap_or((1920, 1080));

        let (width, height) = match self.config.anchor.vertical() {
            true => (
                self.config.module_config.global.width.unwrap_or(30),
                self.config.module_config.global.height.unwrap_or(y),
            ),
            false => (
                self.config.module_config.global.width.unwrap_or(x),
                self.config.module_config.global.height.unwrap_or(30),
            ),
        };

        get_layer_surface(SctkLayerSurfaceSettings {
            layer: Layer::Top,
            keyboard_interactivity: self.config.kb_focus,
            anchor: (&self.config.anchor).into(),
            exclusive_zone: self.config.exclusive_zone(),
            size: Some((Some(width), Some(height))),
            namespace: "bar-rs".to_string(),
            output,
            margin: self.config.module_config.global.margin,
            id: self.layer_id,
            ..Default::default()
        })
    }

    fn theme(&self, window_id: Id) -> Theme {
        if let Some(mod_id) = self
            .popup
            .and_then(|(m_id, p_id)| (p_id == window_id).then_some(m_id))
        {
            self.registry.get_module_by_id(mod_id).popup_theme()
        } else {
            Theme::custom(
                "Bar theme".to_string(),
                Palette {
                    background: self.config.module_config.global.background_color,
                    text: Color::WHITE,
                    primary: Color::WHITE,
                    success: Color::WHITE,
                    warning: Color::WHITE,
                    danger: Color::WHITE,
                },
            )
        }
    }
}

trait OptionExt<T> {
    fn map_none<F>(self, f: F) -> Self
    where
        F: FnOnce();
}

impl<T> OptionExt<T> for Option<T> {
    fn map_none<F>(self, f: F) -> Self
    where
        F: FnOnce(),
    {
        if self.is_none() {
            f();
        }
        self
    }
}
