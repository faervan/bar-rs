use std::collections::HashMap;

use clap::Args;
use iced::Color;
use optfield::optfield;
use serde::{Deserialize, Serialize};
use toml_example::TomlExample;

use crate::helpers::serde_with::SerdeIntermediate;

#[optfield(
    pub ThemeOverride,
    attrs,
    field_doc,
    field_attrs,
    merge_fn = pub
)]
#[derive(Args, Debug, Clone, Serialize, Deserialize, TomlExample, PartialEq)]
pub struct Theme {
    #[serde(with = "serde_with")]
    #[toml_example(default = "rgba(0, 0, 0, 0.5)")]
    #[arg(long, value_parser = clap_parse::color)]
    /// The background of the bar
    pub background: Color,

    #[serde(with = "serde_with")]
    #[toml_example(default = "#0000")]
    #[arg(long, value_parser = clap_parse::color)]
    /// The background of the modules
    pub mod_background: Color,

    #[serde(with = "serde_with")]
    #[toml_example(default = "white")]
    #[arg(long, value_parser = clap_parse::color)]
    /// Normal text color
    pub text: Color,

    #[serde(with = "serde_with")]
    #[toml_example(default = "rgb(0, 0, 255)")]
    #[arg(long, value_parser = clap_parse::color)]
    /// Special/foreground text color
    pub primary: Color,

    #[serde(with = "serde_with")]
    #[toml_example(default = "#0f0")]
    #[arg(long, value_parser = clap_parse::color)]
    /// Color of success
    pub success: Color,

    #[serde(with = "serde_with")]
    #[toml_example(default = "red")]
    #[arg(long, value_parser = clap_parse::color)]
    /// Color of failure
    pub danger: Color,

    #[serde(default, with = "SerdeIntermediate")]
    #[toml_example(skip)]
    #[arg(
        long,
        value_parser = clap_parse::custom_colors,
        help = "Additional custom color variables\n    \
                Example: `--custom \"my_color1=blue my_color2=#fed\"`"
    )]
    /// Additional custom color variables
    pub custom: HashMap<String, Color>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: iced::color!(0x000000, 0.5),
            mod_background: iced::color!(0x000000, 0.),
            text: iced::color!(0xffffff),
            primary: iced::color!(0x0000ff),
            success: iced::color!(0x00ff00),
            danger: iced::color!(0xff0000),
            custom: HashMap::default(),
        }
    }
}

impl From<&Theme> for iced::theme::Palette {
    fn from(theme: &Theme) -> Self {
        Self {
            background: theme.background,
            text: theme.text,
            primary: theme.primary,
            success: theme.success,
            danger: theme.danger,
        }
    }
}

mod clap_parse {
    use std::collections::HashMap;

    use clap::Parser;
    use iced::Color;

    pub fn color(value: &str) -> Result<Color, csscolorparser::ParseColorError> {
        let color = csscolorparser::parse(value)?;
        Ok(Color::from(color.to_array()))
    }

    pub fn custom_colors(value: &str) -> anyhow::Result<HashMap<String, Color>> {
        #[derive(Parser)]
        #[command(no_binary_name = true)]
        struct Custom {
            colors: Vec<String>,
        }
        let custom = Custom::try_parse_from(value.split_whitespace())?;
        custom
            .colors
            .into_iter()
            .map(|pair| {
                pair.split_once('=')
                    .ok_or(anyhow::anyhow!(
                        "Invalid key value pair, expected `VARIABLE=COLOR`"
                    ))
                    .and_then(|(k, v)| {
                        csscolorparser::parse(v)
                            .map(|v| (k.to_string(), Color::from(v.to_array())))
                            .map_err(|e| anyhow::anyhow!("{v}: invalid color value: {e}"))
                    })
            })
            .collect::<Result<_, _>>()
    }
}

mod serde_with {
    use iced::Color;
    use serde::{Deserializer, Serialize as _, Serializer};

    use crate::helpers::accept_option::{AcceptOption, ImplAcceptOption};

    impl ImplAcceptOption for Color {}

    pub fn serialize<S, A>(value: &A, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        A: AcceptOption<Color>,
    {
        let value = value.as_opt();
        let Some(value) = value else {
            return serializer.serialize_none();
        };
        let color = csscolorparser::Color::from(value.into_rgba8());
        if A::IS_OPTION {
            Some(color).serialize(serializer)
        } else {
            color.serialize(serializer)
        }
    }
    pub fn deserialize<'de, D, A>(deserializer: D) -> Result<A, D::Error>
    where
        D: Deserializer<'de>,
        A: AcceptOption<Color>,
    {
        let color: Option<csscolorparser::Color> = A::deserialize_v(deserializer)?;
        Ok(A::from_opt(color.map(|c| Color::from(c.to_array()))))
    }
}

mod intermediate {
    use std::collections::HashMap;

    use iced::Color;
    use serde::{Deserialize, Serialize};

    use crate::helpers::serde_with::ImplSerdeIntermediate;

    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    struct ColorIntermediate(#[serde(with = "super::serde_with")] Color);

    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct MapIntermediate(HashMap<String, ColorIntermediate>);

    impl From<&HashMap<String, Color>> for MapIntermediate {
        fn from(value: &HashMap<String, Color>) -> Self {
            Self(
                value
                    .iter()
                    .map(|(k, v)| (k.clone(), ColorIntermediate(*v)))
                    .collect(),
            )
        }
    }

    impl From<MapIntermediate> for HashMap<String, Color> {
        fn from(value: MapIntermediate) -> Self {
            value.0.into_iter().map(|(k, v)| (k, v.0)).collect()
        }
    }

    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct OptionIntermediate(Option<MapIntermediate>);

    impl From<&Option<HashMap<String, Color>>> for OptionIntermediate {
        fn from(value: &Option<HashMap<String, Color>>) -> Self {
            OptionIntermediate(value.as_ref().map(Into::into))
        }
    }

    impl From<OptionIntermediate> for Option<HashMap<String, Color>> {
        fn from(value: OptionIntermediate) -> Self {
            value.0.map(Into::into)
        }
    }

    impl ImplSerdeIntermediate<MapIntermediate> for HashMap<String, Color> {}
    impl ImplSerdeIntermediate<OptionIntermediate> for Option<HashMap<String, Color>> {}
}
