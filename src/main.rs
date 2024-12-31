use std::{collections::HashMap, path::PathBuf, process::exit, sync::Arc};

use config::{get_config_dir, read_config, Config, EnabledModules};
use hyprland::keyword::Keyword;
use iced::{alignment::Horizontal::{Left, Right}, daemon, theme::Palette, widget::{container, row}, window::{self, settings::PlatformSpecific, Id, Level, Settings}, Alignment::Center, Color, Element, Font, Length::Fill, Size, Subscription, Task, Theme};
use modules::{all_modules, hyprland::reserve_bar_space, Module};
use tokio::sync::mpsc;

mod modules;
mod config;

const BAR_HEIGHT: u16 = 30;
const NERD_FONT: Font = Font::with_name("3270 Nerd Font");

fn main() -> iced::Result {
    daemon("Bar", Bar::update, Bar::view)
        .theme(Bar::theme)
        .font(include_bytes!("../assets/3270/3270NerdFont-Regular.ttf"))
        .subscription(
            |state| Subscription::batch(
                state.get_modules(None)
                    .filter_map(|m| m.subscription())
            )
        )
        .run_with(Bar::new)
}

#[derive(Debug, Clone)]
enum Message {
    UpdateModule {
        id: String,
        data: Arc<dyn Module>,
    },
    GetConfig(mpsc::Sender<Arc<Config>>),
    ToggleWindow,
    WindowOpened,
}

#[derive(Debug)]
struct Bar {
    _config_file: PathBuf,
    config: Arc<Config>,
    opened: bool,
    modules: HashMap<String, Arc<dyn Module>>,
}

impl Bar {
    fn new() -> (Self, Task<Message>) {
        let config_file = get_config_dir();
        let config = read_config(&config_file);

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
                _config_file: config_file,
                config: config.into(),
                opened: true,
                modules: all_modules(),
            },
            task.map(|_| Message::WindowOpened)
        )
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::UpdateModule { id, data } => {
                *self.modules.get_mut(&id).unwrap() = data;
            }
            Message::GetConfig(sx) => sx.try_send(self.config.clone()).unwrap(),
            Message::ToggleWindow => {
                self.opened = !self.opened;
                return match self.opened {
                    true => Self::open_window().1.map(|_| Message::WindowOpened),
                    false => window::get_latest().and_then(window::close)
                }
            }
            Message::WindowOpened => {}
        }
        Task::none()
    }

    fn view(&self, _window_id: Id) -> Element<Message> {
        let make_row = |get: fn(fn(&Vec<String>) -> Vec<&String>, &EnabledModules) -> Vec<&String>| row(
            self.get_modules(Some(get))
                .map(|m| m.view())
        );
        let left = make_row(|into, m| into(&m.left));
        let center = make_row(|into, m| into(&m.center));
        let right = make_row(|into, m| into(&m.right));
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

    // I like it :)
    fn get_modules(&self, get: Option<fn(fn(&Vec<String>) -> Vec<&String>, &EnabledModules) -> Vec<&String>>)
        -> impl Iterator<Item = &Arc<dyn Module>> {
        get.unwrap_or(|_, m| m.get_all().collect())(
            |list| list.iter().collect(),
            &self.config.enabled_modules
        ).into_iter()
            .filter_map(|module|
                self.modules
                    .get(module)
                    .map_none(|| eprintln!("No module with name {module} found!"))
            )
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
