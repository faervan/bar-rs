use std::{any::TypeId, collections::HashMap};

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::widget::{container, rich_text, span};
use iced::Element;
use iced::Padding;

use crate::config::parse::StringExt;
use crate::config::popup_config::PopupConfig;
use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    listeners::wayfire::WayfireListener,
    modules::Module,
    Message, NERD_FONT,
};
use crate::{impl_on_click, impl_wrapper};

/// I am unaware of a IPC method that gives a list of currently active workspaces (the ones with an
/// open window), and this is generally tricky here, since all workspaces of a wset grid are active
/// in a way. It would probably be posible to calculate the workspace of each active window
/// manually, but I'm too lazy to do that atm.

#[derive(Debug, Default, Builder)]
pub struct WayfireWorkspaceMod {
    pub active: (i64, i64),
    icons: HashMap<(i64, i64), String>,
    cfg_override: ModuleConfigOverride,
    icon_padding: Padding,
    fallback_icon: Option<String>,
}

impl Module for WayfireWorkspaceMod {
    fn name(&self) -> String {
        "wayfire.workspaces".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        _popup_config: &PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<Message> {
        container(
            rich_text([span(
                self.icons
                    .get(&self.active)
                    .or(self.fallback_icon.as_ref())
                    .cloned()
                    .unwrap_or(format!("{}/{}", self.active.0, self.active.1)),
            )
            .padding(self.icon_padding)
            .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
            .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
            .font(NERD_FONT)])
            .fill(anchor),
        )
        .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin))
        .into()
    }

    impl_wrapper!();

    fn requires(&self) -> Vec<std::any::TypeId> {
        vec![TypeId::of::<WayfireListener>()]
    }

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _templates: &mut Handlebars,
    ) {
        self.cfg_override = config.into();
        self.icon_padding = config
            .get("icon_padding")
            .and_then(|v| v.into_insets().map(|i| i.into()))
            .unwrap_or(Self::default().icon_padding);
        self.fallback_icon = config.get("fallback_icon").and_then(|v| v.clone());
        config.iter().for_each(|(key, val)| {
            if let Some(key) = key
                .strip_prefix('(')
                .and_then(|v| v.strip_suffix(')'))
                .and_then(|v| {
                    let [x, y] = v.split(',').map(|item| item.trim()).collect::<Vec<&str>>()[..]
                    else {
                        return None;
                    };
                    x.parse().and_then(|x| y.parse().map(|y| (x, y))).ok()
                })
            {
                self.icons.insert(key, val.clone().unwrap_or(String::new()));
            }
        });
    }

    impl_on_click!();
}
