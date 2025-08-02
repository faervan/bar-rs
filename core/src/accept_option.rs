use serde::{de::DeserializeOwned, Deserialize as _, Deserializer};

pub trait AcceptOption<T> {
    /// If bool is true, the T was wrapped in Option before as_opt was called
    fn as_opt(&self) -> (Option<&T>, bool);
    fn from_opt(opt: Option<T>) -> Self;
    fn deserialize_v<'de, D, V>(deserializer: D) -> Result<Option<V>, D::Error>
    where
        D: Deserializer<'de>,
        V: DeserializeOwned;
}
impl<T> AcceptOption<T> for T {
    fn as_opt(&self) -> (Option<&T>, bool) {
        (Some(self), false)
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
    fn as_opt(&self) -> (Option<&T>, bool) {
        (self.as_ref(), true)
    }
    fn from_opt(opt: Option<T>) -> Self {
        opt
    }
    fn deserialize_v<'de, D, V>(deserializer: D) -> Result<Option<V>, D::Error>
    where
        D: Deserializer<'de>,
        V: DeserializeOwned,
    {
        Option::deserialize(deserializer)
    }
}
