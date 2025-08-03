use std::{collections::HashMap, fmt::Debug};

use iced::Element;
use smithay_client_toolkit::shell::wlr_layer::Anchor;
use toml::Table;

use crate::{config::style::ContainerStyle, registry::Registry, template_engine::TemplateEngine};

pub type Context = HashMap<String, Box<dyn ToString + Send + Sync>>;

pub trait Module<Message: 'static>: Debug {
    fn variant_names(&self) -> Vec<&str>;

    fn active(&self) -> bool {
        true
    }

    fn view(&self, variant: &str, anchor: &Anchor, context: &Context) -> Element<Message>;

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
        engine: TemplateEngine<Message>,
    ) {
    }
}

pub fn register_modules<Message: 'static>(registry: &mut Registry<Message>) {}

mod custom {
    use std::{collections::HashMap, fmt::Debug};

    use toml::Table;

    use crate::{config::style::ContainerStyle, template_engine::Token};

    use super::Module;

    #[derive(Debug)]
    pub struct CustomModules<Message: 'static + Debug> {
        modules: HashMap<String, CustomModule<Message>>,
    }

    struct CustomModule<Message: 'static> {
        sources: Vec<String>,
        style: ContainerStyle,
        config: Table,
        token: Box<dyn Token<Message>>,
    }

    impl<Message: 'static + Debug> Debug for CustomModule<Message> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "CustomModule")
        }
    }

    impl<Message: 'static + Debug> Module<Message> for CustomModules<Message> {
        fn variant_names(&self) -> Vec<&str> {
            self.modules.keys().map(String::as_str).collect()
        }
        fn view(
            &self,
            variant: &str,
            anchor: &smithay_client_toolkit::shell::wlr_layer::Anchor,
            context: &super::Context,
        ) -> iced::Element<Message> {
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
            engine: crate::template_engine::TemplateEngine<Message>,
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
