use serde::de::{Deserializer, Visitor};
use std::fmt;
use std::marker::PhantomData;

pub struct UntaggedEnumVisitor<Value> {
    value: PhantomData<Value>,
}

impl<Value> UntaggedEnumVisitor<Value> {
    pub fn new() -> Self {
        UntaggedEnumVisitor { value: PhantomData }
    }

    pub fn deserialize<'de, D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, Value> Visitor<'de> for UntaggedEnumVisitor<Value> {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("TODO")
    }
}
