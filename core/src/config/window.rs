use clap::Args;
use optfield::optfield;
use serde::{Deserialize, Serialize};
use smithay_client_toolkit::shell::wlr_layer::{Anchor, KeyboardInteractivity};

#[optfield(
    pub WindowConfigOverride,
    attrs = (derive(Args, Debug, Clone, Serialize, Deserialize)),
    field_doc,
    field_attrs,
    merge_fn = pub
)]
#[derive(Args, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    #[arg(long, value_parser = clap_parser::parse_anchor)]
    #[serde(with = "serde_with::anchor")]
    /// The anchor to use when positioning the window. May be `top`, `bottom`, `left` or `right`
    pub anchor: Anchor,

    #[arg(long)]
    /// The height of the window
    pub height: u32,

    #[arg(long)]
    /// The width of the window
    pub width: u32,

    #[arg(long, value_parser = clap_parser::parse_keyboard)]
    #[serde(with = "serde_with::keyboard")]
    /// Determines if the window should be focusable and receive keyboard inputs. May be `none`,
    /// `on_demand` or `exclusive`.
    pub keyboard_focus: KeyboardInteractivity,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            anchor: Anchor::BOTTOM,
            height: 30,
            width: 1000,
            keyboard_focus: KeyboardInteractivity::None,
        }
    }
}

mod clap_parser {
    use smithay_client_toolkit::shell::wlr_layer::{Anchor, KeyboardInteractivity};

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
}

mod serde_with {
    macro_rules! gen_serde_with {
        ($mod_name:ident, $type:ty, [ $( ($variant_name:expr, $variant_value:path) ),* $(,)? ]) => {
            pub mod $mod_name {
                #[allow(unused_imports)]
                use smithay_client_toolkit::shell::wlr_layer::Anchor;
                #[allow(unused_imports)]
                use smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity;
                use serde::{de::Error as _, ser::Error as _};

                #[allow(private_bounds)]
                pub fn serialize<S, A>(value: &A, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::ser::Serializer,
                    A: crate::accept_option::AcceptOption<$type>,
                {
                    let (value, is_opt) = value.as_opt();
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
                    if is_opt {
                        serializer.serialize_some(string)
                    } else {
                        serializer.serialize_str(string)
                    }
                }
                #[allow(private_bounds)]
                pub fn deserialize<'de, D, A>(deserializer: D) -> Result<A, D::Error>
                where
                    D: serde::de::Deserializer<'de>,
                    A: crate::accept_option::AcceptOption<$type>,
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

    gen_serde_with!(
        keyboard,
        KeyboardInteractivity,
        [
            ("none", KeyboardInteractivity::None),
            ("on_demand", KeyboardInteractivity::OnDemand),
            ("exclusive", KeyboardInteractivity::Exclusive)
        ]
    );
}
