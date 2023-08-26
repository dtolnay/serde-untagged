mod error;

use serde::de::{DeserializeSeed, Deserializer, Expected, Visitor};
use std::fmt;
use std::marker::PhantomData;
use std::mem;

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
    visit_seq: Option<Box<dyn FnOnce(SeqAccess) -> Result<Value, Error> + 'closure>>,
    visit_map: Option<Box<dyn FnOnce(MapAccess) -> Result<Value, Error> + 'closure>>,
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
            visit_seq: None,
            visit_map: None,
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

    pub fn seq(mut self, visit: impl FnOnce(SeqAccess) -> Result<Value, Error> + 'closure) -> Self {
        self.visit_seq = Some(Box::new(visit));
        self
    }

    pub fn map(mut self, visit: impl FnOnce(MapAccess) -> Result<Value, Error> + 'closure) -> Self {
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
        formatter.write_str("TODO")
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
        if let Some(visit_i8) = self.visit_i8 {
            visit_i8(v).map_err(error::convert)
        } else {
            self.visit_i64(v as i64)
        }
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_i16) = self.visit_i16 {
            visit_i16(v).map_err(error::convert)
        } else {
            self.visit_i64(v as i64)
        }
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_i32) = self.visit_i32 {
            visit_i32(v).map_err(error::convert)
        } else {
            self.visit_i64(v as i64)
        }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_i64) = self.visit_i64 {
            visit_i64(v).map_err(error::convert)
        } else {
            DefaultVisitor::new(&self).visit_i64(v)
        }
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_i128) = self.visit_i128 {
            visit_i128(v).map_err(error::convert)
        } else {
            DefaultVisitor::new(&self).visit_i128(v)
        }
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_u8) = self.visit_u8 {
            visit_u8(v).map_err(error::convert)
        } else {
            self.visit_u64(v as u64)
        }
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_u16) = self.visit_u16 {
            visit_u16(v).map_err(error::convert)
        } else {
            self.visit_u64(v as u64)
        }
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_u32) = self.visit_u32 {
            visit_u32(v).map_err(error::convert)
        } else {
            self.visit_u64(v as u64)
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_u64) = self.visit_u64 {
            visit_u64(v).map_err(error::convert)
        } else {
            DefaultVisitor::new(&self).visit_u64(v)
        }
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_u128) = self.visit_u128 {
            visit_u128(v).map_err(error::convert)
        } else {
            DefaultVisitor::new(&self).visit_u128(v)
        }
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(visit_f32) = self.visit_f32 {
            visit_f32(v).map_err(error::convert)
        } else {
            self.visit_f64(v as f64)
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
        } else {
            self.visit_str(v.encode_utf8(&mut [0u8; 4]))
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
        A: serde::de::SeqAccess<'de>,
    {
        if let Some(visit_seq) = self.visit_seq {
            let seq = SeqAccess {
                erased: Box::new(seq),
            };
            visit_seq(seq).map_err(error::convert)
        } else {
            DefaultVisitor::new(&self).visit_seq(seq)
        }
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if let Some(visit_map) = self.visit_map {
            let map = MapAccess {
                erased: Box::new(map),
            };
            visit_map(map).map_err(error::convert)
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

struct ErasedValue {
    ptr: *mut (),
    drop: unsafe fn(*mut ()),
}

impl ErasedValue {
    unsafe fn new<T>(value: T) -> Self {
        ErasedValue {
            ptr: Box::into_raw(Box::new(value)).cast(),
            drop: {
                unsafe fn drop<T>(ptr: *mut ()) {
                    let _ = Box::from_raw(ptr.cast::<T>());
                }
                drop::<T>
            },
        }
    }

    unsafe fn take<T>(self) -> T {
        let b = Box::from_raw(self.ptr.cast::<T>());
        mem::forget(self);
        *b
    }
}

impl Drop for ErasedValue {
    fn drop(&mut self) {
        unsafe { (self.drop)(self.ptr) }
    }
}

trait ErasedDeserializeSeed<'de> {
    fn erased_deserialize(
        &mut self,
        deserializer: Box<dyn erased_serde::Deserializer<'de> + '_>,
    ) -> Result<ErasedValue, erased_serde::Error>;
}

impl<'de, Seed> ErasedDeserializeSeed<'de> for Option<Seed>
where
    Seed: serde::de::DeserializeSeed<'de>,
{
    fn erased_deserialize(
        &mut self,
        deserializer: Box<dyn erased_serde::Deserializer<'de> + '_>,
    ) -> Result<ErasedValue, erased_serde::Error> {
        self.take()
            .unwrap()
            .deserialize(deserializer)
            .map(|value| unsafe { ErasedValue::new(value) })
    }
}

impl<'de> DeserializeSeed<'de> for &mut dyn ErasedDeserializeSeed<'de> {
    type Value = ErasedValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deserializer = Box::new(<dyn erased_serde::Deserializer>::erase(deserializer));
        self.erased_deserialize(deserializer)
            .map_err(serde::de::Error::custom)
    }
}

