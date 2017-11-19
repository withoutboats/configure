//! Controlling the source of configuration.
//!
//! If you are happy with the default configuration source - pulling from
//! environmental variables and falling back to your Cargo.toml - nothing in
//! this module should be of interest to you.
//!
//! Libraries should **never** try to set the configuration source; only
//! binaries should ever override the default.
use std::env::{self, VarError};
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::{Mutex, Once, ONCE_INIT};

use failure::Error;
use toml;

pub use value::Value;

/// The global static holding the active configuration source for this project.
pub static CONFIGURATION: ActiveConfiguration = ActiveConfiguration {
    _private: (),
};

static mut SOURCE: Option<&'static ConfigSource> = None;

static INIT: Once = ONCE_INIT;

/// The active configuration source.
///
/// The onyl value of this type is the CONFIGURATION global static, which
/// controls what the source of configuration values is. End users can set
/// the configuration source using the `set` method, while libraries which
/// need to be configured can use the `get` method.
pub struct ActiveConfiguration {
    _private: (),
}

impl ActiveConfiguration {
    /// Set the active configuration.
    ///
    /// This can only be called once. If it is called more than once,
    /// subsequent calls have no effect. This should only be called by the
    /// final binary which is using the configuration, it should not be called
    /// by libraries.
    ///
    /// If you set the active configuration, you should do so very early in
    /// your program, preferably as close to the beginning of main as possible.
    /// That way, the configuration source is consistent for every dependency.
    pub fn set(&'static self, source: Box<ConfigSource>) {
        INIT.call_once(|| unsafe { SOURCE = Some(&*Box::into_raw(source)) });
    }

    /// Get the active configuration.
    ///
    /// Libraries which need to construct configuration can use this to get 
    /// the active source of configuration. Normally they would derive
    /// Configure for their config struct, which will call this method.
    pub fn get(&'static self) -> &'static ConfigSource {
        INIT.call_once(|| {
            let source = match DefaultSource::init() {
                Ok(Some(toml))  => Box::new(DefaultSource::Toml(toml)),
                Ok(None)        => Box::new(DefaultSource::NoManifest),
                Err(err)        => {
                    let err = Mutex::new(Some(err));
                    Box::new(DefaultSource::Error(err))
                }
            };
            unsafe { SOURCE = Some(&*Box::into_raw(source)) }
        });
        unsafe { SOURCE.unwrap() }
    }
}

enum DefaultSource {
    Toml(toml::Value),
    Error(Mutex<Option<Error>>),
    NoManifest,
}

impl DefaultSource {
    fn init() -> Result<Option<toml::Value>, Error> {
        let path = match env::var_os("CARGO_MANIFEST_DIR") {
            Some(string)    => {
                let dir: PathBuf = string.into();
                dir.join("Cargo.toml")
            }
            None            => return Ok(None),
        };

        let mut file = match File::open(path) {
            Ok(file)                                                => file,
            Err(ref err) if err.kind() == io::ErrorKind::NotFound   => return Ok(None),
            Err(err)                                                => return Err(err.into()),
        };
        let mut string = String::new();
        file.read_to_string(&mut string)?;
        let manifest: toml::Value = toml::from_str(&string)?;
        let metadata = manifest.get("package")
                        .and_then(|package| package.get("metadata"))
                        .map(|metadata| metadata.clone());
        Ok(metadata)
    }
}

/// A source for configuration values.
///
/// If you are not happy with the default configuration source, you can set
/// your own by defining a ConfigSource type and setting the active
/// configuration using `CONFIGURATION.set()`.
pub trait ConfigSource {
    /// Find a particular configuration value from the environment.
    ///
    /// This receives three arguments, all of which are string literals.
    ///
    /// - The first is the environment variable this value is expected to be set
    ///   under.
    /// - The second is the name of the crate this configuration struct is
    ///   from.
    /// - The third is the name of the field of this configuration value..
    fn find(
        &self,
        env_var: &'static str,
        package: &'static str,
        key: &'static str,
    ) -> Result<Option<Value>, Error>;
}

impl ConfigSource for DefaultSource {
    fn find(&self, env_var: &str, package: &str, key: &str) -> Result<Option<Value>, Error> {
        match env::var(env_var) {
            Ok(string)                  => {
                return Ok(Some(string.into()))
            }
            Err(VarError::NotPresent)   => { }
            Err(err)                    => return Err(err.into()),
        }
        match *self {
            DefaultSource::Toml(ref toml)   => {
                let value = toml.get(package)
                                .and_then(|table| table.get(key))
                                .map(|value| value.clone().into());
                Ok(value)
            }
            DefaultSource::Error(ref err)   => {
                match err.lock() {
                    Ok(mut lock)    => {
                        match lock.take() {
                            Some(err)   => Err(err),
                            None        => Ok(None)
                        }
                    }
                    Err(_)      => Ok(None),
                }
            }
            DefaultSource::NoManifest       => Ok(None),
        }
    }
}
