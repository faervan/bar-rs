use std::{path::PathBuf, process::{exit, Command}};

use chrono::Local;
use config::{get_config_dir, read_config, Config};
use hyprland::keyword::Keyword;
use iced::{alignment::Horizontal::{Left, Right}, daemon, theme::Palette, widget::{container, row, text, Row}, window::{self, settings::PlatformSpecific, Id, Level, Settings}, Alignment::Center, Background, Border, Color, Element, Font, Length::Fill, Padding, Size, Subscription, Task, Theme};
use iced::widget::text::{Rich, Span};
use modules::{battery::{battery_stats, BatteryStats}, cpu::cpu_usage, hyprland::{hyprland_events, reserve_bar_space, OpenWorkspaces}, playerctl::{playerctl, MediaStats}, sys_tray::system_tray, volume::{volume, VolumeStats}};
use tokio::sync::mpsc;

mod modules;
mod config;

const BAR_HEIGHT: u16 = 30;
const NERD_FONT: Font = Font::with_name("3270 Nerd Font");

fn main() -> iced::Result {
    daemon("Bar", Bar::update, Bar::view)
        .theme(Bar::theme)
        .font(include_bytes!("../assets/3270/3270NerdFont-Regular.ttf"))
        .subscription(|state| {
            let mut subs = vec![
                Subscription::run(cpu_usage),
                Subscription::run(volume),
                Subscription::run(playerctl),
                Subscription::run(hyprland_events),
                Subscription::run(system_tray),
            ];
            if state.config.show_batteries {
                subs.push(Subscription::run(battery_stats));
            }
            Subscription::batch(subs)
        })
        .run_with(Bar::new)
}

#[derive(Debug, Clone)]
enum Message {
    CPU(usize),
    Battery(BatteryStats),
    Volume(VolumeStats),
    Media(MediaStats),
    Workspaces(OpenWorkspaces),
    Window(Option<String>),
    GetConfig(mpsc::Sender<Config>),
    ToggleWindow,
    WindowOpened,
}

#[derive(Debug)]
struct Bar {
    _config_file: PathBuf,
    config: Config,
    opened: bool,
    data: ModuleData,
}

#[derive(Default, Debug)]
struct ModuleData {
    cpu_usage: usize,
    ram_usage: usize,
    battery: BatteryStats,
    volume: VolumeStats,
    media: MediaStats,
    workspaces: OpenWorkspaces,
    window: Option<String>,
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
                config,
                opened: true,
                data: ModuleData::default(),
            },
            task.map(|_| Message::WindowOpened)
        )
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        let data = &mut self.data;
        match msg {
            Message::CPU(perc) => {
                data.cpu_usage = perc;
                data.ram_usage = Command::new("sh")
                    .arg("-c")
                    .arg("free | grep Mem | awk '{printf \"%.0f\", $3/$2 * 100.0}'")
                    .output()
                    .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to get memory usage. err: {e}");
                        "0".to_string()
                    })
                    .parse()
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to parse memory usage (output from free), e: {e}");
                        999
                    });
            }
            Message::Battery(stats) => data.battery = stats,
            Message::Volume(stats) => data.volume = stats,
            Message::Media(stats) => data.media = stats,
            Message::Workspaces(ws) => data.workspaces = ws,
            Message::Window(window) => data.window = window,
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
        let left = row![
            self.workspaces(),
            self.window()
        ];

        let right = row![
            self.media(),
            self.volume()
        ]
            .push_maybe(
                self.config.show_batteries
                    .then_some(self.battery())
            )
            .push(self.cpu())
            .push(self.memory());

        row(
            [
                (left, Left),
                (self.time(), Center.into()),
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

    fn workspaces(&self) -> Row<Message> {
        row(
            self.data.workspaces.open
                .iter()
                .enumerate()
                .map(|(id, ws)| {
                    let mut span = Span::new(ws)
                        .size(20)
                        .padding(Padding {top: -3., bottom: 0., right: 10., left: 5.})
                        .font(NERD_FONT);
                    if id == self.data.workspaces.active {
                        span = span
                            .background(Background::Color(Color::WHITE).scale_alpha(0.5))
                            .border(Border::default().rounded(8))
                            .color(Color::BLACK);
                    }
                    Rich::with_spans([span])
                        .center()
                        .height(Fill)
                        .into()
                })
        ).spacing(15)
    }

    fn window(&self) -> Row<Message> {
        row![
            text![
                "{}",
                self.data.window.as_ref()
                    .unwrap_or(&"".to_string())
            ].center().height(Fill)
        ]
    }

    fn time(&self) -> Row<Message> {
        let time = Local::now();
        row![
            text!("")
                .center().height(Fill).size(20).font(NERD_FONT),
            text![
                " {}", time.format("%a, %d. %b  ")
            ].center().height(Fill),
            text!("")
                .center().height(Fill).size(25).font(NERD_FONT),
            text![
                " {}", time.format("%H:%M")
            ].center().height(Fill),
        ].spacing(10)
    }

    fn media(&self) -> Row<Message> {
        let data = &self.data;
        row![
            text!("{}", data.media.icon)
                .center().height(Fill).size(20).font(NERD_FONT),
            text![
                "{}{}",
                data.media.title,
                data.media.artist.as_ref()
                    .map(|name| format!(" - {name}"))
                    .unwrap_or("".to_string())
            ].center().height(Fill)
        ].spacing(15)
    }

    fn volume(&self) -> Row<Message> {
        row![
            text!("{}", self.data.volume.icon)
                .center().height(Fill).size(20).font(NERD_FONT),
            text![
                "{}%",
                self.data.volume.level,
            ].center().height(Fill)
        ].spacing(10)
    }

    fn battery(&self) -> Row<Message> {
        let data = &self.data;
        row![
            text!("{}", data.battery.icon)
                .center().height(Fill).size(20).font(NERD_FONT),
            text![
                " {}% ({}h {}min left)",
                data.battery.capacity,
                data.battery.hours,
                data.battery.minutes
            ].center().height(Fill)
        ]
    }

    fn cpu(&self) -> Row<Message> {
        row![
            text!("󰻠")
                .center().height(Fill).size(20).font(NERD_FONT),
            text![
                "{}%", self.data.cpu_usage
            ].center().height(Fill),
        ].spacing(10)
    }

    fn memory(&self) -> Row<Message> {
        row![
            text!("󰍛")
                .center().height(Fill).size(20).font(NERD_FONT),
            text![
                "{}%", self.data.ram_usage
            ].center().height(Fill)
        ].spacing(10)
    }
}
