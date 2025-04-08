use std::{
    any::TypeId,
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    num,
    time::Duration,
};

use bar_rs_derive::Builder;
use iced::{futures::SinkExt, stream, Subscription};
use tokio::time::sleep;

use crate::{helpers::ParseTemplate, Message};

use super::Module;

#[derive(Debug, Builder)]
pub struct CpuMod {
    avg_usage: CpuStats<u8>,
    cores: HashMap<CpuType, CpuStats<u8>>,
    icon: String,
    format: String,
    popup_format: String,
    core_format: String,
}

impl Default for CpuMod {
    fn default() -> Self {
        Self {
            avg_usage: Default::default(),
            cores: HashMap::new(),
            /*popup_cfg_override: PopupConfigOverride {
                width: Some(150),
                height: Some(350),
                ..Default::default()
            },*/
            icon: "󰻠".to_string(),
            format: "row(icon({{icon}}), {{total}}%)".to_string(),
            popup_format: "column(Total: {{total}}%, User: {{user}}%, System: {{system}}%, Guest: {{guest}}%, text({{cores}}))".to_string(),
            core_format: "Core {{index}}: {{total}}%".to_string(),
        }
    }
}

impl Module for CpuMod {
    fn name(&self) -> String {
        "cpu".to_string()
    }

    fn context<'a>(&'a self) -> HashMap<String, Box<dyn ToString + Send + Sync>> {
        create_map!(
            ("total", self.avg_usage.all),
            ("user", self.avg_usage.user),
            ("system", self.avg_usage.system),
            ("guest", self.avg_usage.guest),
            (
                "cores",
                self.cores
                    .iter()
                    .map(|(ty, stats)| {
                        create_map!(
                            ("index", ty.get_core_index()),
                            ("total", stats.all),
                            ("user", stats.user),
                            ("system", stats.system),
                            ("guest", stats.guest)
                        )
                        .parse_template(&self.core_format)
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            ("icon", self.icon.clone())
        )
    }

    fn module_format(&self) -> &str {
        &self.format
    }

    fn popup_format(&self) -> &str {
        &self.popup_format
    }

    fn subscription(&self) -> Option<Subscription<Message>> {
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
                            m.cores = cores;
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
