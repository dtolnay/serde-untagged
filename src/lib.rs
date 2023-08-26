mod error;

use serde::de::{Deserializer, Expected, Visitor};
use std::fmt;
use std::marker::PhantomData;

pub use crate::error::Error;

pub struct UntaggedEnumVisitor<'closure, 'de, Value> {
    visit_str: Option<Box<dyn FnOnce(&str) -> Result<Value, Error> + 'closure>>,
    visit_borrowed_str: Option<Box<dyn FnOnce(&'de str) -> Result<Value, Error> + 'closure>>,
}

impl<'closure, 'de, Value> UntaggedEnumVisitor<'closure, 'de, Value> {
    pub fn new() -> Self {
        UntaggedEnumVisitor {
            visit_str: None,
            visit_borrowed_str: None,
        }
    }

    pub fn string(mut self, visit: impl FnOnce(&str) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_str = Some(Box::new(visit));
        self
    }

    pub fn borrowed_str(
        mut self,
        visit: impl FnOnce(&'de str) -> Result<Value, Error> + 'closure,
    ) -> Self {
        self.visit_borrowed_str = Some(Box::new(visit));
        self
    }

    pub fn deserialize<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'closure, 'de, Value> Visitor<'de> for UntaggedEnumVisitor<'closure, 'de, Value> {
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
            DefaultVisitor::new(&self).visit_str(v)
        }
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_borrowed_str) = self.visit_borrowed_str {
            visit_borrowed_str(v).map_err(error::convert)
        } else {
            self.visit_str(v)
        }
    }
}

struct DefaultVisitor<'a, E, T> {
    expected: &'a E,
    value: PhantomData<T>,
}

impl<'a, E, T> DefaultVisitor<'a, E, T> {
    fn new(expected: &'a E) -> Self {
        DefaultVisitor {
            expected,
            value: PhantomData,
        }
    }
}

impl<'a, 'de, V, T> Visitor<'de> for DefaultVisitor<'a, V, T>
where
    V: Expected,
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.expected.fmt(formatter)
    }
}
