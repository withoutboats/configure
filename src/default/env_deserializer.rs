use std::borrow::Cow;

use serde::de::*;
use serde::de::{Error as ErrorTrait};
use erased_serde::Error;

pub struct EnvDeserializer<'a>(pub Cow<'a, str>);

impl<'a, 'de> IntoDeserializer<'de, Error> for EnvDeserializer<'a> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self { self }
}

macro_rules! deserialize_number {
    ($($f:ident($t:ty): $v:ident;)*) => {$(
        fn $f<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>,
        {
            let x = self.0.parse::<$t>().map_err(|e| Error::custom(e.to_string()))?;
            visitor.$v(x)
        }
    )*}
}

impl<'a, 'de> Deserializer<'de> for EnvDeserializer<'a> {
    type Error = Error;
    
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>,
    {
        visitor.visit_str(&self.0)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        match &self.0[..] {
            "0" | "false"   | "False"   | "FALSE"   => visitor.visit_bool(false),
            "1" | "true"    | "True"    | "TRUE"    => visitor.visit_bool(true),
            _                                       => {
                Err(Error::invalid_value(Unexpected::Str(&self.0), &visitor))
            }
        }
    }

    deserialize_number! { 
        deserialize_i8(i8):     visit_i8;
        deserialize_i16(i16):   visit_i16;
        deserialize_i32(i32):   visit_i32;
        deserialize_i64(i64):   visit_i64;
        deserialize_u8(u8):     visit_u8;
        deserialize_u16(u16):   visit_u16;
        deserialize_u32(u32):   visit_u32;
        deserialize_u64(u64):   visit_u64;
        deserialize_f32(f32):   visit_f32;
        deserialize_f64(f64):   visit_f64;
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        let mut chars = self.0.chars();
        if let Some(c) = chars.next() {
            if chars.next().is_none() {
                return visitor.visit_char(c)
            }
        }
        Err(Error::invalid_value(Unexpected::Str(&self.0), &visitor))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        visitor.visit_str(&self.0)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        visitor.visit_string(self.0.into_owned())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        if let Some(bytes) = hex(&self.0[..]) {
            visitor.visit_bytes(&bytes[..])
        } else {
            Err(Error::invalid_value(Unexpected::Str(&self.0), &visitor))
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        if let Some(bytes) = hex(&self.0[..]) {
            visitor.visit_byte_buf(bytes)
        } else {
            Err(Error::invalid_value(Unexpected::Str(&self.0), &visitor))
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self, 
        _name: &'static str, 
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self, 
        _name: &'static str, 
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        let seq = self.0.split(',').map(|s| EnvDeserializer(Cow::Borrowed(s)));
        visitor.visit_seq(value::SeqDeserializer::new(seq))
    }

    fn deserialize_tuple<V>(
        self, 
        _len: usize, 
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        let seq = self.0.split(',').map(|s| EnvDeserializer(Cow::Borrowed(s)));
        visitor.visit_seq(value::SeqDeserializer::new(seq))
    }

    fn deserialize_tuple_struct<V>(
        self, 
        _name: &'static str, 
        _len: usize, 
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        let seq = self.0.split(',').map(|s| EnvDeserializer(Cow::Borrowed(s)));
        visitor.visit_seq(value::SeqDeserializer::new(seq))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        Err(Error::invalid_type(Unexpected::Map, &visitor))
    }

    fn deserialize_struct<V>(
        self, 
        _name: &'static str, 
        _fields: &'static [&'static str], 
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        Err(Error::invalid_type(Unexpected::Map, &visitor))
    }

    fn deserialize_enum<V>(
        self, 
        _name: &'static str, 
        variants: &'static [&'static str], 
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        visitor.visit_enum(EnumAccessor {
            env_var: &self.0,
            variants: variants,
        })
    }

    fn deserialize_identifier<V>(
        self, 
        _visitor: V
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        Err(Error::custom("cannot deserialize identifier from env var"))
    }

