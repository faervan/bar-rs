use std::collections::BTreeMap;
use std::fmt::Display;
use std::{collections::HashMap, time::Duration};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::widget::button::Style;
use iced::widget::{column, container, scrollable};
use iced::{futures::SinkExt, stream, widget::text, Element, Subscription};
use tokio::{fs, io, runtime, select, sync::mpsc, task, time::sleep};
use udev::Device;

use crate::button::button;
use crate::config::popup_config::{PopupConfig, PopupConfigOverride};
use crate::helpers::UnEscapeString;
use crate::impl_wrapper;
use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    Message, NERD_FONT,
};

use super::Module;

#[derive(Debug, Builder)]
pub struct BatteryMod {
    avg: AverageStats,
    batteries: Vec<Battery>,
    cfg_override: ModuleConfigOverride,
    popup_cfg_override: PopupConfigOverride,
    icons: BTreeMap<u8, String>,
    icons_charging: BTreeMap<u8, String>,
}

impl Default for BatteryMod {
    fn default() -> Self {
        BatteryMod {
            avg: AverageStats::default(),
            batteries: vec![],
            cfg_override: Default::default(),
            popup_cfg_override: PopupConfigOverride {
                width: Some(250),
                height: Some(250),
                ..Default::default()
            },
            icons: BTreeMap::from([
                (80, "󱊣".to_string()),
                (60, "󱊢".to_string()),
                (25, "󱊡".to_string()),
                (0, "󰂎".to_string()),
            ]),
            icons_charging: BTreeMap::from([
                (80, "󱊦 ".to_string()),
                (60, "󱊥 ".to_string()),
                (25, "󱊤 ".to_string()),
                (0, "󰢟 ".to_string()),
            ]),
        }
    }
}

impl BatteryMod {
    fn icon(&self, capacity: Option<u8>, charging: Option<bool>) -> &String {
        let capacity = capacity.unwrap_or(self.avg.capacity);
        let is_charging = charging.unwrap_or(self.avg.charging);
        let icons = match is_charging {
            true => &self.icons_charging,
            false => &self.icons,
        };
        icons
            .iter()
            .filter(|(k, _)| capacity >= **k)
            .last()
            .unwrap()
            .1
    }
}

#[derive(Debug, Default)]
struct AverageStats {
    capacity: u8,
    charging: bool,
    hours: u16,
    minutes: u16,
    // If you have a laptop with AC not plugged in, yet all batteries report power_now as 0,
    // something's gotta be wrong.
    _cursed: bool,
}

#[derive(Debug, Default)]
enum BatteryState {
    Charging,
    Discharging,
    #[default]
    Idle,
}

impl Display for BatteryState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Charging => "charging",
                Self::Discharging => "discharging",
                Self::Idle => "",
            }
        )
    }
}

#[derive(Debug, Default)]
struct Battery {
    name: String,
    model_name: String,
    energy_now: f32,
    energy_full: f32,
    health: u8,
    state: BatteryState,
    /// (hours, minutes)
    remaining: Option<(u16, u16)>,
}

impl Battery {
    fn capacity(&self) -> u8 {
        (self.energy_now / self.energy_full * 100.) as u8
    }

    fn is_charging(&self) -> bool {
        matches!(self.state, BatteryState::Charging)
    }

    fn icon<'a>(&'a self, module: &'a BatteryMod) -> &'a String {
        module.icon(Some(self.capacity()), Some(self.is_charging()))
    }
}

#[derive(Default, Debug)]
struct BatteryStats {
    name: String,
    model_name: String,
    energy_now: f32,
    energy_full: f32,
    energy_full_design: f32,
    power_now: f32,
    voltage_now: f32,
    charging: bool,
}

