use std::{collections::HashMap, time::Duration};

use bar_rs_derive::Builder;
use iced::{
    Element, Subscription,
    futures::{SinkExt, channel::mpsc::Sender},
    stream,
    widget::text,
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
    listeners::net::NetListener,
};

use super::{Module, require_listener};

#[derive(Debug, Default)]
pub struct NetState {
    pub speed: f32,
    pub total: u64,
    pub icon: String,
}

#[derive(Debug)]
pub struct NetPublicIpState {
    pub ip: String,
    pub icon: String,
    pub url: String,
    pub interval: u64,
}

impl Default for NetPublicIpState {
    fn default() -> Self {
        Self {
            ip: "---".to_string(),
            icon: "📍︎".to_string(),
            url: "https://ip.me/".to_string(),
            interval: 300,
        }
    }
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

fn public_ip_view<'a>(
    state: &'a NetPublicIpState,
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
            text!("{}", &state.ip)
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

fn read_public_ip_config(
    state: &mut NetPublicIpState,
    cfg_override: &mut ModuleConfigOverride,
    config: &HashMap<String, Option<String>>,
) {
    *cfg_override = config.into();
    state.icon = config
        .get("icon")
        .and_then(|v| v.clone())
        .unwrap_or_else(|| state.icon.clone());
    if let Some(Some(url_str)) = config.get("url") {
        state.url = url_str.clone();
    }
    if let Some(Some(interval_str)) = config.get("interval")
        && let Ok(parsed) = interval_str.parse::<u64>()
    {
        state.interval = parsed;
    }
}

async fn get_public_ip_config(sender: &mut Sender<Message>) -> (u64, String) {
    let (sx, rx) = iced::futures::channel::oneshot::channel();
    sender
        .send(Message::action(move |reg| {
            let public_ip = reg.get_module::<NetPublicIpMod>();
            let url = public_ip.state.url.clone();
            let interval = public_ip.state.interval;
            let _ = sx.send((interval, url));
        }))
        .await
        .unwrap_or_else(|err| {
            eprintln!("Trying to get net public_ip config failed with err: {err}")
        });
    rx.await.unwrap_or((300, "https://ip.me/".to_string()))
}

async fn fetch_public_ip(url: &str) -> Option<String> {
    let client = reqwest::Client::new();
    let body = client
        .get(url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .text()
        .await
        .ok()?
        .trim()
        .to_string();
    body.parse::<std::net::IpAddr>().is_ok().then_some(body)
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

#[derive(Debug, Builder)]
pub struct NetUploadMod {
    pub state: NetState,
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

    fn requires(&self) -> Vec<std::any::TypeId> {
        vec![require_listener::<NetListener>()]
    }

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _templates: &mut handlebars::Handlebars,
    ) {
        read_config(&mut self.state, &mut self.cfg_override, config);
    }

    impl_on_click!();
}

#[derive(Debug, Builder)]
pub struct NetDownloadMod {
    pub state: NetState,
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

    fn requires(&self) -> Vec<std::any::TypeId> {
        vec![require_listener::<NetListener>()]
    }

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _templates: &mut handlebars::Handlebars,
    ) {
        read_config(&mut self.state, &mut self.cfg_override, config);
    }

    impl_on_click!();
}

#[derive(Debug, Default, Builder)]
pub struct NetPublicIpMod {
    pub state: NetPublicIpState,
    cfg_override: ModuleConfigOverride,
}

impl Module for NetPublicIpMod {
    fn name(&self) -> String {
        "net.public_ip".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        _popup_config: &crate::config::popup_config::PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &handlebars::Handlebars,
    ) -> Element<'_, Message> {
        public_ip_view(&self.state, &self.cfg_override, config, anchor)
    }

    impl_wrapper!();

    fn subscription(&self) -> Option<Subscription<Message>> {
        Some(Subscription::run(|| {
            stream::channel(1, |mut sender: Sender<Message>| async move {
                let (public_ip_interval, public_ip_url) = get_public_ip_config(&mut sender).await;

                loop {
                    if let Some(ip) = fetch_public_ip(&public_ip_url).await {
                        sender
                            .send(Message::update(move |reg| {
                                let public_ip_mod = reg.get_module_mut::<NetPublicIpMod>();
                                public_ip_mod.state.ip = ip;
                            }))
                            .await
                            .unwrap_or_else(|err| {
                                eprintln!("Trying to send public ip failed with err: {err}")
                            });
                    }
                    sleep(Duration::from_secs(public_ip_interval)).await;
                }
            })
        }))
    }

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _templates: &mut handlebars::Handlebars,
    ) {
        read_public_ip_config(&mut self.state, &mut self.cfg_override, config);
    }

    impl_on_click!();
}
