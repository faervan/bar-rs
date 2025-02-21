use std::{
    collections::{BTreeMap, HashMap},
    ffi::CString,
    mem,
};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::{
    widget::{button::Style, container, scrollable, text, Container, Text},
    Element,
};
use libc::{__errno_location, statvfs};

use crate::{
    button::button,
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
        popup_config::{PopupConfig, PopupConfigOverride},
    },
    fill::FillExt,
    helpers::UnEscapeString,
    impl_on_click, impl_wrapper, Message, NERD_FONT,
};

use super::Module;

#[derive(Debug, Builder, Default)]
pub struct DiskUsageMod {
    icon: Option<String>,
    cfg_override: ModuleConfigOverride,
    popup_cfg_override: PopupConfigOverride,
    path: CString,
}

#[derive(Debug, Default)]
/// All values are represented in megabytes, except the `_perc` fields
struct FileSystemStats {
    total: u64,
    free: u64,
    used: u64,
    /// Free space in percentage points
    free_perc: u8,
    /// Used space in percentage points
    used_perc: u8,
}

impl From<FileSystemStats> for BTreeMap<&'static str, u64> {
    fn from(value: FileSystemStats) -> Self {
        BTreeMap::from([
            ("total", value.total),
            ("total_gb", value.total / 1000),
            ("used", value.used),
            ("used_gb", value.used / 1000),
            ("free", value.free),
            ("free_gb", value.free / 1000),
            ("used_perc", value.used_perc.into()),
            ("free_perc", value.free_perc.into()),
        ])
    }
}

impl From<statvfs> for FileSystemStats {
    fn from(value: statvfs) -> Self {
        let free_perc = (value.f_bavail as f32 / value.f_blocks as f32 * 100.) as u8;
        Self {
            total: value.f_blocks * value.f_frsize / 1_000_000,
            free: value.f_bavail * value.f_frsize / 1_000_000,
            used: (value.f_blocks - value.f_bavail) * value.f_frsize / 1_000_000,
            free_perc,
            used_perc: 100 - free_perc,
        }
    }
}

impl Module for DiskUsageMod {
    fn name(&self) -> String {
        "disk_usage".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        popup_config: &PopupConfig,
        anchor: &BarAnchor,
        handlebars: &Handlebars,
    ) -> Element<Message> {
        let Ok(stats) = get_stats(&self.path) else {
            return "Error".into();
        };
        let ctx: BTreeMap<&'static str, u64> = stats.into();
        let format = handlebars
            .render("disk_usage", &ctx)
            .map_err(|e| eprintln!("Failed to render disk_usage stats: {e}"))
            .unwrap_or_default();
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
                    text(format)
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
        let Ok(stats) = get_stats(&self.path) else {
            return "Error".into();
        };
        let ctx: BTreeMap<&'static str, u64> = stats.into();
        let format = template
            .render("disk_usage_popup", &ctx)
            .map_err(|e| eprintln!("Failed to render disk_usage stats: {e}"))
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
        self.path = config
            .get("path")
            .and_then(|v| v.clone().and_then(|v| CString::new(v).ok()))
            .unwrap_or_else(|| CString::new("/").unwrap());
        templates
            .register_template_string(
                "disk_usage",
                config
                    .get("format")
                    .unescape()
                    .unwrap_or("{{used_perc}}%".to_string()),
            )
            .unwrap_or_else(|e| eprintln!("Failed to parse battery popup format: {e}"));
        templates
            .register_template_string(
                "disk_usage_popup",
                popup_config
                    .get("format")
                    .unescape()
                    .unwrap_or("Total: {{total_gb}}GB\nUsed: {{used_gb}}GB ({{used_perc}}%)\nFree: {{free_gb}}GB ({{free_perc}}%)".to_string()),
            )
            .unwrap_or_else(|e| eprintln!("Failed to parse battery popup format: {e}"));
    }

    impl_on_click!();
}

fn get_stats(path: &CString) -> Result<FileSystemStats, ()> {
    let mut raw_stats: statvfs = unsafe { mem::zeroed() };
    if unsafe { libc::statvfs(path.as_ptr(), &mut raw_stats) } != 0 {
        eprintln!(
            "Got an error while executing the statvfs syscall: {}",
            unsafe { *__errno_location() }
        );
        return Err(());
    }
    Ok(raw_stats.into())
}
