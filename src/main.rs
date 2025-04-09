use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    path::PathBuf,
    process::{exit, Command},
    sync::Arc,
    time::Duration,
};

use config::{anchor::BarAnchor, get_config_dir, read_config, Config, EnabledModules, Thrice};
use fill::FillExt;
use iced::{
    daemon,
    event::{
        listen_with,
        wayland::{self, OutputEvent},
        PlatformSpecific,
    },
    platform_specific::shell::commands::{
        layer_surface::{destroy_layer_surface, get_layer_surface, Layer},
        popup::{destroy_popup, get_popup},
    },
    runtime::platform_specific::wayland::{
        layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
        popup::{SctkPopupSettings, SctkPositioner},
    },
    stream,
    theme::Palette,
    widget::{container, stack},
    window::Id,
    Alignment, Color, Element, Font, Rectangle, Subscription, Task, Theme,
};
use list::{list, DynamicAlign};
use listeners::register_listeners;
use modules::{register_modules, Module};
use registry::Registry;
use resolvers::register_resolvers;
use smithay_client_toolkit::{
    output::OutputInfo, reexports::client::protocol::wl_output::WlOutput,
};
use template_engine::TemplateEngine;
use tokio::{
    sync::{broadcast, mpsc},
    time::sleep,
};

mod config;
#[macro_use]
mod list;
mod button;
mod fill;
#[macro_use]
mod helpers;
mod listeners;
mod modules;
mod registry;
mod resolvers;
mod template_engine;

const NERD_FONT: Font = Font::with_name("3270 Nerd Font");

fn main() -> iced::Result {
    daemon("Bar", Bar::update, Bar::view)
        .theme(Bar::theme)
        .font(include_bytes!("../assets/3270/3270NerdFont-Regular.ttf"))
        .subscription(|state| {
            Subscription::batch([
                listen_with(|event, _, _| {
                    if let iced::Event::PlatformSpecific(PlatformSpecific::Wayland(
                        wayland::Event::Output(event, wl_output),
                    )) = event
                    {
                        Some(Message::OutputEvent { event, wl_output })
                    } else {
                        None
                    }
                }),
                if state.open {
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
                    })
                } else {
                    Subscription::none()
                },
            ])
        })
        .run_with(Bar::new)
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
        event: OutputEvent,
        wl_output: WlOutput,
    },
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
    fn popup<'a>(
        id: TypeId,
        width: i32,
        height: i32,
        anchor: &BarAnchor,
    ) -> impl Fn(
        iced::Event,
        iced::core::Layout,
        iced::mouse::Cursor,
        &mut dyn iced::core::Clipboard,
        &Rectangle,
    ) -> Message
           + 'a {
        let anchor = *anchor;
        move |_: iced::Event,
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
                type_id: id,
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

#[derive(Debug)]
struct Bar {
    config_file: Arc<PathBuf>,
    config: Arc<Config>,
    registry: Registry,
    outputs: HashMap<WlOutput, Option<OutputInfo>>,
    layer_id: Id,
    open: bool,
    popup: Option<(TypeId, Id)>,
    engine: TemplateEngine,
}

impl Bar {
    fn new() -> (Self, Task<Message>) {
        let mut registry = Registry::default();
        register_modules(&mut registry);
        register_listeners(&mut registry);
        register_resolvers(&mut registry);

        let mut engine = TemplateEngine::new();

        let config_file = get_config_dir();
        let config = read_config(&config_file, &mut registry, &mut engine);

        ctrlc::set_handler(|| {
            println!("Received exit signal...Exiting");
            exit(0);
        })
        .unwrap();

        let bar = Self {
            config_file: config_file.into(),
            config: config.into(),
            registry,
            outputs: HashMap::new(),
            layer_id: Id::unique(),
            open: false,
            popup: None,
            engine,
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
                    close_with_children: false,
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
                self.registry
                    .get_modules(self.config.enabled_modules.get_all(), &self.config)
                    .for_each(|m| {
                        self.engine.set_context(
                            m.as_any().type_id(),
                            m.context(),
                            m.extra_context(),
                        );
                    });
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
                    read_config(&self.config_file, &mut self.registry, &mut self.engine).into();
                if self.config.hard_reload {
                    self.open = false;
                    self.registry = Registry::default();
                    return destroy_layer_surface(self.layer_id)
                        .chain(self.open())
                        .chain(Task::done(Message::LoadRegistry));
                }
            }
            Message::LoadRegistry => {
                register_modules(&mut self.registry);
                register_listeners(&mut self.registry);
                register_resolvers(&mut self.registry);
                self.config =
                    read_config(&self.config_file, &mut self.registry, &mut self.engine).into();
                self.open = true;
            }
            Message::OutputEvent { event, wl_output } => match event {
                OutputEvent::Created(info_maybe) => {
                    self.outputs.insert(wl_output, info_maybe);
                    //println!("new output! {:#?}", self.outputs);
                    if !self.open {
                        self.open = true;
                        return Task::stream(stream::channel(1, |_| async {
                            sleep(Duration::from_millis(500)).await;
                        }))
                        .chain(self.open());
                    }
                }
                OutputEvent::InfoUpdate(info) => {
                    self.outputs.insert(wl_output, Some(info));
                }
                OutputEvent::Removed => {
                    self.outputs.remove(&wl_output);
                }
            },
        }
        Task::none()
    }

    fn view<'a>(&'a self, window_id: Id) -> Element<'a, Message> {
        if window_id == self.layer_id {
            self.bar_view()
        } else if let Some(mod_id) = self
            .popup
            .and_then(|(m_id, p_id)| (p_id == window_id).then_some(m_id))
        {
            self.registry.get_module_by_id(mod_id).popup_wrapper(
                &self.config.popup_config,
                &self.config.anchor,
                &self.engine,
            )
        } else {
            "Internal error".into()
        }
    }

    fn bar_view(&self) -> Element<Message> {
        let anchor = &self.config.anchor;
        let make_list = |spacing: fn(&Thrice<f32>) -> f32,
                         field: fn(&EnabledModules) -> &Vec<String>| {
            container(
                list(
                    anchor,
                    self.registry
                        .get_modules(field(&self.config.enabled_modules).iter(), &self.config)
                        .filter(|&m| m.active())
                        .map(|m| {
                            m.module_wrapper(
                                &self.config.module_config.local,
                                &self.config.popup_config,
                                anchor,
                                &self.engine,
                            )
                        }),
                )
                .spacing(spacing(&self.config.module_config.global.spacing)),
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
        let (output, info) = self
            .outputs
            .iter()
            .find(|(_, info)| {
                info.as_ref()
                    .is_some_and(|info| info.name == self.config.monitor)
            })
            .map(|(o, info)| (IcedOutput::Output(o.clone()), info))
            .unwrap_or_else(|| {
                if let Some(m) = self.config.monitor.as_ref() {
                    eprintln!("No output with name {m} could be found!");
                }
                (IcedOutput::Active, &None)
            });
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
