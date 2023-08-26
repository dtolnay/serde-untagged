mod error;

use serde::de::{Deserializer, Unexpected, Visitor};
use std::fmt;

pub use crate::error::Error;

pub struct UntaggedEnumVisitor<'closure, Value> {
    visit_str: Option<Box<dyn FnOnce(&str) -> Result<Value, Error> + 'closure>>,
}

impl<'closure, Value> UntaggedEnumVisitor<'closure, Value> {
    pub fn new() -> Self {
        UntaggedEnumVisitor { visit_str: None }
    }

    pub fn string(mut self, visit: impl FnOnce(&str) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_str = Some(Box::new(visit));
        self
    }

    pub fn deserialize<'de, D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'closure, 'de, Value> Visitor<'de> for UntaggedEnumVisitor<'closure, Value> {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("TODO")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_str) = self.visit_str {
            visit_str(v).map_err(error::convert)
        } else {
            Err(E::invalid_type(Unexpected::Str(v), &self))
        }
    }
}
