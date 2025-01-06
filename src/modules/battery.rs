use std::{collections::HashMap, time::Duration};

use bar_rs_derive::Builder;
use iced::{
    futures::SinkExt,
    stream,
    widget::{row, text},
    Length::Fill,
    Subscription,
};
use tokio::{fs, io, runtime, select, sync::mpsc, task, time::sleep};
use udev::Device;

use crate::{config::module_config::LocalModuleConfig, Message, NERD_FONT};

use super::Module;

#[derive(Debug, Default, Builder)]
pub struct BatteryMod {
    capacity: u16,
    hours: u16,
    minutes: u16,
    icon: &'static str,
}

impl Module for BatteryMod {
    fn id(&self) -> String {
        "battery".to_string()
    }

    fn view(&self, config: &LocalModuleConfig) -> iced::Element<Message> {
        row![
            text!("{}", self.icon)
                .center()
                .height(Fill)
                .size(config.icon_size)
                .font(NERD_FONT),
            text![
                " {}% ({}h {}min left)",
                self.capacity,
                self.hours,
                self.minutes
            ]
            .center()
            .height(Fill)
            .size(config.font_size)
        ]
        .into()
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
                            sx.send(()).await.expect("mpsc channel closed");
                        }
                    })
                    .await
                    .unwrap();
                }));
            });

            stream::channel(1, |mut sender| async move {
                tokio::spawn(async move {
                    loop {
                        let stats = get_stats().await.unwrap();
                        sender
                            .send(Message::update(Box::new(move |reg| {
                                *reg.get_module_mut::<BatteryMod>() = stats
                            })))
                            .await
                            .unwrap_or_else(|err| {
                                eprintln!("Trying to send battery_stats failed with err: {err}");
                            });
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

async fn get_stats() -> Result<BatteryMod, io::Error> {
    let mut entries = fs::read_dir("/sys/class/power_supply").await?;
    let mut batteries = vec![];
    while let Ok(Some(dev_name)) = entries.next_entry().await {
        if let Ok(dev_type) =
            fs::read_to_string(&format!("{}/type", dev_name.path().to_string_lossy())).await
        {
            if dev_type.trim() == "Battery" {
                batteries.push(dev_name.path());
            }
        }
    }
    let properties = vec![
        "energy_now",
        "energy_full",
        "power_now",
        "voltage_now",
        "status",
    ];
    let batteries = loop {
        let bats = batteries.iter().fold(vec![], |mut acc, bat| {
            let Ok(bat) = Device::from_syspath(bat) else {
                eprintln!(
                    "Battery {} could not be turned into a udev Device",
                    bat.to_string_lossy()
                );
                return acc;
            };

            let mut map = HashMap::new();
            for prop in &properties {
                map.insert(
                    prop,
                    bat.property_value(format!("POWER_SUPPLY_{}", prop.to_uppercase()))
                        .and_then(|v| v.to_str())
                        .map(|v| {
                            // Charging status is the only text value, so we map it to bool (0 or 1)
                            match *prop == "status" {
                                true => match v {
                                    "Charging" => "1",
                                    _ => "0",
                                },
                                false => v,
                            }
                        })
                        .and_then(|v| v.parse::<f32>().ok())
                        .unwrap_or(0.),
                );
            }

            acc.push(map);
            acc
        });
        if bats.iter().any(|bat| *bat.get(&"power_now").unwrap() != 0.) {
            sleep(Duration::from_secs(1)).await;
            break bats;
        }
    };

    let energy_now = batteries.iter().fold(0., |mut acc, bat| {
        acc += bat.get(&"energy_now").unwrap_or(&0.);
        acc
    });
    let energy_full = batteries.iter().fold(0., |mut acc, bat| {
        acc += bat.get(&"energy_full").unwrap_or(&0.);
        acc
    });
    let (power_now, voltage_now) = batteries
        .iter()
        .filter(|bat| *bat.get(&"power_now").unwrap_or(&0.) != 0.)
        .fold((0., 0.), |mut acc, bat| {
            acc.0 += bat.get(&"power_now").unwrap();
            acc.1 += bat.get(&"voltage_now").unwrap_or(&0.);
            acc
        });

    let capacity = (100. / energy_full * energy_now).round() as u16;
    let charging = batteries
        .iter()
        .any(|bat| *bat.get(&"status").unwrap() == 1.);
    let time_remaining = match charging {
        true => {
            (energy_full - energy_now)
                / 1000000.
                / ((power_now / 1000000.) * (voltage_now / 1000000.))
                * 12.55
        }
        false => energy_now / power_now,
    };

    Ok(BatteryMod {
        capacity,
        hours: time_remaining.floor() as u16,
        minutes: ((time_remaining - time_remaining.floor()) * 60.) as u16,
        icon: match charging {
            false => match capacity {
                n if n >= 80 => "󱊣",
                n if n >= 60 => "󱊢",
                n if n >= 25 => "󱊡",
                _ => "󰂎",
            },
            true => match capacity {
                n if n >= 80 => "󱊦 ",
                n if n >= 60 => "󱊥 ",
                n if n >= 25 => "󱊤 ",
                _ => "󰢟",
            },
        },
    })
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
