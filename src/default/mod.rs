mod env_deserializer;

use std::borrow::Cow;
use std::env::{self, VarError};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::slice;
use std::sync::Arc;

use serde::de::{self, Deserializer, IntoDeserializer, MapAccess, Error as ErrorTrait, Visitor};
use erased_serde::{Error, Deserializer as DynamicDeserializer};
use heck::ShoutySnakeCase;
use toml;

use self::env_deserializer::EnvDeserializer;

#[derive(Clone)]
pub struct DefaultSource {
    toml: Option<Arc<toml::Value>>,
}

impl DefaultSource {
    pub(crate) fn init() -> DefaultSource {
        DefaultSource {
            toml: DefaultSource::toml().map(Arc::new),
        }
    }

    #[cfg(test)]
    pub fn test(toml: Option<toml::Value>) -> DefaultSource {
        DefaultSource {
            toml: toml.map(Arc::new),
        }
    }

    fn toml() -> Option<toml::Value> {
        let path = match env::var_os("CARGO_MANIFEST_DIR") {
            Some(string)    => {
                let dir: PathBuf = string.into();
                dir.join("Cargo.toml")
            }
            None            => return None,
        };

        let mut file = match File::open(path) {
            Ok(file)    => file,
            Err(_)      => return None,
        };

        let mut string = String::new();
        let _ = file.read_to_string(&mut string);
        let manifest: toml::Value = match toml::from_str(&string) {
            Ok(toml)    => toml,
            Err(_)      => return None,
        };
        manifest.get("package")
                .and_then(|package| package.get("metadata"))
                .map(|metadata| metadata.clone())
    }

    pub fn prepare(&self, package: &'static str) -> Box<DynamicDeserializer<'static>> {
        let deserializer = DefaultDeserializer {
            source: self.clone(),
            package: package,
        };
        Box::new(DynamicDeserializer::erase(deserializer)) as Box<DynamicDeserializer>
    }
}

struct DefaultDeserializer {
    source: DefaultSource,
    package: &'static str,
}

impl<'de> Deserializer<'de> for DefaultDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>,
    {
        Err(Error::custom("The default configuration deserializer only supports /
                           deserializing structs."))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>,
    {
        visitor.visit_map(MapAccessor {
            deserializer: self,
            fields: fields.iter(),
            next_val: None,
        })
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V
    ) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>, 
    {
        self.deserialize_struct(_name, &[], visitor)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        bytes byte_buf map tuple_struct newtype_struct
        tuple ignored_any identifier enum option 
    }
}

struct MapAccessor {
    deserializer: DefaultDeserializer,
    fields: slice::Iter<'static, &'static str>,
    next_val: Option<Either>,
}

enum Either {
    Env(String),
    Toml(toml::Value),
}

impl<'de> MapAccess<'de> for MapAccessor {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where K: de::DeserializeSeed<'de>,
    {
        while let Some(field) = self.fields.next() {
            let var_name = format!("{}_{}", self.deserializer.package, field)
                                .to_shouty_snake_case();
            match env::var(&var_name) {
                Ok(env_var)                     => {
                    self.next_val = Some(Either::Env(env_var));
                }
                Err(VarError::NotPresent)       => {
                    let toml = self.deserializer.source.toml.as_ref()
                        .and_then(|toml| toml.get(self.deserializer.package))
                        .and_then(|package| package.get(field));

                    match toml {
                        Some(toml)  => {
                            self.next_val = Some(Either::Toml(toml.clone()));
                        }
                        // If there is neither an env var nor a toml value,
                        // this field is not set. Skip it.
                        None        => continue,
                    }
                }
                Err(VarError::NotUnicode(_))    => {
                    return Err(Error::custom(format!("`{}` is not valid unicode", var_name)));
                }
            }

            let key = seed.deserialize(field.into_deserializer())?;
            return Ok(Some(key));
        }

        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where V: de::DeserializeSeed<'de>, 
    {
        match self.next_val.take() {
            Some(Either::Env(env))      => {
                seed.deserialize(EnvDeserializer(Cow::Owned(env)))
            }
            Some(Either::Toml(toml))    => {
                seed.deserialize(toml).map_err(|e| Error::custom(e.to_string()))
            }
            None                        => {
                Err(Error::custom("called `next_value` without calling `next_key`"))
            }
        }
    }
}
