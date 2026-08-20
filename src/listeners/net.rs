use std::{fs, io, io::BufRead, time::Duration};

use bar_rs_derive::Builder;
use iced::{
    Subscription,
    futures::{SinkExt, channel::mpsc::Sender},
    stream,
};
use tokio::time::sleep;

use crate::{
    Message,
    modules::net::{NetDownloadMod, NetUploadMod},
};

use super::Listener;

const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Builder)]
pub struct NetListener;

struct NetStats {
    rx_bytes: u64,
    tx_bytes: u64,
}

fn read_stats(iface: &str) -> Result<NetStats, io::Error> {
    let file = fs::File::open("/proc/net/dev")?;
    let reader = io::BufReader::new(file);
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
    let reader = io::BufReader::new(file);
    for line in reader.lines().map_while(Result::ok).skip(1) {
        let mut fields = line.split_whitespace();
        let iface = fields.next()?;
        let destination = fields.next()?;
        if destination == "00000000" {
            return Some(iface.to_string());
        }
    }
    None
}

impl Listener for NetListener {
    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(|| {
            stream::channel(1, |mut sender: Sender<Message>| async move {
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
                    let now = std::time::Instant::now();
                    let (rx_speed, tx_speed) = match last {
                        Some((prev_rx, prev_tx, prev_time)) => {
                            let elapsed = now.duration_since(prev_time).as_secs_f32().max(0.001);
                            (
                                stats.rx_bytes.saturating_sub(prev_rx) as f32 / elapsed,
                                stats.tx_bytes.saturating_sub(prev_tx) as f32 / elapsed,
                            )
                        }
                        None => (0., 0.),
                    };
                    last = Some((stats.rx_bytes, stats.tx_bytes, now));
                    sender
                        .send(Message::update(move |reg| {
                            let upload = reg.get_module_mut::<NetUploadMod>();
                            upload.state.speed = tx_speed;
                            upload.state.total = stats.tx_bytes;
                            let download = reg.get_module_mut::<NetDownloadMod>();
                            download.state.speed = rx_speed;
                            download.state.total = stats.rx_bytes;
                        }))
                        .await
                        .unwrap_or_else(|err| {
                            eprintln!("Trying to send net speed failed with err: {err}")
                        });
                    sleep(SAMPLE_INTERVAL).await;
                }
            })
        })
    }
}
