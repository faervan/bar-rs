use std::{fmt::Debug, path::PathBuf, process::exit, sync::Arc};

use config::{get_config_dir, read_config, Config, EnabledModules};
use hyprland::keyword::Keyword;
use iced::{alignment::Horizontal::{Left, Right}, daemon, theme::Palette, widget::{container, row}, window::{self, settings::PlatformSpecific, Id, Level, Settings}, Alignment::Center, Color, Element, Font, Length::Fill, Size, Subscription, Task, Theme};
use listeners::register_listeners;
use modules::{hyprland::reserve_bar_space, register_modules};
use registry::Registry;
use tokio::sync::mpsc;

mod modules;
mod listeners;
mod config;
mod registry;

const BAR_HEIGHT: u16 = 30;
const NERD_FONT: Font = Font::with_name("3270 Nerd Font");

fn main() -> iced::Result {
    daemon("Bar", Bar::update, Bar::view)
        .theme(Bar::theme)
        .font(include_bytes!("../assets/3270/3270NerdFont-Regular.ttf"))
        .subscription(
            |state| Subscription::batch({
                state.registry
                    .get_modules(
                        state.config.enabled_modules.get_all()
                    )
                    .filter_map(|m|
                        state.config.enabled_modules
                            .contains(&m.id())
                            .then(|| m.subscription())
                    )
                    .flatten()
                    .chain(
                        state.registry
                            .get_listeners(&state.config.enabled_listeners)
                            .map(|l| l.subscription())
                    )
            })
        )
        .run_with(Bar::new)
}

#[derive(Clone)]
struct UpdateFn(Arc<Box<dyn FnOnce(&mut Registry) + Send + Sync>>);
impl Debug for UpdateFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Arc<Box<dyn FnOnce(&mut Registry)>> can't be displayed")
    }
}

#[derive(Debug, Clone)]
enum Message {
    Update(UpdateFn),
    Perform(fn(&mut Registry, &Arc<Config>, &mut bool) -> Task<Message>),
    GetConfig(mpsc::Sender<(Arc<PathBuf>, Arc<Config>)>),
    ReloadConfig,
    WindowOpened,
}

impl Message {
    fn update(closure: Box<dyn FnOnce(&mut Registry) + Send + Sync>) -> Self {
        Message::Update(UpdateFn(Arc::new(closure)))
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
        let config = read_config(&config_file, &registry);

        let monitor = config.monitor.clone();
        reserve_bar_space(&monitor);

        ctrlc::set_handler(move || {
            Keyword::set("monitor", format!("{monitor}, addreserved, 0, 0, 0, 0"))
                .expect("Failed to clear reserved space using hyprctl");
            exit(0);
        }).expect("Failed to exec exit handler");

        let (_window_id, task) = Self::open_window();

        (
            Self {
                config_file: config_file.into(),
                config: config.into(),
                opened: true,
                registry,
            },
            task.map(|_| Message::WindowOpened)
        )
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Update(task) => {
                Arc::into_inner(task.0).unwrap()(&mut self.registry);
            }
            Message::Perform(task) => return task(&mut self.registry, &self.config, &mut self.opened),
            Message::GetConfig(sx) => sx.try_send((
                self.config_file.clone(),
                self.config.clone()
            )).unwrap(),
            Message::ReloadConfig => {
                println!("Reloading config from {}", self.config_file.to_string_lossy());
                self.config = read_config(&self.config_file, &self.registry).into();
            }
            Message::WindowOpened => {}
        }
        Task::none()
    }

    fn view(&self, _window_id: Id) -> Element<Message> {
        let make_row = |field: fn(&EnabledModules) -> &Vec<String>|
            row(self.registry
                .get_modules(
                    field(&self.config.enabled_modules).iter()
                )
                .map(|m| m.view())
            );
        let left = make_row(|m| &m.left);
        let center = make_row(|m| &m.center);
        let right = make_row(|m| &m.right);
        row(
            [
                (left, Left),
                (center, Center.into()),
                (right, Right)
            ].map(|(row, alignment)|
                container(
                    row.spacing(20)
                )
                .width(Fill)
                .align_x(alignment)
                .into()
            )
        ).padding([0, 10]).into()
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
        Theme::custom("Custom".to_string(), Palette {
            background: Color::from_rgba(0., 0., 0., 0.5),
            text: Color::BLACK,
            primary: Color::BLACK,
            success: Color::WHITE,
            danger: Color::BLACK,
        })
    }
}

trait OptionExt<T> {
    fn map_none<F>(self, f: F) -> Self
        where F: FnOnce();
}

impl<T> OptionExt<T> for Option<T> {
    fn map_none<F>(self, f: F) -> Self
            where F: FnOnce() {
        if self.is_none() {
            f();
        }
        self
    }
}
