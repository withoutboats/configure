use toml;
use serde::de::{self, Deserializer};

/// An opaque value for deserialization of values.
///
/// Can be constructed from a toml::Value or a String.
pub struct Value {
    toml: toml::Value,
}

impl From<toml::Value> for Value {
    fn from(toml: toml::Value) -> Value {
        Value { toml }
    }
}

impl From<String> for Value {
    fn from(string: String) -> Value {
        let toml = if string.contains(',') {
            toml::Value::Array(string.split(',').map(From::from).collect())
        } else {
            toml::Value::String(string)
        };
        Value { toml }
    }
}

impl<'de> Deserializer<'de> for Value {
    type Error = toml::de::Error;
    
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor<'de>,
    {
        self.toml.deserialize_any(visitor)
    }
    
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where V: de::Visitor<'de>,
    {
        self.toml.deserialize_enum(name, variants, visitor)
    }
    
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor<'de>,
    {
        self.toml.deserialize_option(visitor)
    }
    
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V
    ) -> Result<V::Value, Self::Error>
        where V: de::Visitor<'de>
    {
        self.toml.deserialize_newtype_struct(name, visitor)
    }
    
    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        bytes byte_buf map unit_struct tuple_struct struct
        tuple ignored_any identifier
    }
}
