use crate::any::ErasedValue;
use crate::error::Error;
use crate::seed::ErasedDeserializeSeed;
use serde::de::{DeserializeSeed, SeqAccess};

trait ErasedSeqAccess<'de> {
    fn erased_next_element_seed(
        &mut self,
        seed: &mut dyn ErasedDeserializeSeed<'de>,
    ) -> Result<Option<ErasedValue>, Error>;

    fn erased_size_hint(&self) -> Option<usize>;
}

pub struct Seq<'closure, 'de> {
    erased: Box<dyn ErasedSeqAccess<'de> + 'closure>,
}

impl<'closure, 'de> Seq<'closure, 'de> {
    pub(crate) fn new<A>(seq: A) -> Self
    where
        A: SeqAccess<'de> + 'closure,
    {
        Seq {
            erased: Box::new(seq),
        }
    }
}

impl<'closure, 'de> SeqAccess<'de> for Seq<'closure, 'de> {
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
    Access: SeqAccess<'de>,
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
