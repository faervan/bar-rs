use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

pub trait AcceptOption<T> {
    /// If bool is true, the T was wrapped in Option before as_opt was called
    fn as_opt(&self) -> (Option<&T>, bool);
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
pub trait ImplAcceptOption {}
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
        <Option<_> as Deserialize>::deserialize(deserializer)
    }
}

pub trait SerdeIntermediate<I>
where
    Self: Sized,
{
    fn serialize<'a, S>(&'a self, serilizer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        I: From<&'a Self> + Serialize;
    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        I: Into<Self> + Deserialize<'de>;
}

impl<T, I> SerdeIntermediate<I> for T
where
    T: ImplSerdeIntermediate<I>,
{
    fn serialize<'a, S>(&'a self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        I: From<&'a Self> + Serialize,
    {
        I::from(self).serialize(serializer)
    }
    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        I: Into<Self> + Deserialize<'de>,
    {
        Ok(I::deserialize(deserializer)?.into())
    }
}

pub trait ImplSerdeIntermediate<I> {}