impl Module for BatteryMod {
    fn name(&self) -> String {
        "battery".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        popup_config: &PopupConfig,
        anchor: &BarAnchor,
        handlebars: &Handlebars,
    ) -> Element<Message> {
        let mut ctx = BTreeMap::new();
        ctx.insert("capacity", self.avg.capacity as u16);
        ctx.insert("hours", self.avg.hours);
        ctx.insert("minutes", self.avg.minutes);
        button(
            list![
                anchor,
                container(
                    text(self.icon(None, None))
                        .fill(anchor)
                        .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                        .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                        .font(NERD_FONT)
                )
                .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin)),
                container(
                    text(handlebars.render("battery", &ctx).unwrap_or_default())
                        .fill(anchor)
                        .color(self.cfg_override.text_color.unwrap_or(config.text_color))
                        .size(self.cfg_override.font_size.unwrap_or(config.font_size))
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
        container(scrollable(
            column(self.batteries.iter().map(|bat| {
                let state = bat.state.to_string();
                let capacity = bat.capacity().to_string();
                let energy = (bat.energy_now.floor() as u32 / 1000000).to_string();
                let health = bat.health.to_string();

                let remaining = bat
                    .remaining
                    .and_then(|(hours, minutes)| {
                        let time_ctx = BTreeMap::from([("hours", hours), ("minutes", minutes)]);
                        template
                            .render("battery_popup_time_remaining", &time_ctx)
                            .map_err(|e| eprintln!("Failed to render remaining battery time: {e}"))
                            .ok()
                    })
                    .unwrap_or_default();

                let mut ctx = BTreeMap::new();
                ctx.insert("name", &bat.name);
                ctx.insert("state", &state);
                ctx.insert("icon", bat.icon(self));
                ctx.insert("capacity", &capacity);
                ctx.insert("energy", &energy);
                ctx.insert("health", &health);
                ctx.insert("time_remaining", &remaining);
                ctx.insert("model", &bat.model_name);

                container(
                    text(
                        template
                            .render("battery_popup", &ctx)
                            .map_err(|e| eprintln!("Failed to render battery stats: {e}"))
                            .unwrap_or_default(),
                    )
                    .size(
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
                .into()
            }))
            .spacing(self.popup_cfg_override.spacing.unwrap_or(config.spacing)),
        ))
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
        templates
            .register_template_string(
                "battery",
                config
                    .get("format")
                    .unescape()
                    .unwrap_or("{{capacity}}% ({{hours}}h {{minutes}}min left)".to_string()),
            )
            .unwrap_or_else(|e| eprintln!("Failed to parse battery format: {e}"));
        templates
            .register_template_string(
                "battery_popup",
                popup_config
                    .get("format").unescape()
                    .unwrap_or("{{name}}: {{state}}\n\t{{icon}} {{capacity}}% ({{energy}} Wh)\n\thealth: {{health}}%{{time_remaining}}\n\tmodel: {{model}}".to_string()),
            )
            .unwrap_or_else(|e| eprintln!("Failed to parse battery popup format: {e}"));
        templates
            .register_template_string(
                "battery_popup_time_remaining",
                popup_config
                    .get("format_time")
                    .unescape()
                    .unwrap_or("\n\t{{hours}}h {{minutes}}min remaining".to_string()),
            )
            .unwrap_or_else(|e| eprintln!("Failed to parse battery popup time format: {e}"));
    }

    fn subscription(&self) -> Option<iced::Subscription<Message>> {
        Some(Subscription::run(|| {
            let (sx, mut rx) = mpsc::channel(10);
            std::thread::spawn(move || {
                let local = task::LocalSet::new();
                let runtime = runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                runtime.block_on(local.run_until(async move {
                    task::spawn_local(async move {
                        let socket = udev::MonitorBuilder::new()
                            .and_then(|b| b.match_subsystem_devtype("power_supply", "power_supply"))
                            .and_then(|b| b.listen())
                            .expect("Failed to build udev MonitorBuilder");

                        loop {
                            let Some(event) = socket.iter().next() else {
                                sleep(Duration::from_millis(10)).await;
                                continue;
                            };

                            if event.sysname() != "AC" {
                                continue;
                            }
                            sleep(Duration::from_secs(1)).await;
                            if sx.send(()).await.is_err() {
                                return;
                            }
                        }
                    })
                    .await
                    .unwrap();
                }));
            });

            stream::channel(1, |mut sender| async move {
                tokio::spawn(async move {
                    loop {
                        let (avg, batteries) = get_stats(None, false).await.unwrap();
                        if sender
                            .send(Message::update(move |reg| {
                                let m = reg.get_module_mut::<BatteryMod>();
                                m.avg = avg;
                                m.batteries = batteries
                            }))
                            .await
                            .is_err()
                        {
                            return;
                        }
                        select! {
                            _ = sleep(Duration::from_secs(30)) => {}
                            _ = rx.recv() => {}
                        }
                    }
                });
            })
        }))
    }
}

impl From<(&Device, String)> for BatteryStats {
    fn from((device, name): (&Device, String)) -> Self {
        BatteryStats {
            name,
            model_name: get_property(device, "POWER_SUPPLY_MODEL_NAME").to_string(),
            energy_now: get_property(device, "POWER_SUPPLY_ENERGY_NOW")
                .parse()
                .unwrap_or(0.),
            energy_full: get_property(device, "POWER_SUPPLY_ENERGY_FULL")
                .parse()
                .unwrap_or(0.),
            energy_full_design: get_property(device, "POWER_SUPPLY_ENERGY_FULL_DESIGN")
                .parse()
                .unwrap_or(0.),
            power_now: get_property(device, "POWER_SUPPLY_POWER_NOW")
                .parse()
                .unwrap_or(0.),
            voltage_now: get_property(device, "POWER_SUPPLY_VOLTAGE_NOW")
                .parse()
                .unwrap_or(0.),
            charging: matches!(get_property(device, "POWER_SUPPLY_STATUS"), "Charging"),
        }
    }
}

impl From<&Vec<BatteryStats>> for AverageStats {
    fn from(batteries: &Vec<BatteryStats>) -> Self {
        let energy_now = batteries.iter().fold(0., |mut acc, bat| {
            acc += bat.energy_now;
            acc
        });
        let energy_full = batteries.iter().fold(0., |mut acc, bat| {
            acc += bat.energy_full;
            acc
        });
        let (power_now, voltage_now) =
            batteries
                .iter()
                .filter(|bat| bat.power_now != 0.)
                .fold((0., 0.), |mut acc, bat| {
                    acc.0 += bat.power_now;
                    acc.1 += bat.voltage_now;
                    acc
                });

        let capacity = (100. / energy_full * energy_now).round() as u8;
        let charging = batteries.iter().any(|bat| bat.charging);
        let time_remaining = match charging {
            true => {
                (energy_full - energy_now)
                    / 1000000.
                    / ((power_now / 1000000.) * (voltage_now / 1000000.))
                    * 12.55
            }
            false => energy_now / power_now,
        };
        AverageStats {
            capacity,
            charging,
            hours: time_remaining.floor() as u16,
            minutes: ((time_remaining - time_remaining.floor()) * 60.) as u16,
            _cursed: power_now == 0.,
        }
    }
}

impl From<BatteryStats> for Battery {
    fn from(stats: BatteryStats) -> Self {
        let remaining = (stats.power_now != 0.).then(|| {
            let t = match stats.charging {
                true => {
                    (stats.energy_full - stats.energy_now)
                        / 1000000.
                        / ((stats.power_now / 1000000.) * (stats.voltage_now / 1000000.))
                        * 12.55
                }
                false => stats.energy_now / stats.power_now,
            };
            (t.floor() as u16, ((t - t.floor()) * 60.) as u16)
        });
        Battery {
            name: stats.name,
            model_name: stats.model_name,
            energy_now: stats.energy_now,
            energy_full: stats.energy_full,
            health: (stats.energy_full / stats.energy_full_design * 100.) as u8,
            state: match stats.charging {
                true => BatteryState::Charging,
                false => match stats.power_now == 0. {
                    true => BatteryState::Idle,
                    false => BatteryState::Discharging,
                },
            },
            remaining,
        }
    }
}

fn get_property<'a>(device: &'a Device, property: &'static str) -> &'a str {
    device
        .property_value(property)
        .and_then(|v| v.to_str())
        .unwrap_or("")
}

async fn get_stats(
    selection: Option<&Vec<String>>,
    is_blacklist: bool,
) -> Result<(AverageStats, Vec<Battery>), io::Error> {
    let mut entries = fs::read_dir("/sys/class/power_supply").await?;
    let mut batteries = vec![];
    while let Ok(Some(dev_name)) = entries.next_entry().await {
        if let Some(selection) = selection {
            if is_blacklist
                == selection.contains(&dev_name.file_name().to_string_lossy().to_string())
            {
                continue;
            }
        }
        if let Ok(dev_type) =
            fs::read_to_string(&format!("{}/type", dev_name.path().to_string_lossy())).await
        {
            if dev_type.trim() == "Battery" {
                batteries.push(dev_name.path());
            }
        }
    }
    let batteries = batteries.iter().fold(vec![], |mut acc, bat| {
        let Ok(device) = Device::from_syspath(bat) else {
            eprintln!(
                "Battery {} could not be turned into a udev Device",
                bat.to_string_lossy()
            );
            return acc;
        };

        acc.push(BatteryStats::from((
            &device,
            bat.file_name().unwrap().to_string_lossy().to_string(),
        )));
        acc
    });
    Ok((
        (&batteries).into(),
        batteries.into_iter().map(|b| b.into()).collect(),
    ))
}

/*
    How upower calculates remaining time (upower/src/up-daemon.c):
    /* calculate a quick and dirty time remaining value
     * NOTE: Keep in sync with per-battery estimation code! */
    if (energy_rate_total > 0) {
        if (state_total == UP_DEVICE_STATE_DISCHARGING)
            time_to_empty_total = SECONDS_PER_HOUR * (energy_total / energy_rate_total);
        else if (state_total == UP_DEVICE_STATE_CHARGING)
            time_to_full_total = SECONDS_PER_HOUR * ((energy_full_total - energy_total) / energy_rate_total);
    }
*/
