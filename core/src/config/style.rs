use std::collections::HashMap;

use iced::{Color, Padding};
use optfield::optfield;
use serde::{Deserialize, Serialize};
use toml_example::TomlExample;

#[optfield(
    pub StyleOverride,
    attrs = (derive(Debug, Clone, Serialize, Deserialize, TomlExample)),
    field_doc,
    field_attrs,
    merge_fn = pub
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TomlExample)]
#[serde(default)]
pub struct Style {
    /// The size of text (and text icons)
    pub font_size: f32,

    #[serde(with = "serde_with_color")]
    #[toml_example(default = "#fff")]
    /// The font color
    pub color: ColorDescriptor,

    #[serde(with = "serde_with_color")]
    #[toml_example(default = "$background")]
    /// The background color
    pub background_color: Option<ColorDescriptor>,

    #[serde(with = "serde_with_padding")]
    #[toml_example(default = [0])]
    /// The space around this item separating it from neighboring items
    pub margin: Padding,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            font_size: 16.,
            color: ColorDescriptor::ThemeColor("text".to_string()),
            background_color: None,
            margin: Padding::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorDescriptor {
    Color(Color),
    ThemeColor(String),
}

// Note: all fields from ContainerStyle need to be present for ContainerStyleOverride as well!
#[derive(Debug, Clone, Serialize, Deserialize, TomlExample, Default, PartialEq)]
pub struct ContainerStyle {
    #[toml_example(nesting)]
    /// The style of this item and default style for all contained items
    pub style: Style,

    #[serde(with = "serde_with_padding")]
    #[toml_example(default = [4, 10])]
    /// The space around the contained items
    pub padding: Padding,

    #[toml_example(nesting)]
    /// Style classes available for all contained items
    pub class: HashMap<String, Style>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStyleOverride {
    pub style: StyleOverride,

    #[serde(with = "serde_with_padding")]
    /// The space around the contained items
    pub padding: Option<Padding>,

    pub class: HashMap<String, StyleOverride>,
}

impl ContainerStyle {
    pub fn merge_opt(
        &mut self,
        ContainerStyleOverride {
            style,
            padding,
            class,
        }: ContainerStyleOverride,
    ) {
        self.style.merge_opt(style);
        if let Some(padding) = padding {
            self.padding = padding;
        }
        for (class, style_override) in class {
            match self.class.get_mut(&class) {
                Some(style) => style.merge_opt(style_override),
                None => {
                    let mut style = Style::default();
                    style.merge_opt(style_override);
                    self.class.insert(class, style);
                }
            }
        }
    }

    pub fn class(&self, class: &str) -> &Style {
        self.class.get(class).unwrap_or(&self.style)
    }
}

mod serde_with_padding {
    use iced::Padding;
    use serde::{de::Error as _, ser::SerializeSeq as _, Deserializer, Serializer};

    use crate::helpers::accept_option::{AcceptOption, ImplAcceptOption};

    impl ImplAcceptOption for Padding {}

    pub fn serialize<S, A>(value: &A, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        A: AcceptOption<Padding>,
    {
        let value = value.as_opt();
        let Some(value) = value else {
            return serializer.serialize_none();
        };
        let vec = match value {
            _ if value.top == value.bottom
                && value.top == value.left
                && value.top == value.right =>
            {
                vec![value.top]
            }
            _ if value.top == value.bottom && value.left == value.right => {
                vec![value.left, value.top]
            }
            _ => vec![value.top, value.right, value.bottom, value.left],
        };
        if A::IS_OPTION {
            serializer.serialize_some(&vec)
        } else {
            let mut seq = serializer.serialize_seq(Some(vec.len()))?;
            for item in &vec {
                seq.serialize_element(item)?;
            }
            seq.end()
        }
    }
    pub fn deserialize<'de, D, A>(deserializer: D) -> Result<A, D::Error>
    where
        D: Deserializer<'de>,
        A: AcceptOption<Padding>,
    {
        let vec: Option<Vec<f32>> = A::deserialize_v(deserializer)?;
        let padding = match vec {
            Some(vec) => Some(match vec.len() {
                1 => Padding::from(vec[0]),
                2 => Padding::from([vec[0], vec[1]]),
                4 => Padding::from([vec[0], vec[1], vec[2], vec[3]]),
                _ => {
                    return Err(D::Error::custom(
                        "Invalid number of values: expected 1, 2 or 4",
                    ))
                }
            }),
            None => None,
        };
        Ok(A::from_opt(padding))
    }
}

mod serde_with_color {
    use iced::Color;
    use serde::{de::Error as _, Deserializer, Serialize as _, Serializer};

    use crate::helpers::accept_option::{AcceptOption, ImplAcceptOption};

    use super::ColorDescriptor;

    impl ImplAcceptOption for ColorDescriptor {}

    pub fn serialize<S, A>(value: &A, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        A: AcceptOption<ColorDescriptor>,
    {
        let value = value.as_opt();
        let Some(value) = value else {
            return serializer.serialize_none();
        };
        let string = match value {
            ColorDescriptor::Color(color) => {
                let css_color = csscolorparser::Color::from(color.into_rgba8());
                match css_color.name() {
                    Some(name) => name.to_string(),
                    None => css_color.to_css_hex(),
                }
            }
            ColorDescriptor::ThemeColor(name) => format!("${name}"),
        };
        if A::IS_OPTION {
            Some(string).serialize(serializer)
        } else {
            string.serialize(serializer)
        }
    }
    pub fn deserialize<'de, D, A>(deserializer: D) -> Result<A, D::Error>
    where
        D: Deserializer<'de>,
        A: AcceptOption<ColorDescriptor>,
    {
        let string: Option<String> = A::deserialize_v(deserializer)?;
        let color_descriptor = match string {
            Some(string) => Some(match string.strip_prefix('$') {
                Some(name) => ColorDescriptor::ThemeColor(name.to_string()),
                None => ColorDescriptor::Color(Color::from(
                    csscolorparser::parse(&string)
                        .map_err(D::Error::custom)?
                        .to_array(),
                )),
            }),
            None => None,
        };
        Ok(A::from_opt(color_descriptor))
    }
}
