use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    hash::Hash,
    io::{self, BufRead, BufReader},
    num,
    time::Duration,
};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::widget::{button::Style, container, scrollable, Container, Text};
use iced::{futures::SinkExt, stream, widget::text, Element, Subscription};
use tokio::time::sleep;

use crate::{
    button::button,
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
        popup_config::{PopupConfig, PopupConfigOverride},
    },
    fill::FillExt,
    helpers::UnEscapeString,
    impl_wrapper, Message, NERD_FONT,
};

use super::Module;

#[derive(Debug, Builder)]
pub struct CpuMod {
    avg_usage: CpuStats<u8>,
    cores: BTreeMap<CpuType, CpuStats<u8>>,
    cfg_override: ModuleConfigOverride,
    popup_cfg_override: PopupConfigOverride,
    icon: Option<String>,
}

impl Default for CpuMod {
    fn default() -> Self {
        Self {
            avg_usage: Default::default(),
            cores: BTreeMap::new(),
            cfg_override: Default::default(),
            popup_cfg_override: PopupConfigOverride {
                width: Some(150),
                height: Some(350),
                ..Default::default()
            },
            icon: None,
        }
    }
}

impl Module for CpuMod {
    fn name(&self) -> String {
        "cpu".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        popup_config: &PopupConfig,
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
        .on_event_with(Message::popup::<Self>(
            self.popup_cfg_override.width.unwrap_or(popup_config.width),
            self.popup_cfg_override
                .height
                .unwrap_or(popup_config.height),
            anchor,
        ))
        .style(|_, _| Style::default())
        .into()
    }

    fn popup_view<'a>(
        &'a self,
        config: &'a PopupConfig,
        template: &Handlebars,
    ) -> Element<'a, Message> {
        let fmt_text = |text: Text<'a>| -> Container<'a, Message> {
            container(
                text.size(
                    self.popup_cfg_override
                        .font_size
                        .unwrap_or(config.font_size),
                )
                .color(
                    self.popup_cfg_override
                        .text_color
                        .unwrap_or(config.text_color),
                ),
            )
            .padding(
                self.popup_cfg_override
                    .text_margin
                    .unwrap_or(config.text_margin),
            )
        };
        let ctx = BTreeMap::from([
            ("total", self.avg_usage.all.to_string()),
            ("user", self.avg_usage.user.to_string()),
            ("system", self.avg_usage.system.to_string()),
            ("guest", self.avg_usage.guest.to_string()),
            (
                "cores",
                self.cores
                    .iter()
                    .map(|(ty, stats)| {
                        let core = BTreeMap::from([
                            ("index", ty.get_core_index().to_string()),
                            ("total", stats.all.to_string()),
                            ("user", stats.user.to_string()),
                            ("system", stats.system.to_string()),
                            ("guest", stats.guest.to_string()),
                        ]);
                        template
                            .render("cpu_core", &core)
                            .map_err(|e| eprintln!("Failed to render cpu core stats: {e}"))
                            .unwrap_or_default()
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            ),
        ]);
        let format = template
            .render("cpu", &ctx)
            .map_err(|e| eprintln!("Failed to render cpu stats: {e}"))
            .unwrap_or_default();
        container(scrollable(fmt_text(text(format))))
            .padding(self.popup_cfg_override.padding.unwrap_or(config.padding))
            .style(|_| container::Style {
                background: Some(
                    self.popup_cfg_override
                        .background
                        .unwrap_or(config.background),
                ),
                border: self.popup_cfg_override.border.unwrap_or(config.border),
                ..Default::default()
            })
            .fill_maybe(
                self.popup_cfg_override
                    .fill_content_to_size
                    .unwrap_or(config.fill_content_to_size),
            )
            .into()
    }

    impl_wrapper!();

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        popup_config: &HashMap<String, Option<String>>,
        templates: &mut Handlebars,
    ) {
        self.cfg_override = config.into();
        self.popup_cfg_override.update(popup_config);
        self.icon = config.get("icon").and_then(|v| v.clone());
        templates
            .register_template_string(
                "cpu",
                popup_config
                    .get("format")
                    .unescape()
                    .unwrap_or("Total: {{total}}%\nUser: {{user}}%\nSystem: {{system}}%\nGuest: {{guest}}%\n{{cores}}".to_string()),
            )
            .unwrap_or_else(|e| eprintln!("Failed to parse battery popup format: {e}"));
        templates
            .register_template_string(
                "cpu_core",
                popup_config
                    .get("format_core")
                    .unescape()
                    .unwrap_or("Core {{index}}: {{total}}%".to_string()),
            )
            .unwrap_or_else(|e| eprintln!("Failed to parse battery popup format: {e}"));
    }

    fn subscription(&self) -> Option<iced::Subscription<Message>> {
        Some(Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                let interval: u64 = 500;
                let gap: u64 = 2000;
                loop {
                    let Ok(mut raw_stats1) = read_raw_stats()
                        .map_err(|e| eprintln!("Failed to read cpu stats from /proc/stat: {e:?}"))
                    else {
                        return;
                    };
                    sleep(Duration::from_millis(interval)).await;
                    let Ok(mut raw_stats2) = read_raw_stats() else {
                        eprintln!("Failed to read cpu stats from /proc/stat");
                        return;
                    };

                    let avg = (
                        &raw_stats1.remove(&CpuType::All).unwrap(),
                        &raw_stats2.remove(&CpuType::All).unwrap(),
                    )
                        .into();

                    let cores = raw_stats1
                        .into_iter()
                        .filter_map(|(ty, stats1)| {
                            raw_stats2
                                .get(&ty)
                                .map(|stats2| (ty, (&stats1, stats2).into()))
                        })
                        .collect();

                    sender
                        .send(Message::update(move |reg| {
                            let m = reg.get_module_mut::<CpuMod>();
                            m.avg_usage = avg;
                            m.cores = cores
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

#[derive(Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum CpuType {
    #[default]
    All,
    Core(u8),
}

impl CpuType {
    fn get_core_index(&self) -> u8 {
        match self {
            CpuType::All => 255,
            CpuType::Core(index) => *index,
        }
    }
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
