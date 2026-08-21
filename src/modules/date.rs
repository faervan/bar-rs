use std::collections::HashMap;

use bar_rs_derive::Builder;
use chrono::{Datelike, Days, Local, Months, NaiveDate, Weekday};
use handlebars::Handlebars;
use iced::widget::button::Style;
use iced::widget::{column, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::config::popup_config::{PopupConfig, PopupConfigOverride};
use crate::{
    Message, NERD_FONT,
    button::button,
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
};
use crate::{impl_on_click, impl_wrapper};

use super::Module;

const CELL_WIDTH: f32 = 32.;
const DIMMED_ALPHA: f32 = 0.4;
const WEEKDAY_LABELS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

/// The icons used to navigate the calendar popup.
#[derive(Debug)]
struct NavIcons {
    prev_year: String,
    prev_month: String,
    next_month: String,
    next_year: String,
}

impl Default for NavIcons {
    fn default() -> Self {
        Self {
            // Nerd Font double-angle (year) and angle (month) arrows. They are
            // present in the bundled font and fit inside their advance width,
            // so neighbouring arrows cannot overlap.
            prev_year: "".to_string(),
            prev_month: "".to_string(),
            next_month: "".to_string(),
            next_year: "".to_string(),
        }
    }
}

#[derive(Debug, Builder)]
pub struct DateMod {
    cfg_override: ModuleConfigOverride,
    popup_cfg_override: PopupConfigOverride,
    icon: String,
    fmt: String,
    /// Any day inside the month currently shown in the calendar popup.
    shown_month: NaiveDate,
    first_weekday: Weekday,
    nav_icons: NavIcons,
}

impl Default for DateMod {
    fn default() -> Self {
        let today = Local::now().date_naive();
        Self {
            cfg_override: Default::default(),
            popup_cfg_override: PopupConfigOverride::default(),
            icon: "".to_string(),
            fmt: "%a, %d. %b".to_string(),
            shown_month: today.with_day(1).unwrap_or(today),
            first_weekday: Weekday::Mon,
            nav_icons: NavIcons::default(),
        }
    }
}

impl DateMod {
    /// The first cell of the grid: the first `first_weekday` on or before the
    /// 1st of the shown month, together with how many days lead into the month.
    fn grid(&self) -> (NaiveDate, u32) {
        let leading = self.shown_month.weekday().days_since(self.first_weekday);
        (self.shown_month - Days::new(leading as u64), leading)
    }

    fn shift_month(&mut self, months: i64) {
        let shifted = match months.is_negative() {
            true => self
                .shown_month
                .checked_sub_months(Months::new(months.unsigned_abs() as u32)),
            false => self
                .shown_month
                .checked_add_months(Months::new(months as u32)),
        };
        if let Some(shifted) = shifted {
            self.shown_month = shifted;
        }
    }

    /// Number of days in the shown month.
    fn days_in_month(&self) -> u32 {
        // At the very end of `NaiveDate`'s range the next month does not exist;
        // fall back to the shown month, whose day count is still meaningful.
        let next_month = self
            .shown_month
            .checked_add_months(Months::new(1))
            .unwrap_or(self.shown_month);
        next_month.pred_opt().map_or(1, |last| last.day())
    }

    fn day_cell<'a>(
        &self,
        date: NaiveDate,
        today: NaiveDate,
        font_size: f32,
        text_color: Color,
        accent_color: Color,
    ) -> Element<'a, Message> {
        let is_today = date == today;
        let in_shown_month = date.month() == self.shown_month.month();
        // Today in the month being shown gets the full accent treatment. When
        // another month is shown it is only outlined, so it cannot be mistaken
        // for the current month's today. Neighbouring days are dimmed, except
        // today which keeps full contrast for its outline to read clearly.
        let is_today_here = is_today && in_shown_month;
        let color = match (is_today, in_shown_month) {
            (true, true) => contrast_text(accent_color),
            (true, false) => text_color,
            (false, true) => text_color,
            (false, false) => text_color.scale_alpha(DIMMED_ALPHA),
        };
        let cell = container(text!("{:2}", date.day()).size(font_size).color(color))
            .center_x(Length::Fixed(CELL_WIDTH));
        if is_today_here {
            return cell
                .style(move |_| container::Style {
                    background: Some(Background::Color(accent_color)),
                    ..Default::default()
                })
                .into();
        }
        if is_today {
            return cell
                .style(move |_| container::Style {
                    border: Border {
                        color: accent_color,
                        width: 1.,
                        radius: 2.into(),
                    },
                    ..Default::default()
                })
                .into();
        }
        cell.into()
    }
}