    fn deserialize_ignored_any<V>(
        self, 
        visitor: V
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        visitor.visit_str(&self.0)
    }
}

struct EnumAccessor<'a> {
    env_var: &'a str,
    variants: &'static [&'static str],
}

impl<'a, 'de> EnumAccess<'de> for EnumAccessor<'a> {
    type Error = Error;
    type Variant = VariantAccessor;

    fn variant_seed<V>(
        self, 
        seed: V
    ) -> Result<(V::Value, Self::Variant), Self::Error>
        where V: DeserializeSeed<'de>
    {
        if let Some(&variant) = self.variants.iter().find(|&&v| v == self.env_var) {
            let value = seed.deserialize(variant.into_deserializer())?;
            Ok((value, VariantAccessor))
        } else {
            Err(Error::unknown_variant(self.env_var, self.variants))
        }
    }
}

struct VariantAccessor;

impl<'de> VariantAccess<'de> for VariantAccessor {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
        where T: DeserializeSeed<'de>
    {
        Err(Error::invalid_type(Unexpected::NewtypeVariant, &"a unit variant"))
    }

    fn tuple_variant<V>(
        self, 
        _len: usize, 
        _visitor: V
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        Err(Error::invalid_type(Unexpected::TupleVariant, &"a unit variant"))
    }

    fn struct_variant<V>(
        self, 
        _fields: &'static [&'static str], 
        _visitor: V
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        Err(Error::invalid_type(Unexpected::StructVariant, &"a unit variant"))
    }
    
}

fn hex(s: &str) -> Option<Vec<u8>> {
    let s = if s.starts_with("0x") { &s[2..] } else { s };

    let mut bytes = Vec::with_capacity(s.len() / 2);

    let mut char_indices = s.char_indices();

    while let Some((init, _)) = char_indices.next() {
        if let Some(_) = char_indices.next() {
            match u8::from_str_radix(&s[init..(init + 2)], 16) {
                Ok(byte)    => bytes.push(byte),
                Err(_)      => return None,
            }
        } else {
            return None
        }
    }

    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deserializer(s: &'static str) -> EnvDeserializer<'static> {
        EnvDeserializer(Cow::Borrowed(s))
    }

    #[test]
    fn test_hex_parser() {
        assert_eq!(hex(""), Some(vec![]));
        assert_eq!(hex("01"), Some(vec![0x1]));
        assert_eq!(hex("ff"), Some(vec![0xff]));
        assert_eq!(hex("01ff70"), Some(vec![0x1, 0xff, 0x70]));
        assert_eq!(hex("0x04"), Some(vec![0x4]));
        assert_eq!(hex("0xdeadbeef"), Some(vec![0xde, 0xad, 0xbe, 0xef]));
        assert_eq!(hex("1"), None);
        assert_eq!(hex("not hexadecimal"), None);
    }

    #[test]
    fn test_enum_accessor() {
        #[derive(Deserialize, Eq, PartialEq, Debug)]
        enum Foo {
            Bar,
            Baz,
        }

        assert_eq!(Foo::deserialize(deserializer("Bar")).unwrap(), Foo::Bar);
        assert_eq!(Foo::deserialize(deserializer("Baz")).unwrap(), Foo::Baz);
        assert!(Foo::deserialize(deserializer("Foo")).is_err());
    }

    #[test]
    fn test_numbers() {
        assert_eq!( i8::deserialize(deserializer("-7")).unwrap(), -7i8);
        assert_eq!(i16::deserialize(deserializer("-300")).unwrap(), -300i16);
        assert_eq!(i32::deserialize(deserializer("-55555")).unwrap(), -55555i32);
        assert_eq!(u64::deserialize(deserializer("1")).unwrap(), 1u64);
        assert_eq!(f32::deserialize(deserializer("0.25")).unwrap(), 0.25f32);
    }

    #[test]
    fn test_strings() {
        assert_eq!(String::deserialize(deserializer("Hello world!")).unwrap(),
                   String::from("Hello world!"))
    }
}
