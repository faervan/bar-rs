use std::{
    collections::HashMap,
    fs,
    io::{self, BufRead, BufReader},
    time::Duration,
};

use bar_rs_derive::Builder;
use iced::{
    Element, Subscription, futures::SinkExt, futures::channel::mpsc::Sender, stream, widget::text,
};
use tokio::time::sleep;

use crate::{
    Message, NERD_FONT,
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    impl_on_click, impl_wrapper,
};

use super::Module;

const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);

enum Direction {
    Rx,
    Tx,
}

#[derive(Debug, Default)]
struct NetState {
    speed: f32,
    total: u64,
    icon: String,
}

fn net_view<'a>(
    state: &'a NetState,
    cfg_override: &ModuleConfigOverride,
    config: &LocalModuleConfig,
    anchor: &BarAnchor,
) -> Element<'a, Message> {
    list![
        anchor,
        iced::widget::container(
            text!("{}", state.icon)
                .fill(anchor)
                .size(cfg_override.icon_size.unwrap_or(config.icon_size))
                .color(cfg_override.icon_color.unwrap_or(config.icon_color))
                .font(NERD_FONT)
        )
        .padding(cfg_override.icon_margin.unwrap_or(config.icon_margin)),
        iced::widget::container(
            text!("{}", format_speed(state.speed))
                .fill(anchor)
                .size(cfg_override.font_size.unwrap_or(config.font_size))
                .color(cfg_override.text_color.unwrap_or(config.text_color))
        )
        .padding(cfg_override.text_margin.unwrap_or(config.text_margin)),
    ]
    .spacing(cfg_override.spacing.unwrap_or(config.spacing))
    .into()
}

fn read_config(
    state: &mut NetState,
    cfg_override: &mut ModuleConfigOverride,
    config: &HashMap<String, Option<String>>,
) {
    *cfg_override = config.into();
    state.icon = config
        .get("icon")
        .and_then(|v| v.clone())
        .unwrap_or_else(|| state.icon.clone());
}

async fn net_loop<M: Module + NetModule>(sender: &mut Sender<Message>, direction: Direction) {
    let mut last = None;
    loop {
        let Some(iface) = default_interface() else {
            sleep(SAMPLE_INTERVAL).await;
            continue;
        };
        let Ok(stats) = read_stats(&iface) else {
            sleep(SAMPLE_INTERVAL).await;
            continue;
        };
        let total = match direction {
            Direction::Rx => stats.rx_bytes,
            Direction::Tx => stats.tx_bytes,
        };
        let speed = match last {
            Some((prev_total, prev_time)) => {
                let elapsed = std::time::Instant::now().duration_since(prev_time);
                let elapsed = elapsed.as_secs_f32().max(0.001);
                (total.saturating_sub(prev_total)) as f32 / elapsed
            }
            None => 0.,
        };
        last = Some((total, std::time::Instant::now()));
        sender
            .send(Message::update(move |reg| {
                let m = reg.get_module_mut::<M>();
                m.state().speed = speed;
                m.state().total = total;
            }))
            .await
            .unwrap_or_else(|err| eprintln!("Trying to send net speed failed with err: {err}"));
        sleep(SAMPLE_INTERVAL).await;
    }
}

struct NetStats {
    rx_bytes: u64,
    tx_bytes: u64,
}

fn read_stats(iface: &str) -> Result<NetStats, io::Error> {
    let file = fs::File::open("/proc/net/dev")?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let Some((name, data)) = line.split_once(':') else {
            continue;
        };
        if name.trim() != iface {
            continue;
        }
        let fields: Vec<&str> = data.split_whitespace().collect();
        return Ok(NetStats {
            rx_bytes: fields.first().and_then(|v| v.parse().ok()).unwrap_or(0),
            tx_bytes: fields.get(8).and_then(|v| v.parse().ok()).unwrap_or(0),
        });
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("interface {iface} not found in /proc/net/dev"),
    ))
}

/// Find the interface used by the default route
fn default_interface() -> Option<String> {
    let file = fs::File::open("/proc/net/route").ok()?;
    let reader = BufReader::new(file);
    for line in reader.lines().map_while(Result::ok).skip(1) {
        let mut fields = line.split_whitespace();
        let destination = fields.next()?;
        if destination == "00000000" {
            let iface = fields.next()?.to_string();
            return Some(iface);
        }
    }
    None
}

/// Format a byte rate, e.g. `1.5 MB/s`.
fn format_speed(speed: f32) -> String {
    let (value, unit) = if speed >= 1024. * 1024. {
        (speed / (1024. * 1024.), "MB/s")
    } else if speed >= 1024. {
        (speed / 1024., "KB/s")
    } else {
        (speed, "B/s")
    };
    format!("{value:.1} {unit}")
}

trait NetModule {
    fn state(&mut self) -> &mut NetState;
}

#[derive(Debug, Builder)]
pub struct NetUploadMod {
    state: NetState,
    cfg_override: ModuleConfigOverride,
}

impl Default for NetUploadMod {
    fn default() -> Self {
        Self {
            state: NetState {
                icon: "󰕒".to_string(),
                ..Default::default()
            },
            cfg_override: Default::default(),
        }
    }
}

impl NetModule for NetUploadMod {
    fn state(&mut self) -> &mut NetState {
        &mut self.state
    }
}

impl Module for NetUploadMod {
    fn name(&self) -> String {
        "net.upload".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        _popup_config: &crate::config::popup_config::PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &handlebars::Handlebars,
    ) -> Element<'_, Message> {
        net_view(&self.state, &self.cfg_override, config, anchor)
    }

    impl_wrapper!();

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _templates: &mut handlebars::Handlebars,
    ) {
        read_config(&mut self.state, &mut self.cfg_override, config);
    }

    impl_on_click!();

    fn subscription(&self) -> Option<iced::Subscription<Message>> {
        Some(Subscription::run(|| {
            stream::channel(1, |mut sender: Sender<Message>| async move {
                net_loop::<Self>(&mut sender, Direction::Tx).await;
            })
        }))
    }
}

#[derive(Debug, Builder)]
pub struct NetDownloadMod {
    state: NetState,
    cfg_override: ModuleConfigOverride,
}

impl Default for NetDownloadMod {
    fn default() -> Self {
        Self {
            state: NetState {
                icon: "󰇚".to_string(),
                ..Default::default()
            },
            cfg_override: Default::default(),
        }
    }
}

impl NetModule for NetDownloadMod {
    fn state(&mut self) -> &mut NetState {
        &mut self.state
    }
}

impl Module for NetDownloadMod {
    fn name(&self) -> String {
        "net.download".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        _popup_config: &crate::config::popup_config::PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &handlebars::Handlebars,
    ) -> Element<'_, Message> {
        net_view(&self.state, &self.cfg_override, config, anchor)
    }

    impl_wrapper!();

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _templates: &mut handlebars::Handlebars,
    ) {
        read_config(&mut self.state, &mut self.cfg_override, config);
    }

    impl_on_click!();

    fn subscription(&self) -> Option<iced::Subscription<Message>> {
        Some(Subscription::run(|| {
            stream::channel(1, |mut sender: Sender<Message>| async move {
                net_loop::<Self>(&mut sender, Direction::Rx).await;
            })
        }))
    }
}
