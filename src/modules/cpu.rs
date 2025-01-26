use std::{
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::{self, BufRead, BufReader},
    num,
    time::Duration,
};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::widget::{button::Style, container, scrollable};
use iced::{futures::SinkExt, stream, widget::text, Element, Subscription};
use tokio::time::sleep;

use crate::{
    button::button,
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    impl_wrapper, Message, NERD_FONT,
};

use super::Module;

#[derive(Debug, Default, Builder)]
pub struct CpuMod {
    avg_usage: CpuStats<u8>,
    _cores: HashMap<u8, CpuStats<u8>>,
    cfg_override: ModuleConfigOverride,
    icon: Option<String>,
}

impl Module for CpuMod {
    fn name(&self) -> String {
        "cpu".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<Message> {
        button(
            list![
                anchor,
                container(
                    text!("{}", self.icon.as_ref().unwrap_or(&"󰻠".to_string()))
                        .fill(anchor)
                        .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                        .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                        .font(NERD_FONT)
                )
                .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin)),
                container(
                    text!["{}%", self.avg_usage.all]
                        .fill(anchor)
                        .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                        .color(self.cfg_override.text_color.unwrap_or(config.text_color))
                )
                .padding(self.cfg_override.text_margin.unwrap_or(config.text_margin)),
            ]
            .spacing(self.cfg_override.spacing.unwrap_or(config.spacing)),
        )
        .on_event_with(Message::popup::<Self>(250, 250))
        .style(|_, _| Style::default())
        .into()
    }

    fn popup_view(&self) -> Element<Message> {
        /*container(scrollable(column(self.batteries.iter().map(|bat| {
            text!(
                "{}: {}\n\t{} {}% ({} Wh)\n\thealth: {}%{}\n\tmodel: {}",
                bat.name,
                bat.state,
                bat.icon(self),
                bat.capacity(),
                bat.energy_now.floor() as u32 / 1000000,
                bat.health,
                bat.remaining.map_or(Default::default(), |(h, m)| format!(
                    "\n\t{h}h {m}min remaining"
                )),
                bat.model_name,
            )
            .into()
        }))))
        .padding([10, 20])
        .style(|_| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 0.8,
            })),
            border: iced::Border::default().rounded(8),
            ..Default::default()
        })
        .into()*/
    }

    impl_wrapper!();

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _templates: &mut Handlebars,
    ) {
        self.cfg_override = config.into();
        self.icon = config.get("icon").and_then(|v| v.clone());
    }

    fn subscription(&self) -> Option<iced::Subscription<Message>> {
        Some(Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let interval: u64 = 500;
                let gap: u64 = 2000;
                loop {
                    let Ok(raw_stats1) = read_raw_stats()
                        .map_err(|e| eprintln!("Failed to read cpu stats from /proc/stat: {e:?}"))
                    else {
                        return;
                    };
                    sleep(Duration::from_millis(interval)).await;
                    let Ok(raw_stats2) = read_raw_stats() else {
                        eprintln!("Failed to read cpu stats from /proc/stat");
                        return;
                    };

                    let stats = (
                        raw_stats1.get(&CpuType::All).unwrap(),
                        raw_stats2.get(&CpuType::All).unwrap(),
                    )
                        .into();

                    sender
                        .send(Message::update(move |reg| {
                            reg.get_module_mut::<CpuMod>().avg_usage = stats
                        }))
                        .await
                        .unwrap_or_else(|err| {
                            eprintln!("Trying to send cpu_usage failed with err: {err}");
                        });

                    sleep(Duration::from_millis(gap)).await;
                }
            })
        }))
    }
}

#[derive(Default, Hash, PartialEq, Eq)]
enum CpuType {
    #[default]
    All,
    Core(u8),
}

impl From<&str> for CpuType {
    fn from(value: &str) -> Self {
        value
            .strip_prefix("cpu")
            .and_then(|v| v.parse().ok().map(Self::Core))
            .unwrap_or(Self::All)
    }
}

#[derive(Default, Debug)]
struct CpuStats<T> {
    all: T,
    user: T,
    system: T,
    guest: T,
    total: T,
}

impl TryFrom<&str> for CpuStats<usize> {
    type Error = ReadError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let values: Result<Vec<usize>, num::ParseIntError> =
            value.split_whitespace().map(|p| p.parse()).collect();
        // Documentation can be found at
        // https://docs.kernel.org/filesystems/proc.html#miscellaneous-kernel-statistics-in-proc-stat
        let [user, nice, system, idle, iowait, irq, softirq, steal, guest, guest_nice] =
            values?[..]
        else {
            return Err(ReadError::ValueListInvalid);
        };
        let all = user + nice + system + irq + softirq;
        Ok(CpuStats {
            all,
            user: user + nice,
            system,
            guest: guest + guest_nice,
            total: all + idle + iowait + steal,
        })
    }
}

impl From<(&CpuStats<usize>, &CpuStats<usize>)> for CpuStats<u8> {
    fn from((stats1, stats2): (&CpuStats<usize>, &CpuStats<usize>)) -> Self {
        let delta_all = stats2.all - stats1.all;
        let delta_user = stats2.user - stats1.user;
        let delta_system = stats2.system - stats1.system;
        let delta_guest = stats2.guest - stats1.guest;
        let delta_total = stats2.total - stats1.total;
        if delta_total == 0 {
            return Self::default();
        }
        Self {
            all: ((delta_all as f32 / delta_total as f32) * 100.) as u8,
            user: ((delta_user as f32 / delta_total as f32) * 100.) as u8,
            system: ((delta_system as f32 / delta_total as f32) * 100.) as u8,
            guest: ((delta_guest as f32 / delta_total as f32) * 100.) as u8,
            total: 0,
        }
    }
}

fn read_raw_stats() -> Result<HashMap<CpuType, CpuStats<usize>>, ReadError> {
    // Documentation can be found at
    // https://docs.kernel.org/filesystems/proc.html#miscellaneous-kernel-statistics-in-proc-stat
    let file = File::open("/proc/stat")?;
    let reader = BufReader::new(file);
    let lines = reader.lines().filter_map(|l| {
        l.ok().and_then(|line| {
            let (cpu, data) = line.split_once(' ')?;
            Some((cpu.into(), data.try_into().ok()?))
        })
    });
    Ok(lines.collect())
}

#[allow(dead_code)]
#[derive(Debug)]
enum ReadError {
    IoError(io::Error),
    ParseError(num::ParseIntError),
    ValueListInvalid,
}

impl From<io::Error> for ReadError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<num::ParseIntError> for ReadError {
    fn from(value: num::ParseIntError) -> Self {
        Self::ParseError(value)
    }
}
