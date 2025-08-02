use clap::Args;
use iced::Color;
use optfield::optfield;
use serde::{Deserialize, Serialize};

#[optfield(
    pub ThemeOverride,
    attrs = (derive(Args, Debug, Clone, Serialize, Deserialize)),
    field_doc,
    field_attrs = add(arg(long, value_parser = parse_color)),
    merge_fn = pub
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: iced::color!(0x000000, 0.5),
            mod_background: iced::color!(0x000000, 0.),
            text: iced::color!(0xffffff),
            primary: iced::color!(0x0000ff),
            success: iced::color!(0x00ff00),
            danger: iced::color!(0xff0000),
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

fn parse_color(value: &str) -> Result<Color, csscolorparser::ParseColorError> {
    let color = csscolorparser::parse(value)?;
    Ok(Color::from(color.to_array()))
}

mod serde_with {
    use iced::Color;
    use serde::{Deserializer, Serialize as _, Serializer};

    use crate::accept_option::AcceptOption;

    pub fn serialize<S, A>(value: &A, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        A: AcceptOption<Color>,
    {
        let (value, is_opt) = value.as_opt();
        let Some(value) = value else {
            return serializer.serialize_none();
        };
        let color = csscolorparser::Color::from(value.into_rgba8());
        if is_opt {
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
