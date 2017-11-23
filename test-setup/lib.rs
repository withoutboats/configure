extern crate erased_serde;
extern crate serde;
extern crate configure;

use serde::de::Deserializer;
use erased_serde::Error;

use configure::Configure;

const FIELDS: &[&str] = &["first_field", "second_field", "third_field"];

#[derive(Debug, Eq, PartialEq)]
pub struct Configuration {
    pub first_field: u32,
    pub second_field: String,
    pub third_field: Option<Vec<u16>>,
}

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration {
            first_field: 100,
            second_field: String::from("FooBar"),
            third_field: Some(vec![]),
        }
    }
}
impl Configure for Configuration {
    fn generate() -> Result<Configuration, Error> {
        let mut cfg = Configuration::default();
        cfg.regenerate()?;
        Ok(cfg)
    }

    fn regenerate(&mut self) -> Result<(), Error> {
        let deserializer = configure::source::CONFIGURATION.get("test");
        deserializer.deserialize_struct("Configuration", FIELDS, visitors::CfgVisitor {
            default: self,
        })?;
        Ok(())
        
    }
}

mod visitors {
    use std::fmt;

    use serde::de::*;
    use super::Configuration;

    enum Field {
        First,
        Second,
        Third,
    }

    struct FieldVisitor;

    impl<'de> Visitor<'de> for FieldVisitor {
        type Value = Field;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "expecting a field name")
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Field, E> {
            match v {
                "first_field"   => Ok(Field::First),
                "second_field"  => Ok(Field::Second),
                "third_field"   => Ok(Field::Third),
                field           => Err(E::unknown_field(field, super::FIELDS)),
            }
        }
    }

    impl<'de> Deserialize<'de> for Field {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Field, D::Error> {
            deserializer.deserialize_identifier(FieldVisitor)
        }
    }

    pub struct CfgVisitor<'a> {
        pub default: &'a mut Configuration,
    }
    
    impl<'a, 'de> Visitor<'de> for CfgVisitor<'a> {
        type Value = &'a mut Configuration;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "expecting a configuration struct")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where A: MapAccess<'de>,
        {
            while let Some(field) = map.next_key()? {
                match field {
                    Field::First    => {
                        self.default.first_field = map.next_value()?;
                    }
                    Field::Second   => {
                        self.default.second_field = map.next_value()?;
                    }
                    Field::Third    => {
                        self.default.third_field = map.next_value()?;
                    }
                }
            }
            Ok(self.default)
        }
    }
}
