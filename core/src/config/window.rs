use clap::Args;
use optfield::optfield;
use serde::{Deserialize, Serialize};
use smithay_client_toolkit::shell::wlr_layer::Anchor;

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
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            anchor: Anchor::BOTTOM,
        }
    }
}

mod clap_parser {
    use smithay_client_toolkit::shell::wlr_layer::Anchor;

    pub fn parse_anchor(value: &str) -> Result<Anchor, &'static str> {
        Anchor::from_name(&value.to_uppercase())
            .ok_or("allowed anchors are: `top`, `bottom`, `left` and `right`")
    }
}

mod serde_with {
    use serde::{Deserialize as _, Deserializer};

    trait AcceptOption<T> {
        /// If bool is true, the T was wrapped in Option before as_opt was called
        fn as_opt(&self) -> (Option<&T>, bool);
        fn from_opt(opt: Option<T>) -> Self;
        fn deserialize_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
        where
            D: Deserializer<'de>;
    }
    impl<T> AcceptOption<T> for T {
        fn as_opt(&self) -> (Option<&T>, bool) {
            (Some(self), false)
        }
        fn from_opt(opt: Option<T>) -> Self {
            opt.unwrap()
        }
        fn deserialize_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(Some(String::deserialize(deserializer)?))
        }
    }
    impl<T> AcceptOption<T> for Option<T>
    where
        T: AcceptOption<T>,
    {
        fn as_opt(&self) -> (Option<&T>, bool) {
            (self.as_ref(), true)
        }
        fn from_opt(opt: Option<T>) -> Self {
            opt
        }
        fn deserialize_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
        where
            D: Deserializer<'de>,
        {
            Option::deserialize(deserializer)
        }
    }

    macro_rules! gen_serde_with {
        ($mod_name:ident, $type:ty, [ $( ($variant_name:expr, $variant_value:path) ),* $(,)? ]) => {
            pub mod $mod_name {
                use smithay_client_toolkit::shell::wlr_layer::Anchor;
                use serde::{de::Error as _, ser::Error as _};

                #[allow(private_bounds)]
                pub fn serialize<S, A>(value: &A, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::ser::Serializer,
                    A: super::AcceptOption<$type>,
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
                    A: super::AcceptOption<$type>,
                {
                    let value = A::deserialize_string(deserializer)?;
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

    gen_serde_with!(anchor, Anchor, [("top", Anchor::TOP)]);
}
