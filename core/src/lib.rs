pub use derive;

pub mod config;
pub mod daemon;
pub mod directories;
pub mod ipc;
mod message;
pub mod module;
pub mod registry;
pub mod serde_with;
mod state;
mod subscription;
pub mod template_engine;
pub mod window;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use iced::{Color, Padding};
    use toml_example::TomlExample;

    use crate::config::{
        style::{ColorDescriptor, ContainerStyle, Style},
        theme::Theme,
        ConfigOptions,
    };

    #[test]
    fn verify_default_config() {
        let example = ConfigOptions::toml_example();
        assert_eq!(
            toml::from_str::<ConfigOptions>(&example).unwrap(),
            ConfigOptions::default()
        );
    }

    #[test]
    fn verify_default_style_config() {
        let example = ContainerStyle::toml_example();
        let default_style = Style {
            font_size: 16.,
            color: ColorDescriptor::Color(Color::WHITE),
            background_color: None,
            margin: Padding::ZERO,
        };
        assert_eq!(
            toml::from_str::<ContainerStyle>(&example).unwrap(),
            ContainerStyle {
                style: default_style.clone(),
                padding: Padding::from([4, 10]),
                class: HashMap::from([(String::from("example"), default_style)])
            }
        );
    }

    #[test]
    fn verify_default_theme_config() {
        let example = Theme::toml_example();
        assert_eq!(toml::from_str::<Theme>(&example).unwrap(), Theme::default());
    }
}