impl Module for DateMod {
    fn name(&self) -> String {
        "date".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        popup_config: &PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<'_, Message> {
        let time = Local::now();
        let content = list![
            anchor,
            container(
                text!("{}", self.icon)
                    .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                    .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                    .font(NERD_FONT)
                    .fill(anchor)
            )
            .fill(anchor)
            .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin)),
            container(
                text!("{}", time.format(&self.fmt))
                    .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                    .color(self.cfg_override.text_color.unwrap_or(config.text_color))
                    .fill(anchor)
            )
            .fill(anchor)
            .padding(self.cfg_override.text_margin.unwrap_or(config.text_margin)),
        ]
        .spacing(self.cfg_override.spacing.unwrap_or(config.spacing));

        button(content)
            .on_press_with_context(Message::popup::<Self>(
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
        _template: &Handlebars,
    ) -> Element<'a, Message> {
        let font_size = self
            .popup_cfg_override
            .font_size
            .unwrap_or(config.font_size);
        let text_color = self
            .popup_cfg_override
            .text_color
            .unwrap_or(config.text_color);
        // The accent is also used for today's highlight, so it shares the
        // popup's icon colour.
        let accent_color = self
            .popup_cfg_override
            .icon_color
            .unwrap_or(config.icon_color);
        let icon_size = self
            .popup_cfg_override
            .icon_size
            .unwrap_or(config.icon_size);
        let icon_margin = self
            .popup_cfg_override
            .icon_margin
            .unwrap_or(config.icon_margin);
        let spacing = self.popup_cfg_override.spacing.unwrap_or(config.spacing);

        let nav = |label: &'a str, f: fn(&mut DateMod)| {
            button(
                text(label)
                    .font(NERD_FONT)
                    .size(icon_size)
                    .color(accent_color),
            )
            .padding(icon_margin)
            .on_press(Message::update(move |reg| {
                f(reg.get_module_mut::<DateMod>())
            }))
            .style(|_, _| Style::default())
        };

        let title = text!("{}", self.shown_month.format("%B %Y"))
            .size(font_size)
            .color(text_color);
        let header = row((0..7).map(|offset| {
            let weekday = self.first_weekday.num_days_from_monday() as usize + offset;
            container(
                text(WEEKDAY_LABELS[weekday % 7])
                    .size(font_size)
                    .color(text_color),
            )
            .center_x(Length::Fixed(CELL_WIDTH))
            .into()
        }));

        let today = Local::now().date_naive();
        let (grid_start, leading) = self.grid();
        let weeks_in_grid = (leading + self.days_in_month()).div_ceil(7);
        let weeks = column((0..weeks_in_grid).map(|week| {
            row((0..7)
                .map(|day| {
                    let date = grid_start + Days::new((week * 7 + day) as u64);
                    self.day_cell(date, today, font_size, text_color, accent_color)
                })
                .collect::<Vec<_>>())
            .into()
        }));

        container(
            column![
                row![
                    nav(&self.nav_icons.prev_year, |d: &mut DateMod| {
                        d.shift_month(-12)
                    }),
                    nav(&self.nav_icons.prev_month, |d: &mut DateMod| {
                        d.shift_month(-1)
                    }),
                    container(title).center_x(Length::Fill),
                    nav(&self.nav_icons.next_month, |d: &mut DateMod| {
                        d.shift_month(1)
                    }),
                    nav(&self.nav_icons.next_year, |d: &mut DateMod| {
                        d.shift_month(12)
                    }),
                ]
                .align_y(Alignment::Center)
                .spacing(4),
                header,
                weeks,
            ]
            .align_x(Alignment::Center)
            .spacing(spacing),
        )
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
        _templates: &mut Handlebars,
    ) {
        let default = Self::default();
        self.cfg_override = config.into();
        self.popup_cfg_override.update(popup_config);
        self.icon = config
            .get("icon")
            .and_then(|v| v.clone())
            .unwrap_or(default.icon);
        self.fmt = config
            .get("format")
            .and_then(|v| v.clone())
            .unwrap_or(default.fmt);
        self.first_weekday = match popup_config.get("first_weekday").and_then(|v| v.as_deref()) {
            Some(day) if day.eq_ignore_ascii_case("sunday") => Weekday::Sun,
            _ => Weekday::Mon,
        };
        self.nav_icons = {
            let default_icons = default.nav_icons;
            NavIcons {
                prev_year: popup_config
                    .get("icon_prev_year")
                    .and_then(|v| v.clone())
                    .unwrap_or(default_icons.prev_year),
                prev_month: popup_config
                    .get("icon_prev_month")
                    .and_then(|v| v.clone())
                    .unwrap_or(default_icons.prev_month),
                next_month: popup_config
                    .get("icon_next_month")
                    .and_then(|v| v.clone())
                    .unwrap_or(default_icons.next_month),
                next_year: popup_config
                    .get("icon_next_year")
                    .and_then(|v| v.clone())
                    .unwrap_or(default_icons.next_year),
            }
        };
    }

    impl_on_click!();
}

/// Pick a readable text color for the given background.
fn contrast_text(bg: Color) -> Color {
    // WCAG-relative luminance at which black overtakes white as the more
    // readable choice (both give a contrast ratio of ~4.58).
    const LUMINANCE_THRESHOLD: f32 = 0.179;
    if bg.relative_luminance() > LUMINANCE_THRESHOLD {
        Color::BLACK
    } else {
        Color::WHITE
    }
}
