use std::{collections::HashMap, fmt::Debug};

use custom::CustomModules;
use downcast_rs::{impl_downcast, Downcast};
use smithay_client_toolkit::shell::wlr_layer::Anchor;
use toml::Table;

use crate::{
    config::style::ContainerStyle, registry::Registry, template_engine::TemplateEngine, Element,
};

pub type Context = HashMap<String, Box<dyn ToString + Send + Sync>>;

pub trait Module: Downcast + Debug + Send + Sync {
    fn variant_names(&self) -> Vec<&str>;

    fn active(&self) -> bool {
        true
    }

    fn view(&self, variant: &str, anchor: &Anchor, context: &Context) -> Element;

    #[allow(unused_variables)]
    fn sources(&self, variant: &str) -> Vec<&String> {
        vec![]
    }

    #[allow(unused_variables)]
    fn read_config(
        &mut self,
        variant: &str,
        config: Table,
        style: ContainerStyle,
        engine: &TemplateEngine,
    ) {
    }
}
impl_downcast!(Module);

pub fn register_modules(registry: &mut Registry) {
    registry.register_module::<CustomModules>();
    registry.register_module::<dummy::DummyModule>();
}

mod dummy {
    use derive::Builder;
    use iced::widget::text;

    use crate::{module::Module, Element};

    #[derive(Builder, Debug)]
    pub struct DummyModule;

    impl Module for DummyModule {
        fn variant_names(&self) -> Vec<&str> {
            vec!["dummy"]
        }
        fn view(
            &self,
            variant: &str,
            anchor: &smithay_client_toolkit::shell::wlr_layer::Anchor,
            context: &super::Context,
        ) -> Element {
            text!("This is a dummy module!").into()
        }
    }
}

mod custom {
    use std::{collections::HashMap, fmt::Debug};

    use derive::Builder;
    use toml::Table;

    use crate::{config::style::ContainerStyle, message::Message, template_engine::Token, Element};

    use super::Module;

    #[derive(Builder, Default, Debug)]
    pub struct CustomModules {
        modules: HashMap<String, CustomModule>,
    }

    struct CustomModule {
        sources: Vec<String>,
        style: ContainerStyle,
        config: Table,
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
        ) -> Element {
            let Some(custom) = self.modules.get(variant) else {
                log::error!("Invalid variant name of custom module: {variant}");
                return "Invalid variant name".into();
            };
            custom.token.render(context, anchor, &custom.style)
        }
        fn read_config(
            &mut self,
            variant: &str,
            config: Table,
            style: ContainerStyle,
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
                    sources: vec![],
                    style,
                    token: engine.render_token(format),
                    config,
                },
            );
        }
    }
}
