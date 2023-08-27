//! This crate provides a Serde `Visitor` implementation that is useful for
//! deserializing untagged enums.
//!
//! Untagged enum `Deserialize` impls look like this:
//!
//! ```
//! use serde::de::{Deserialize, Deserializer};
//! use serde_untagged::UntaggedEnumVisitor;
//!
//! # macro_rules! impl_deserialize {
//! #     ($MyType:ty) => {
//! impl<'de> Deserialize<'de> for $MyType {
//!     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//!     where
//!         D: Deserializer<'de>,
//!     {
//!         UntaggedEnumVisitor::new()
//!             /*
//!              *
//!              */
//!             .deserialize(deserializer)
//!     }
//! }
//! #     };
//! # }
//! #
//! # struct MyType;
//! # impl_deserialize!(MyType);
//! ```
//!
//! Inside the `/* ... */`, we list each type that the untagged enum needs to
//! support deserializing from, giving a closure that turns the input into
//! $MyType. The following types are supported:
//!
//! - bool
//! - i8, i16, i32, i64, i128, u8, u16, u32, u64, u128
//! - f32
//! - f64
//! - char
//! - string
//! - borrowed\_str
//! - bytes
//! - borrowed\_bytes
//! - byte\_buf
//! - unit
//! - seq
//! - map
//!
//! # Example: string or struct
//!
//! Cargo's `http.ssl-version` configuration supports deserialization from the
//! following two representations:
//!
//! ```toml
//! [http]
//! ssl-version = "tlsv1.3"
//! ```
//!
//! ```toml
//! [http]
//! ssl-version.min = "tlsv1.2"
//! ssl-version.max = "tlsv1.3"
//! ```
//!
//! ```
//! use serde::de::{Deserialize, Deserializer};
//! use serde_derive::Deserialize;
//! use serde_untagged::UntaggedEnumVisitor;
//!
//! pub enum SslVersionConfig {
//!     Single(String),
//!     Range(SslVersionConfigRange),
//! }
//!
//! impl<'de> Deserialize<'de> for SslVersionConfig {
//!     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//!     where
//!         D: Deserializer<'de>,
//!     {
//!         UntaggedEnumVisitor::new()
//!             .string(|single| Ok(SslVersionConfig::Single(single.to_owned())))
//!             .map(|map| map.deserialize().map(SslVersionConfig::Range))
//!             .deserialize(deserializer)
//!     }
//! }
//!
//! #[derive(Deserialize)]
//! pub struct SslVersionConfigRange {
//!     pub min: Option<String>,
//!     pub max: Option<String>,
//! }
//! ```
//!
//! # Example: unit variant or bool
//!
//! Cargo's LTO setting in profiles supports the 5 values `false`, `true`,
//! `"fat"`, `"thin"`, and `"off"`.
//!
//! ```toml
//! [profile.release]
//! lto = "thin"
//! ```
//!
//! ```
//! use serde::de::{Deserialize, Deserializer, IntoDeserializer};
//! use serde_derive::Deserialize;
//! use serde_untagged::UntaggedEnumVisitor;
//!
//! pub enum LinkTimeOptimization {
//!     Enabled(bool),
//!     Enum(LinkTimeOptimizationString),
//! }
//!
//! impl<'de> Deserialize<'de> for LinkTimeOptimization {
//!     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//!     where
//!         D: Deserializer<'de>,
//!     {
//!         UntaggedEnumVisitor::new()
//!             .bool(|b| Ok(LinkTimeOptimization::Enabled(b)))
//!             .string(|string| {
//!                 let de = string.into_deserializer();
//!                 LinkTimeOptimizationString::deserialize(de).map(LinkTimeOptimization::Enum)
//!             })
//!             .deserialize(deserializer)
//!     }
//! }
//!
//! #[derive(Deserialize)]
//! #[serde(rename = "lowercase")]
//! pub enum LinkTimeOptimizationString {
//!     Fat,
//!     Thin,
//!     Off,
//! }
//! ```
//!
//! Since `lto = true` means the same thing as `lto = "fat"` to Cargo, there are
//! really only 4 distinct options. This type could be implemented alternatively
//! as:
//!
//! ```
//! use serde::de::{Deserialize, Deserializer, Unexpected};
//! use serde_untagged::UntaggedEnumVisitor;
//!
//! pub enum LinkTimeOptimization {
//!     ThinLocal,  // false
//!     Fat,        // true or "fat"
//!     Thin,       // "thin"
//!     Off,        // "off"
//! }
//!
//! impl<'de> Deserialize<'de> for LinkTimeOptimization {
//!     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//!     where
//!         D: Deserializer<'de>,
//!     {
//!         UntaggedEnumVisitor::new()
//!             .bool(|b| match b {
//!                 false => Ok(LinkTimeOptimization::ThinLocal),
//!                 true => Ok(LinkTimeOptimization::Fat),
//!             })
//!             .string(|string| match string {
//!                 "fat" => Ok(LinkTimeOptimization::Fat),
//!                 "thin" => Ok(LinkTimeOptimization::Thin),
//!                 "off" => Ok(LinkTimeOptimization::Off),
//!                 _ => Err(serde::de::Error::invalid_value(
//!                     Unexpected::Str(string),
//!                     &r#""fat" or "thin" or "off""#,
//!                 )),
//!             })
//!             .deserialize(deserializer)
//!     }
//! }
//! ```

