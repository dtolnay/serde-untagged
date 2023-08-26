use crate::any::ErasedValue;
use crate::error::Error;
use crate::seed::ErasedDeserializeSeed;
use serde::de::{Deserialize, DeserializeSeed, MapAccess};

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

pub struct Map<'access, 'de> {
    erased: Box<dyn ErasedMapAccess<'de> + 'access>,
}

impl<'access, 'de> Map<'access, 'de> {
    pub(crate) fn new<A>(map: A) -> Self
    where
        A: MapAccess<'de> + 'access,
    {
        Map {
            erased: Box::new(map),
        }
    }

    pub fn deserialize<T>(self) -> Result<T, Error>
    where
        T: Deserialize<'de>,
    {
        T::deserialize(serde::de::value::MapAccessDeserializer::new(self))
    }
}

impl<'access, 'de> MapAccess<'de> for Map<'access, 'de> {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.erased
            .erased_next_key_seed(&mut Some(seed))
            .map(|erased_value| match erased_value {
                Some(value) => Some(unsafe { ErasedValue::take::<T::Value>(value) }),
                None => None,
            })
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.erased
            .erased_next_value_seed(&mut Some(seed))
            .map(|erased_value| unsafe { ErasedValue::take::<T::Value>(erased_value) })
    }

    fn size_hint(&self) -> Option<usize> {
        self.erased.erased_size_hint()
    }
}

impl<'de, Access> ErasedMapAccess<'de> for Access
where
    Access: MapAccess<'de>,
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