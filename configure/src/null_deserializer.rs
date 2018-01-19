pub struct NullDeserializer;
use serde::de::{self, Deserializer, MapAccess, Error as ErrorTrait, Visitor};
use erased_serde::Error;

impl<'de> Deserializer<'de> for NullDeserializer {
    type Error = Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>,
    {
        visitor.visit_map(NullMapAccessor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>,
    {
        visitor.visit_map(NullMapAccessor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _fields: usize,
        visitor: V
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>, 
    {
        visitor.visit_map(NullMapAccessor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>, 
    {
        visitor.visit_map(NullMapAccessor)
    }


    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>, 
    {
        visitor.visit_map(NullMapAccessor)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        bytes byte_buf map
        tuple ignored_any identifier enum option 
    }
}

struct NullMapAccessor;

impl<'de> MapAccess<'de> for NullMapAccessor {
    type Error = Error;

    fn next_key_seed<K>(&mut self, _seed: K) -> Result<Option<K::Value>, Self::Error>
        where K: de::DeserializeSeed<'de>,
    {
        Ok(None)
    }

    fn next_value_seed<V>(&mut self, _seed: V) -> Result<V::Value, Self::Error>
        where V: de::DeserializeSeed<'de>, 
    {
        Err(Error::custom("called `next_value` without calling `next_key`"))
    }
}
