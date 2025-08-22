use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