#![allow(
    clippy::doc_markdown,
    clippy::enum_glob_use,
    clippy::manual_map,
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]

mod any;
mod error;
mod int;
mod map;
mod seed;
mod seq;

use crate::error::Error;
use crate::map::Map;
use crate::seq::Seq;
use serde::de::{Deserializer, Expected, MapAccess, SeqAccess, Unexpected, Visitor};
use std::fmt::{self, Display};
use std::marker::PhantomData;

pub mod de {
    pub use crate::error::Error;
    pub use crate::map::Map;
    pub use crate::seq::Seq;
}

pub struct UntaggedEnumVisitor<'closure, 'de, Value> {
    expecting: Option<Box<dyn Display + 'closure>>,
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
    visit_seq:
        Option<Box<dyn for<'access> FnOnce(Seq<'access, 'de>) -> Result<Value, Error> + 'closure>>,
    visit_map:
        Option<Box<dyn for<'access> FnOnce(Map<'access, 'de>) -> Result<Value, Error> + 'closure>>,
}

impl<'closure, 'de, Value> UntaggedEnumVisitor<'closure, 'de, Value> {
    pub fn new() -> Self {
        UntaggedEnumVisitor {
            expecting: None,
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
            visit_seq: None,
            visit_map: None,
        }
    }

