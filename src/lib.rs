mod error;

use serde::de::{Deserializer, Expected, Visitor};
use std::fmt;
use std::marker::PhantomData;

pub use crate::error::Error;

pub struct UntaggedEnumVisitor<'closure, 'de, Value> {
    visit_bool: Option<Box<dyn FnOnce(bool) -> Result<Value, Error> + 'closure>>,
    visit_i8: Option<Box<dyn FnOnce(i8) -> Result<Value, Error> + 'closure>>,
    visit_i16: Option<Box<dyn FnOnce(i16) -> Result<Value, Error> + 'closure>>,
    visit_i32: Option<Box<dyn FnOnce(i32) -> Result<Value, Error> + 'closure>>,
    visit_i64: Option<Box<dyn FnOnce(i64) -> Result<Value, Error> + 'closure>>,
    visit_i128: Option<Box<dyn FnOnce(i128) -> Result<Value, Error> + 'closure>>,
    visit_u8: Option<Box<dyn FnOnce(u8) -> Result<Value, Error> + 'closure>>,
    visit_u16: Option<Box<dyn FnOnce(u16) -> Result<Value, Error> + 'closure>>,
    visit_u32: Option<Box<dyn FnOnce(u32) -> Result<Value, Error> + 'closure>>,
    visit_u64: Option<Box<dyn FnOnce(u64) -> Result<Value, Error> + 'closure>>,
    visit_u128: Option<Box<dyn FnOnce(u128) -> Result<Value, Error> + 'closure>>,
    visit_f32: Option<Box<dyn FnOnce(f32) -> Result<Value, Error> + 'closure>>,
    visit_f64: Option<Box<dyn FnOnce(f64) -> Result<Value, Error> + 'closure>>,
    visit_char: Option<Box<dyn FnOnce(char) -> Result<Value, Error> + 'closure>>,
    visit_str: Option<Box<dyn FnOnce(&str) -> Result<Value, Error> + 'closure>>,
    visit_borrowed_str: Option<Box<dyn FnOnce(&'de str) -> Result<Value, Error> + 'closure>>,
    visit_bytes: Option<Box<dyn FnOnce(&[u8]) -> Result<Value, Error> + 'closure>>,
    visit_borrowed_bytes: Option<Box<dyn FnOnce(&'de [u8]) -> Result<Value, Error> + 'closure>>,
    visit_byte_buf: Option<Box<dyn FnOnce(Vec<u8>) -> Result<Value, Error> + 'closure>>,
    visit_unit: Option<Box<dyn FnOnce() -> Result<Value, Error> + 'closure>>,
}

impl<'closure, 'de, Value> UntaggedEnumVisitor<'closure, 'de, Value> {
    pub fn new() -> Self {
        UntaggedEnumVisitor {
            visit_bool: None,
            visit_i8: None,
            visit_i16: None,
            visit_i32: None,
            visit_i64: None,
            visit_i128: None,
            visit_u8: None,
            visit_u16: None,
            visit_u32: None,
            visit_u64: None,
            visit_u128: None,
            visit_f32: None,
            visit_f64: None,
            visit_char: None,
            visit_str: None,
            visit_borrowed_str: None,
            visit_bytes: None,
            visit_borrowed_bytes: None,
            visit_byte_buf: None,
            visit_unit: None,
        }
    }

    pub fn bool(mut self, visit: impl FnOnce(bool) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_bool = Some(Box::new(visit));
        self
    }

    pub fn i8(mut self, visit: impl FnOnce(i8) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_i8 = Some(Box::new(visit));
        self
    }

    pub fn i16(mut self, visit: impl FnOnce(i16) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_i16 = Some(Box::new(visit));
        self
    }

    pub fn i32(mut self, visit: impl FnOnce(i32) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_i32 = Some(Box::new(visit));
        self
    }

    pub fn i64(mut self, visit: impl FnOnce(i64) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_i64 = Some(Box::new(visit));
        self
    }

    pub fn i128(mut self, visit: impl FnOnce(i128) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_i128 = Some(Box::new(visit));
        self
    }

    pub fn u8(mut self, visit: impl FnOnce(u8) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_u8 = Some(Box::new(visit));
        self
    }

    pub fn u16(mut self, visit: impl FnOnce(u16) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_u16 = Some(Box::new(visit));
        self
    }

    pub fn u32(mut self, visit: impl FnOnce(u32) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_u32 = Some(Box::new(visit));
        self
    }

    pub fn u64(mut self, visit: impl FnOnce(u64) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_u64 = Some(Box::new(visit));
        self
    }

    pub fn u128(mut self, visit: impl FnOnce(u128) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_u128 = Some(Box::new(visit));
        self
    }

    pub fn f32(mut self, visit: impl FnOnce(f32) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_f32 = Some(Box::new(visit));
        self
    }

    pub fn f64(mut self, visit: impl FnOnce(f64) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_f64 = Some(Box::new(visit));
        self
    }

    pub fn char(mut self, visit: impl FnOnce(char) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_char = Some(Box::new(visit));
        self
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

    pub fn bytes(mut self, visit: impl FnOnce(&[u8]) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_bytes = Some(Box::new(visit));
        self
    }

    pub fn borrowed_bytes(
        mut self,
        visit: impl FnOnce(&'de [u8]) -> Result<Value, Error> + 'closure,
    ) -> Self {
        self.visit_borrowed_bytes = Some(Box::new(visit));
        self
    }

    pub fn byte_buf(
        mut self,
        visit: impl FnOnce(Vec<u8>) -> Result<Value, Error> + 'closure,
    ) -> Self {
        self.visit_byte_buf = Some(Box::new(visit));
        self
    }

    pub fn unit(mut self, visit: impl FnOnce() -> Result<Value, Error> + 'closure) -> Self {
        self.visit_unit = Some(Box::new(visit));
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
