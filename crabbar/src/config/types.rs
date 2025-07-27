use serde::Deserialize;
use smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity;

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BarAnchor {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KbFocus {
    #[default]
    None,
    Exclusive,
    OnDemand,
}

impl From<&KbFocus> for KeyboardInteractivity {
    fn from(value: &KbFocus) -> Self {
        match value {
            KbFocus::None => KeyboardInteractivity::None,
            KbFocus::Exclusive => KeyboardInteractivity::Exclusive,
            KbFocus::OnDemand => KeyboardInteractivity::OnDemand,
        }
    }
}
