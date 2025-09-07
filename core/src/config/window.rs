use clap::Args;
use merge::Merge;
use optfield::optfield;
use serde::{Deserialize, Serialize};
use smithay_client_toolkit::shell::wlr_layer::{Anchor, KeyboardInteractivity};
use toml_example::TomlExample;

use crate::helpers::merge::overwrite_if_some;

#[optfield(
    pub WindowConfigOverride,
    attrs = add(derive(Default)),
    field_doc,
    field_attrs,
    merge_fn = pub
)]
#[derive(Args, Merge, Debug, Clone, Serialize, Deserialize, TomlExample)]
#[serde(default)]
pub struct WindowConfig {
    #[arg(long, value_parser = clap_parser::parse_anchor)]
    #[serde(with = "serde_with::anchor")]
    #[toml_example(default = "Top")]
    #[merge(strategy = overwrite_if_some)]
    /// The anchor to use when positioning the window. May be `top`, `bottom`, `left` or `right`
    pub anchor: Anchor,

    #[arg(long, value_parser = clap_parser::parse_monitor)]
    #[serde(with = "serde_with::monitor")]
    #[toml_example(enum)]
    #[merge(strategy = overwrite_if_some)]
    /// The monitor to open on
    pub monitor: MonitorSelection,

    #[arg(long)]
    #[merge(strategy = overwrite_if_some)]
    /// The height of the window
    pub height: u32,

    #[arg(long)]
    #[merge(strategy = overwrite_if_some)]
    /// The width of the window
    pub width: u32,

    #[arg(long, value_parser = clap_parser::parse_keyboard)]
    #[serde(with = "serde_with::keyboard")]
    #[toml_example(enum)]
    #[merge(strategy = overwrite_if_some)]
    /// Determines if the window should be focusable and receive keyboard inputs. May be `none`,
    /// `on_demand` or `exclusive`.
    pub keyboard_focus: KeyboardInteractivity,
}

impl PartialEq for WindowConfig {
    fn eq(&self, other: &Self) -> bool {
        let monitor_equal = match self.monitor {
            MonitorSelection::All => matches!(other.monitor, MonitorSelection::All),
            MonitorSelection::Active => matches!(other.monitor, MonitorSelection::Active),
            MonitorSelection::Name(ref name) => {
                matches!(&other.monitor, MonitorSelection::Name(n) if n == name)
            }
        };
        self.anchor == other.anchor
            && self.height == other.height
            && self.width == other.width
            && self.keyboard_focus == other.keyboard_focus
            && monitor_equal
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            anchor: Anchor::BOTTOM,
            monitor: MonitorSelection::Active,
            height: 30,
            width: 1000,
            keyboard_focus: KeyboardInteractivity::None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MonitorSelection {
    All,
    Active,
    Name(String),
}

mod clap_parser {
    use smithay_client_toolkit::shell::wlr_layer::{Anchor, KeyboardInteractivity};

    use super::MonitorSelection;

    pub fn parse_anchor(value: &str) -> Result<Anchor, &'static str> {
        Anchor::from_name(&value.to_uppercase())
            .ok_or("allowed anchors are: `top`, `bottom`, `left` and `right`")
    }

    pub fn parse_keyboard(value: &str) -> Result<KeyboardInteractivity, &'static str> {
        Ok(match value {
            "none" => KeyboardInteractivity::None,
            "on_demand" => KeyboardInteractivity::OnDemand,
            "exclusive" => KeyboardInteractivity::Exclusive,
            _ => return Err(
                "allowed keyboard_interactivity values are: `none`, `on_demand` and `exclusive`",
            ),
        })
    }

    pub fn parse_monitor(value: &str) -> Result<MonitorSelection, &'static str> {
        Ok(match value {
            "all" => MonitorSelection::All,
            "active" => MonitorSelection::Active,
            name => MonitorSelection::Name(String::from(name)),
        })
    }
}

mod serde_with {
    use smithay_client_toolkit::shell::wlr_layer::{Anchor, KeyboardInteractivity};

    use crate::helpers::accept_option::ImplAcceptOption;

    macro_rules! gen_serde_with {
        ($mod_name:ident, $type:ty, [ $( ($variant_name:expr, $variant_value:path) ),* $(,)? ]) => {
            pub mod $mod_name {
                #[allow(unused_imports)]
                use smithay_client_toolkit::shell::wlr_layer::Anchor;
                #[allow(unused_imports)]
                use smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity;

                use serde::{de::Error as _, ser::Error as _};

                pub fn serialize<S, A>(value: &A, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::ser::Serializer,
                    A: crate::helpers::accept_option::AcceptOption<$type>,
                {
                    let value = value.as_opt();
                    let Some(value) = value else {
                        return serializer.serialize_none();
                    };
                    let string = match *value {
                        $(
                            $variant_value => $variant_name,
                        )*
                        _ => {
                            return Err(S::Error::custom(format!(
                                "No string representation for {value:?} defined"
                            )))
                        }
                    };
                    if A::IS_OPTION {
                        serializer.serialize_some(string)
                    } else {
                        serializer.serialize_str(string)
                    }
                }
                pub fn deserialize<'de, D, A>(deserializer: D) -> Result<A, D::Error>
                where
                    D: serde::de::Deserializer<'de>,
                    A: crate::helpers::accept_option::AcceptOption<$type>,
                {
                    let value: Option<String> = A::deserialize_v(deserializer)?;
                    Ok(A::from_opt(match value {
                        Some(value) => Some(match value.as_str() {
                            $(
                                $variant_name => $variant_value,
                            )*
                            _ => {
                                return Err(D::Error::custom(format!("Invalid value name: {value}")))
                            }
                        }),
                        None => None,
                    }))
                }
            }
        };
    }

    impl ImplAcceptOption for Anchor {}

    gen_serde_with!(
        anchor,
        Anchor,
        [
            ("top", Anchor::TOP),
            ("bottom", Anchor::BOTTOM),
            ("left", Anchor::LEFT),
            ("right", Anchor::RIGHT)
        ]
    );

    impl ImplAcceptOption for KeyboardInteractivity {}

    gen_serde_with!(
        keyboard,
        KeyboardInteractivity,
        [
            ("none", KeyboardInteractivity::None),
            ("on_demand", KeyboardInteractivity::OnDemand),
            ("exclusive", KeyboardInteractivity::Exclusive)
        ]
    );

    pub mod monitor {
        use crate::{config::window::MonitorSelection, helpers::accept_option::ImplAcceptOption};

        impl ImplAcceptOption for MonitorSelection {}

        pub fn serialize<S, A>(value: &A, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::ser::Serializer,
            A: crate::helpers::accept_option::AcceptOption<MonitorSelection>,
        {
            let value = value.as_opt();
            let Some(value) = value else {
                return serializer.serialize_none();
            };
            let string = match value {
                MonitorSelection::All => "all",
                MonitorSelection::Active => "active",
                MonitorSelection::Name(name) => name.as_str(),
            };
            if A::IS_OPTION {
                serializer.serialize_some(string)
            } else {
                serializer.serialize_str(string)
            }
        }
        pub fn deserialize<'de, D, A>(deserializer: D) -> Result<A, D::Error>
        where
            D: serde::de::Deserializer<'de>,
            A: crate::helpers::accept_option::AcceptOption<MonitorSelection>,
        {
            let value: Option<String> = A::deserialize_v(deserializer)?;
            Ok(A::from_opt(value.map(|value| match value.as_str() {
                "all" => MonitorSelection::All,
                "active" => MonitorSelection::Active,
                name => MonitorSelection::Name(String::from(name)),
            })))
        }
    }
}