trait ErasedSeqAccess<'de> {
    fn erased_next_element_seed(
        &mut self,
        seed: &mut dyn ErasedDeserializeSeed<'de>,
    ) -> Result<Option<ErasedValue>, Error>;

    fn erased_size_hint(&self) -> Option<usize>;
}

pub struct SeqAccess<'closure, 'de> {
    erased: Box<dyn ErasedSeqAccess<'de> + 'closure>,
}

impl<'closure, 'de> serde::de::SeqAccess<'de> for SeqAccess<'closure, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.erased
            .erased_next_element_seed(&mut Some(seed))
            .map(|erased_value| match erased_value {
                Some(value) => unsafe { ErasedValue::take(value) },
                None => None,
            })
    }

    fn size_hint(&self) -> Option<usize> {
        self.erased.erased_size_hint()
    }
}

impl<'de, Access> ErasedSeqAccess<'de> for Access
where
    Access: serde::de::SeqAccess<'de>,
{
    fn erased_next_element_seed(
        &mut self,
        seed: &mut dyn ErasedDeserializeSeed<'de>,
    ) -> Result<Option<ErasedValue>, Error> {
        self.next_element_seed(seed)
            .map_err(serde::de::Error::custom)
    }

    fn erased_size_hint(&self) -> Option<usize> {
        self.size_hint()
    }
}

trait ErasedMapAccess<'de> {
    fn erased_next_key_seed(
        &mut self,
        seed: &mut dyn ErasedDeserializeSeed<'de>,
    ) -> Result<Option<ErasedValue>, Error>;

    fn erased_next_value_seed(
        &mut self,
        seed: &mut dyn ErasedDeserializeSeed<'de>,
    ) -> Result<ErasedValue, Error>;

    fn erased_size_hint(&self) -> Option<usize>;
}

pub struct MapAccess<'closure, 'de> {
    erased: Box<dyn ErasedMapAccess<'de> + 'closure>,
}

impl<'closure, 'de> serde::de::MapAccess<'de> for MapAccess<'closure, 'de> {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.erased
            .erased_next_key_seed(&mut Some(seed))
            .map(|erased_value| match erased_value {
                Some(value) => unsafe { ErasedValue::take(value) },
                None => None,
            })
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.erased
            .erased_next_value_seed(&mut Some(seed))
            .map(|erased_value| unsafe { ErasedValue::take(erased_value) })
    }

    fn size_hint(&self) -> Option<usize> {
        self.erased.erased_size_hint()
    }
}

impl<'de, Access> ErasedMapAccess<'de> for Access
where
    Access: serde::de::MapAccess<'de>,
{
    fn erased_next_key_seed(
        &mut self,
        seed: &mut dyn ErasedDeserializeSeed<'de>,
    ) -> Result<Option<ErasedValue>, Error> {
        self.next_key_seed(seed).map_err(serde::de::Error::custom)
    }

    fn erased_next_value_seed(
        &mut self,
        seed: &mut dyn ErasedDeserializeSeed<'de>,
    ) -> Result<ErasedValue, Error> {
        self.next_value_seed(seed).map_err(serde::de::Error::custom)
    }

    fn erased_size_hint(&self) -> Option<usize> {
        self.size_hint()
    }
}
