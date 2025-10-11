use std::{collections::HashMap, fmt::Debug};

use downcast_rs::{Downcast, impl_downcast};
use smithay_client_toolkit::shell::wlr_layer::Anchor;
use toml::Table;

pub use custom::CustomModules;

use crate::{
    Element,
    config::{
        style::{ContainerStyle, ContainerStyleOverride},
        theme::Theme,
    },
    registry::Registry,
    template_engine::TemplateEngine,
};

pub type Context = HashMap<String, Box<dyn ToString + Send + Sync>>;

pub trait Module: Downcast + Debug + Send + Sync {
    fn variant_names(&self) -> Vec<&str>;

    fn active(&self) -> bool {
        true
    }

    fn view(
        &self,
        variant: &str,
        anchor: &Anchor,
        context: &Context,
        theme: &Theme,
        style: &ContainerStyle,
    ) -> Element<'_>;

    #[allow(unused_variables)]
    fn default_style(&self, variant: &str) -> ContainerStyleOverride {
        ContainerStyleOverride::default()
    }

    #[allow(unused_variables)]
    fn sources(&self, variant: &str) -> Vec<&String> {
        vec![]
    }

    #[allow(unused_variables)]
    fn read_config(&mut self, variant: &str, config: Table, engine: &TemplateEngine) {}
}
impl_downcast!(Module);

pub fn register_modules(registry: &mut Registry) {
    registry.register_module::<CustomModules>();
    registry.register_module::<dummy::DummyModule>();
}

mod dummy {
    use derive::Builder;
    use iced::widget::text;

    use crate::{
        Element,
        config::{style::ContainerStyle, theme::Theme},
        module::Module,
    };

    #[derive(Builder, Debug)]
    pub struct DummyModule;

    impl Module for DummyModule {
        fn variant_names(&self) -> Vec<&str> {
            vec!["dummy"]
        }
        fn view(
            &self,
            _variant: &str,
            _anchor: &smithay_client_toolkit::shell::wlr_layer::Anchor,
            _context: &super::Context,
            _theme: &Theme,
            _style: &ContainerStyle,
        ) -> Element<'_> {
            text!("This is a dummy module!").into()
        }
    }
}

mod custom {
    use std::{collections::HashMap, fmt::Debug};

    use derive::Builder;
    use toml::Table;

    use crate::{
        Element,
        config::{style::ContainerStyle, theme::Theme},
        message::Message,
        template_engine::Token,
    };

    use super::Module;

    #[derive(Builder, Default, Debug)]
    pub struct CustomModules {
        modules: HashMap<String, CustomModule>,
    }

    struct CustomModule {
        _sources: Vec<String>,
        _config: Table,
        token: Box<dyn Token<Message> + Send + Sync>,
    }

    impl Debug for CustomModule {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "CustomModule")
        }
    }

    impl Module for CustomModules {
        fn variant_names(&self) -> Vec<&str> {
            self.modules.keys().map(String::as_str).collect()
        }
        fn view(
            &self,
            variant: &str,
            anchor: &smithay_client_toolkit::shell::wlr_layer::Anchor,
            context: &super::Context,
            theme: &Theme,
            style: &ContainerStyle,
        ) -> Element<'_> {
            let Some(custom) = self.modules.get(variant) else {
                log::error!("Invalid variant name of custom module: {variant}");
                return "Invalid variant name".into();
            };
            custom.token.render(context, anchor, style, theme)
        }
        fn read_config(
            &mut self,
            variant: &str,
            config: Table,
            engine: &crate::template_engine::TemplateEngine,
        ) {
            let format = match config.get("format") {
                Some(toml::Value::String(fmt)) => fmt,
                _ => {
                    log::warn!("No format was specified for the custom module!");
                    "No format specified"
                }
            };
            self.modules.insert(
                String::from(variant),
                CustomModule {
                    _sources: vec![],
                    token: engine.render_token(format),
                    _config: config,
                },
            );
        }
    }
}
