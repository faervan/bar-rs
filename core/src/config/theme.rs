use clap::Args;
use iced::Color;
use optfield::optfield;
use serde::{Deserialize, Serialize};

#[optfield(
    pub ThemeOverride,
    attrs = (derive(Args, Debug, Clone)),
    field_doc,
    field_attrs = (arg(long, value_parser = parse_color)),
    merge_fn = pub
)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Theme {
    #[serde(with = "serde_with")]
    /// The background of the bar
    pub background: Color,

    #[serde(with = "serde_with")]
    /// The background of the modules
    pub mod_background: Color,

    #[serde(with = "serde_with")]
    /// Normal text color
    pub text: Color,

    #[serde(with = "serde_with")]
    /// Special/foreground text color
    pub primary: Color,

    #[serde(with = "serde_with")]
    /// Color of success
    pub success: Color,

    #[serde(with = "serde_with")]
    /// Color of failure
    pub danger: Color,
}

fn parse_color(value: &str) -> Result<Color, csscolorparser::ParseColorError> {
    let color = csscolorparser::parse(value)?;
    Ok(Color::from(color.to_array()))
}

mod serde_with {
    use iced::Color;
    use serde::{Deserialize as _, Deserializer, Serialize as _, Serializer};

    pub fn serialize<S>(value: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let color = csscolorparser::Color::from(value.into_rgba8());
        color.serialize(serializer)
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let color = csscolorparser::Color::deserialize(deserializer)?;
        Ok(Color::from(color.to_array()))
    }
}
