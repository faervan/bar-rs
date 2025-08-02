use std::collections::HashMap;

use iced::Element;
use smithay_client_toolkit::shell::wlr_layer::Anchor;
use toml::Table;

use crate::config::style::ContainerStyle;

pub trait Module {
    fn name(&self) -> String;

    fn active(&self) -> bool {
        true
    }

    fn view<Message>(
        &self,
        anchor: &Anchor,
        context: &HashMap<String, Box<dyn ToString + Send + Sync>>,
    ) -> Element<Message>;

    fn sources(&self) -> Vec<String> {
        vec![]
    }

    #[allow(unused_variables)]
    fn read_config(&mut self, style: ContainerStyle, config: HashMap<String, Table>) {}
}
