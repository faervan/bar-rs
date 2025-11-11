use bar_rs_derive::Builder;

use iced::Element;

use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    impl_wrapper, FillExt, Message,
};

use super::Module;

#[derive(Debug, Default, Builder)]
pub struct EmptyModule {
    cfg_override: ModuleConfigOverride,
}

impl Module for EmptyModule {
    fn name(&self) -> String {
        String::from("empty")
    }
    fn view(
        &self,
        _config: &LocalModuleConfig,
        _popup_config: &crate::config::popup_config::PopupConfig,
        _anchor: &BarAnchor,
        _template: &handlebars::Handlebars,
    ) -> Element<'_, Message> {
        "".into()
    }
    impl_wrapper!();
}
