use std::collections::HashMap;

use iced::Color;
use serde::{de::DeserializeOwned, Deserialize, Deserializer};

/// For the purpose of overwriting configuration presets at runtime, configuration structs are
/// automatically duplicated with all fields wrapped in [Option]. [AcceptOption] makes it easy to
/// generate implementations that accept both the standard field type [T] and the wrapped
/// [Option<T>]
pub trait AcceptOption<T> {
    /// If true, [Self] is [Option<T>]. If false, [Self] is [T].
    const IS_OPTION: bool;
    fn as_opt(&self) -> Option<&T>;
    fn as_opt_mut(&mut self) -> Option<&mut T>;
    fn into_opt(self) -> Option<T>;
    fn from_opt(opt: Option<T>) -> Self;
    fn deserialize_v<'de, D, V>(deserializer: D) -> Result<Option<V>, D::Error>
    where
        D: Deserializer<'de>,
        V: DeserializeOwned;
}
impl<T> AcceptOption<T> for T
where
    T: ImplAcceptOption,
{
    const IS_OPTION: bool = false;
    fn as_opt(&self) -> Option<&T> {
        Some(self)
    }
    fn as_opt_mut(&mut self) -> Option<&mut T> {
        Some(self)
    }
    fn into_opt(self) -> Option<T> {
        Some(self)
    }
    fn from_opt(opt: Option<T>) -> Self {
        opt.unwrap()
    }
    fn deserialize_v<'de, D, V>(deserializer: D) -> Result<Option<V>, D::Error>
    where
        D: Deserializer<'de>,
        V: DeserializeOwned,
    {
        Ok(Some(V::deserialize(deserializer)?))
    }
}
impl<T> AcceptOption<T> for Option<T>
where
    T: AcceptOption<T>,
{
    const IS_OPTION: bool = true;
    fn as_opt(&self) -> Option<&T> {
        self.as_ref()
    }
    fn as_opt_mut(&mut self) -> Option<&mut T> {
        self.as_mut()
    }
    fn into_opt(self) -> Option<T> {
        self
    }
    fn from_opt(opt: Option<T>) -> Self {
        opt
    }
    fn deserialize_v<'de, D, V>(deserializer: D) -> Result<Option<V>, D::Error>
    where
        D: Deserializer<'de>,
        V: DeserializeOwned,
    {
        <Option<_> as Deserialize>::deserialize(deserializer)
    }
}

/// A helper trait that allows to automatically implement [AcceptOption] for [T] by implementing
/// [ImplAcceptOption] for [T]
pub trait ImplAcceptOption {}

impl ImplAcceptOption for String {}
impl ImplAcceptOption for Vec<String> {}
impl ImplAcceptOption for u32 {}
impl ImplAcceptOption for f32 {}
impl ImplAcceptOption for bool {}
impl ImplAcceptOption for HashMap<String, Color> {}