    pub fn expecting(mut self, expecting: impl Display + 'closure) -> Self {
        self.expecting = Some(Box::new(expecting));
        self
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

    pub fn seq(
        mut self,
        visit: impl for<'access> FnOnce(Seq<'access, 'de>) -> Result<Value, Error> + 'closure,
    ) -> Self {
        self.visit_seq = Some(Box::new(visit));
        self
    }

    pub fn map(
        mut self,
        visit: impl for<'access> FnOnce(Map<'access, 'de>) -> Result<Value, Error> + 'closure,
    ) -> Self {
        self.visit_map = Some(Box::new(visit));
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
        if let Some(expecting) = &self.expecting {
            return expecting.fmt(formatter);
        }

        // "a string or array"
        // "an integer, string, or map"
        let mut message = Expecting::new(formatter);
        if self.visit_bool.is_some() {
            message.push("a", "boolean")?;
        }
        if self.visit_i8.is_some()
            || self.visit_i16.is_some()
            || self.visit_i32.is_some()
            || self.visit_i64.is_some()
            || self.visit_i128.is_some()
            || self.visit_u8.is_some()
            || self.visit_u16.is_some()
            || self.visit_u32.is_some()
            || self.visit_u64.is_some()
            || self.visit_u128.is_some()
        {
            message.push("an", "integer")?;
        }
        if self.visit_f32.is_some() || self.visit_f64.is_some() {
            message.push("a", "float")?;
        }
        if self.visit_char.is_some() {
            message.push("a", "character")?;
        }
        if self.visit_str.is_some() {
            message.push("a", "string")?;
        }
        if self.visit_borrowed_str.is_some() && self.visit_str.is_none() {
            message.push("a", "borrowed string")?;
        }
        if self.visit_bytes.is_some()
            || self.visit_borrowed_bytes.is_some()
            || self.visit_byte_buf.is_some()
        {
            message.push("a", "byte array")?;
        }
        if self.visit_unit.is_some() {
            message.push("", "null")?;
        }
        if self.visit_seq.is_some() {
            message.push("an", "array")?;
        }
        if self.visit_map.is_some() {
            message.push("a", "map")?;
        }
        message.flush()
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_bool) = self.visit_bool {
            visit_bool(v).map_err(error::convert)
        } else {
            DefaultVisitor::new(&self).visit_bool(v)
        }
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use crate::int::IntKind::*;
        self.dispatch_integer(v, [I8, I16, I32, I64, I128, U8, U16, U32, U64, U128])
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use crate::int::IntKind::*;
        self.dispatch_integer(v, [I16, I32, I64, I128, I8, U8, U16, U32, U64, U128])
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use crate::int::IntKind::*;
        self.dispatch_integer(v, [I32, I64, I128, I8, I16, U8, U16, U32, U64, U128])
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use crate::int::IntKind::*;
        self.dispatch_integer(v, [I64, I128, I8, I16, I32, U8, U16, U32, U64, U128])
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use crate::int::IntKind::*;
        self.dispatch_integer(v, [I128, I8, I16, I32, I64, U8, U16, U32, U64, U128])
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use crate::int::IntKind::*;
        self.dispatch_integer(v, [U8, U16, U32, U64, U128, I8, I16, I32, I64, I128])
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use crate::int::IntKind::*;
        self.dispatch_integer(v, [U16, U32, U64, U128, U8, I8, I16, I32, I64, I128])
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use crate::int::IntKind::*;
        self.dispatch_integer(v, [U32, U64, U128, U8, U16, I8, I16, I32, I64, I128])
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use crate::int::IntKind::*;
        self.dispatch_integer(v, [U64, U128, U8, U16, U32, I8, I16, I32, I64, I128])
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use crate::int::IntKind::*;
        self.dispatch_integer(v, [U128, U8, U16, U32, U64, I8, I16, I32, I64, I128])
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_f32) = self.visit_f32 {
            visit_f32(v).map_err(error::convert)
        } else {
            self.visit_f64(f64::from(v))
        }
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_f64) = self.visit_f64 {
            visit_f64(v).map_err(error::convert)
        } else {
            DefaultVisitor::new(&self).visit_f64(v)
        }
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_char) = self.visit_char {
            visit_char(v).map_err(error::convert)
        } else if self.visit_str.is_some() {
            self.visit_str(v.encode_utf8(&mut [0u8; 4]))
        } else {
            Err(E::invalid_type(Unexpected::Char(v), &self))
        }
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

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_bytes) = self.visit_bytes {
            visit_bytes(v).map_err(error::convert)
        } else {
            DefaultVisitor::new(&self).visit_bytes(v)
        }
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_borrowed_bytes) = self.visit_borrowed_bytes {
            visit_borrowed_bytes(v).map_err(error::convert)
        } else {
            self.visit_bytes(v)
        }
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_byte_buf) = self.visit_byte_buf {
            visit_byte_buf(v).map_err(error::convert)
        } else {
            self.visit_bytes(&v)
        }
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_unit) = self.visit_unit {
            visit_unit().map_err(error::convert)
        } else {
            DefaultVisitor::new(&self).visit_unit()
        }
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if let Some(visit_seq) = self.visit_seq {
            visit_seq(Seq::new(seq)).map_err(error::convert)
        } else {
            DefaultVisitor::new(&self).visit_seq(seq)
        }
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        if let Some(visit_map) = self.visit_map {
            visit_map(Map::new(map)).map_err(error::convert)
        } else {
            DefaultVisitor::new(&self).visit_map(map)
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

struct Expecting<'e, 'a> {
    formatter: &'e mut fmt::Formatter<'a>,
    count: usize,
    last: Option<&'e str>,
}

impl<'e, 'a> Expecting<'e, 'a> {
    fn new(formatter: &'e mut fmt::Formatter<'a>) -> Self {
        Expecting {
            formatter,
            count: 0,
            last: None,
        }
    }

    fn push(&mut self, article: &str, item: &'e str) -> fmt::Result {
        self.count += 1;
        if self.count == 1 {
            if !article.is_empty() {
                self.formatter.write_str(article)?;
                self.formatter.write_str(" ")?;
            }
            self.formatter.write_str(item)?;
        } else {
            if let Some(last) = self.last.take() {
                self.formatter.write_str(", ")?;
                self.formatter.write_str(last)?;
            }
            self.last = Some(item);
        }
        Ok(())
    }

    fn flush(&mut self) -> fmt::Result {
        if self.count == 0 {
            self.formatter.write_str("unspecified") // ??
        } else if let Some(last) = self.last.take() {
            self.formatter.write_str(" or ")?;
            self.formatter.write_str(last)
        } else {
            Ok(())
        }
    }
}
