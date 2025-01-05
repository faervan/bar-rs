use std::{fmt::Debug, path::PathBuf, process::exit, sync::Arc};

use config::{get_config_dir, read_config, Config, EnabledModules, Thrice};
use hyprland::keyword::Keyword;
use iced::{
    alignment::Horizontal::{Left, Right},
    daemon,
    theme::Palette,
    widget::{container, row},
    window::{self, settings::PlatformSpecific, Id, Level, Settings},
    Alignment::Center,
    Color, Element, Font,
    Length::Fill,
    Size, Subscription, Task, Theme,
};
use listeners::register_listeners;
use modules::{hyprland::reserve_bar_space, register_modules};
use registry::Registry;
use tokio::sync::mpsc;

mod config;
mod listeners;
mod modules;
mod registry;

const BAR_HEIGHT: u16 = 30;
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
                    .filter(|m| state.config.enabled_modules.contains(&m.id()))
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
            "Box<dyn FnOnce(&mut Registry) + Send + Sync> can't be displayed"
        )
    }
}

#[derive(Debug, Clone)]
enum Message {
    Update(Arc<UpdateFn>),
    Perform(fn(&mut Registry, &Arc<Config>, &mut bool) -> Task<Message>),
    GetConfig(mpsc::Sender<(Arc<PathBuf>, Arc<Config>)>),
    ReloadConfig,
    WindowOpened,
}

impl Message {
    fn update(closure: Box<dyn FnOnce(&mut Registry) + Send + Sync>) -> Self {
        Message::Update(Arc::new(UpdateFn(closure)))
    }
}

#[derive(Debug)]
struct Bar {
    config_file: Arc<PathBuf>,
    config: Arc<Config>,
    opened: bool,
    registry: Registry,
}

impl Bar {
    fn new() -> (Self, Task<Message>) {
        let mut registry = Registry::default();
        register_modules(&mut registry);
        register_listeners(&mut registry);

        let config_file = get_config_dir(&registry);
        let config = read_config(&config_file, &mut registry);

        let monitor = config.monitor.clone();
        reserve_bar_space(&monitor);

        ctrlc::set_handler(move || {
            Keyword::set("monitor", format!("{monitor}, addreserved, 0, 0, 0, 0"))
                .expect("Failed to clear reserved space using hyprctl");
            exit(0);
        })
        .expect("Failed to exec exit handler");

        let (_window_id, task) = Self::open_window();

        (
            Self {
                config_file: config_file.into(),
                config: config.into(),
                opened: true,
                registry,
            },
            task.map(|_| Message::WindowOpened),
        )
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Update(task) => {
                Arc::into_inner(task).unwrap().0(&mut self.registry);
            }
            Message::Perform(task) => {
                return task(&mut self.registry, &self.config, &mut self.opened)
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
            Message::WindowOpened => {}
        }
        Task::none()
    }

    fn view(&self, _window_id: Id) -> Element<Message> {
        let make_row = |spacing: fn(&Thrice<f32>) -> f32,
                        field: fn(&EnabledModules) -> &Vec<String>| {
            container(
                row(self
                    .registry
                    .get_modules(field(&self.config.enabled_modules).iter())
                    .map(|m| m.view(&self.config.module_config.local)))
                .spacing(spacing(&self.config.module_config.global.spacing)),
            )
            .width(Fill)
        };
        let left = make_row(|s| s.left, |m| &m.left);
        let center = make_row(|s| s.center, |m| &m.center);
        let right = make_row(|s| s.right, |m| &m.right);
        row([(left, Left), (center, Center.into()), (right, Right)]
            .map(|(row, alignment)| row.align_x(alignment).into()))
        .padding([0, 10])
        .into()
    }

    fn open_window() -> (Id, Task<Id>) {
        window::open(Settings {
            transparent: true,
            decorations: false,
            icon: None,
            resizable: false,
            level: Level::AlwaysOnTop,
            size: Size::new(1920., BAR_HEIGHT as f32),
            platform_specific: PlatformSpecific {
                application_id: "bar-rs".to_string(),
                override_redirect: false,
            },
            ..Default::default()
        })
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
