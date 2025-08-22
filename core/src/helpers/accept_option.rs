use serde::{de::DeserializeOwned, Deserialize, Deserializer};

pub trait AcceptOption<T> {
    /// If true, [Self] is [Option<T>]. If false, [Self] is [T].
    const IS_OPTION: bool;
    fn as_opt(&self) -> Option<&T>;
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
