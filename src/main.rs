use std::{fmt::Debug, path::PathBuf, process::exit, sync::Arc, time::Duration};

use config::{get_config_dir, read_config, Config, EnabledModules, Thrice};
use fill::FillExt;
use iced::{
    daemon,
    platform_specific::shell::commands::{
        layer_surface::{get_layer_surface, KeyboardInteractivity, Layer},
        output::{get_output, get_output_info, OutputInfo},
    },
    runtime::platform_specific::wayland::layer_surface::{IcedOutput, SctkLayerSurfaceSettings},
    stream,
    theme::Palette,
    widget::{container, stack},
    window::Id,
    Alignment, Color, Element, Font, Subscription, Task, Theme,
};
use list::{list, DynamicAlign};
use listeners::register_listeners;
use modules::register_modules;
use registry::Registry;
use tokio::{sync::mpsc, time::sleep};

mod config;
#[macro_use]
mod list;
mod fill;
mod listeners;
mod modules;
mod registry;

const NERD_FONT: Font = Font::with_name("3270 Nerd Font");

fn main() -> iced::Result {
    daemon("Bar", Bar::update, Bar::view)
        .theme(Bar::theme)
        .font(include_bytes!("../assets/3270/3270NerdFont-Regular.ttf"))
        .subscription(|state| {
            Subscription::batch({
                state
                    .registry
                    .get_modules(state.config.enabled_modules.get_all())
                    .filter(|m| state.config.enabled_modules.contains(&m.name()))
                    .filter_map(|m| m.subscription())
                    .chain(
                        state
                            .registry
                            .get_listeners(&state.config.enabled_listeners)
                            .map(|l| l.subscription()),
                    )
            })
        })
        .run_with(Bar::new)
}

struct UpdateFn(Box<dyn FnOnce(&mut Registry) + Send + Sync>);
impl Debug for UpdateFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UpdateFn(Box<dyn FnOnce(&mut Registry) + Send + Sync>) can't be displayed"
        )
    }
}

#[derive(Debug, Clone)]
enum Message {
    Update(Arc<UpdateFn>),
    GetConfig(mpsc::Sender<(Arc<PathBuf>, Arc<Config>)>),
    ReloadConfig,
    GotOutput(Option<IcedOutput>),
    GotOutputInfo(Option<OutputInfo>),
}

impl Message {
    fn update<F>(f: F) -> Self
    where
        F: FnOnce(&mut Registry) + Send + Sync + 'static,
    {
        Message::Update(Arc::new(UpdateFn(Box::new(f))))
    }
}

#[derive(Debug)]
struct Bar {
    config_file: Arc<PathBuf>,
    config: Arc<Config>,
    registry: Registry,
    logical_size: Option<(u32, u32)>,
    output: IcedOutput,
}

impl Bar {
    fn new() -> (Self, Task<Message>) {
        let mut registry = Registry::default();
        register_modules(&mut registry);
        register_listeners(&mut registry);

        let config_file = get_config_dir();
        let config = read_config(&config_file, &mut registry);

        ctrlc::set_handler(|| {
            println!("Received exit signal...Exiting");
            exit(0);
        })
        .unwrap();

        let bar = Self {
            config_file: config_file.into(),
            config: config.into(),
            registry,
            logical_size: None,
            output: IcedOutput::Active,
        };
        let task = match &bar.config.monitor {
            Some(_) => bar.try_get_output(),
            None => bar.open(),
        };

        (bar, task)
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Update(task) => {
                Arc::into_inner(task).unwrap().0(&mut self.registry);
            }
            Message::GetConfig(sx) => sx
                .try_send((self.config_file.clone(), self.config.clone()))
                .unwrap(),
            Message::ReloadConfig => {
                println!(
                    "Reloading config from {}",
                    self.config_file.to_string_lossy()
                );
                self.config = read_config(&self.config_file, &mut self.registry).into();
            }
            Message::GotOutput(optn) => {
                return match optn {
                    Some(output) => {
                        self.output = output;
                        self.try_get_output_info()
                    }
                    None => Task::stream(stream::channel(1, |_| async {
                        sleep(Duration::from_millis(500)).await;
                    }))
                    .chain(self.try_get_output()),
                }
            }
            Message::GotOutputInfo(optn) => {
                return match optn {
                    Some(info) => {
                        self.logical_size = info.logical_size.map(|(x, y)| (x as u32, y as u32));
                        self.open()
                    }
                    None => Task::stream(stream::channel(1, |_| async {
                        sleep(Duration::from_millis(500)).await;
                    }))
                    .chain(self.try_get_output_info()),
                }
            }
        }
        Task::none()
    }

    fn view(&self, _window_id: Id) -> Element<Message> {
        let anchor = &self.config.anchor;
        let make_list = |spacing: fn(&Thrice<f32>) -> f32,
                         field: fn(&EnabledModules) -> &Vec<String>| {
            container(
                list(
                    &self.config.anchor,
                    self.registry
                        .get_modules(field(&self.config.enabled_modules).iter())
                        .map(|m| m.view(&self.config.module_config.local, &self.config.anchor)),
                )
                .spacing(spacing(&self.config.module_config.global.spacing)),
            )
            .fill(anchor)
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
        .padding(match self.config.anchor.vertical() {
            true => [20, 5],
            false => [0, 10],
        })
        .into()
    }

    fn open(&self) -> Task<Message> {
        let (x, y) = self.logical_size.unwrap_or((1920, 1080));
        let (width, height) = match self.config.anchor.vertical() {
            true => (
                self.config.bar_width.unwrap_or(30),
                self.config.bar_height.unwrap_or(y),
            ),
            false => (
                self.config.bar_width.unwrap_or(x),
                self.config.bar_height.unwrap_or(30),
            ),
        };
        get_layer_surface(SctkLayerSurfaceSettings {
            layer: Layer::Top,
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            anchor: (&self.config.anchor).into(),
            exclusive_zone: self.config.exclusive_zone(),
            size: Some((Some(width), Some(height))),
            namespace: "bar-rs".to_string(),
            output: self.output.clone(),
            ..Default::default()
        })
    }

    fn try_get_output(&self) -> Task<Message> {
        let monitor = self.config.monitor.clone();
        get_output(move |output_state| {
            output_state
                .outputs()
                .find(|o| {
                    output_state
                        .info(o)
                        .map(|info| info.name == monitor)
                        .unwrap_or(false)
                })
                .clone()
        })
        .map(|optn| Message::GotOutput(optn.map(IcedOutput::Output)))
    }

    fn try_get_output_info(&self) -> Task<Message> {
        let monitor = self.config.monitor.clone();
        get_output_info(move |output_state| {
            output_state
                .outputs()
                .find(|o| {
                    output_state
                        .info(o)
                        .map(|info| info.name == monitor)
                        .unwrap_or(false)
                })
                .and_then(|o| output_state.info(&o))
                .clone()
        })
        .map(Message::GotOutputInfo)
    }

    fn theme(&self, _window_id: Id) -> Theme {
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
