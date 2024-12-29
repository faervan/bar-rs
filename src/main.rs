use std::{path::PathBuf, process::{exit, Command}};

use chrono::Local;
use config::{get_config_dir, read_config, Config};
use hyprland::keyword::Keyword;
use iced::{alignment::Horizontal::{Left, Right}, application, theme::Palette, widget::{container, row, text, Row}, window::{settings::PlatformSpecific, Level, Settings}, Alignment::Center, Background, Border, Color, Element, Font, Length::Fill, Padding, Size, Subscription, Task, Theme};
use iced::widget::text::{Rich, Span};
use modules::{battery::{battery_stats, BatteryStats}, cpu::cpu_usage, hyprland::{hyprland_events, reserve_bar_space, OpenWorkspaces}, playerctl::{playerctl, MediaStats}, sys_tray::system_tray, volume::{volume, VolumeStats}};
use tokio::sync::mpsc;

mod modules;
mod config;

const BAR_HEIGHT: u16 = 30;
const NERD_FONT: Font = Font::with_name("3270 Nerd Font");

fn main() -> iced::Result {
    application("Bar", Bar::update, Bar::view)
        .theme(|bar| Theme::custom("Custom".to_string(), Palette {
            background: bar.background_color(),
            text: Color::BLACK,
            primary: Color::BLACK,
            success: Color::WHITE,
            danger: Color::BLACK,
        }))
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
        .window(Settings {
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
}

#[derive(Default, Debug)]
struct Bar {
    cpu_usage: usize,
    ram_usage: usize,
    battery: BatteryStats,
    volume: VolumeStats,
    media: MediaStats,
    workspaces: OpenWorkspaces,
    window: Option<String>,
    _config_file: PathBuf,
    config: Config,
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

        (
            Self {
                _config_file: config_file,
                config,
                ..Default::default()
            },
            Task::none()
        )
    }

    fn update(&mut self, msg: Message) {
        match msg {
            Message::CPU(perc) => {
                self.cpu_usage = perc;
                self.ram_usage = Command::new("sh")
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
            Message::Battery(stats) => self.battery = stats,
            Message::Volume(stats) => self.volume = stats,
            Message::Media(stats) => self.media = stats,
            Message::Workspaces(ws) => self.workspaces = ws,
            Message::Window(window) => self.window = window,
            Message::GetConfig(sx) => sx.try_send(self.config.clone()).unwrap(),
        }
    }

    fn view(&self) -> Element<Message> {
        let time = Local::now();

        let left = row![
            // Workspace
            row(
                self.workspaces.open
                    .iter()
                    .enumerate()
                    .map(|(id, ws)| {
                        let mut span = Span::new(ws)
                            .size(20)
                            .padding(Padding {top: -3., bottom: 0., right: 10., left: 5.})
                            .font(NERD_FONT);
                        if id == self.workspaces.active {
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
            ).spacing(15),

            // Window
            row![
                text![
                    "{}",
                    self.window.as_ref()
                        .unwrap_or(&"".to_string())
                ].center().height(Fill)
            ]
        ];

        let center = row![
            // Time
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
            ].spacing(10),
        ];

        let right = row![
            // Media
            row![
                text!("{}", self.media.icon)
                    .center().height(Fill).size(20).font(NERD_FONT),
                text![
                    "{}{}",
                    self.media.title,
                    self.media.artist.as_ref()
                        .map(|name| format!(" - {name}"))
                        .unwrap_or("".to_string())
                ].center().height(Fill)
            ].spacing(15),

            // Volume
            row![
                text!("{}", self.volume.icon)
                    .center().height(Fill).size(20).font(NERD_FONT),
                text![
                    "{}%",
                    self.volume.level,
                ].center().height(Fill)
            ].spacing(10),

        ].push_maybe(
            self.config.show_batteries
                .then_some(self.battery())
        ).push(
            // CPU
            row![
                text!("󰻠")
                    .center().height(Fill).size(20).font(NERD_FONT),
                text![
                    "{}%", self.cpu_usage
                ].center().height(Fill),
            ].spacing(10)
        ).push(
            // Memory
            row![
                text!("󰍛")
                    .center().height(Fill).size(20).font(NERD_FONT),
                text![
                    "{}%", self.ram_usage
                ].center().height(Fill)
            ].spacing(10),
        );

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

    fn battery(&self) -> Row<Message> {
        row![
            text!("{}", self.battery.icon)
                .center().height(Fill).size(20).font(NERD_FONT),
            text![
                " {}% ({}h {}min left)",
                self.battery.capacity,
                self.battery.hours,
                self.battery.minutes
            ].center().height(Fill)
        ]
    }

    fn background_color(&self) -> Color {
        Color::from_rgba(0., 0., 0., 0.5)
    }
}
