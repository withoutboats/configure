//! Controlling the source of configuration.
//!
//! A source of configuration is something that implements Deserializer.
//! The configuration for each package will pass the name of that package to
//! the source of configuration to get a deserializer for that package's
//! configuration struct.
//!
//! If you are happy with the default configuration source - pulling from
//! environmental variables and falling back to your Cargo.toml - nothing in
//! this module should be of interest to you.
//!
//! Libraries should **never** try to set the configuration source; only
//! binaries should ever override the default.
use std::sync::{Once, ONCE_INIT};

use serde::Deserializer;
use erased_serde::Deserializer as DynamicDeserializer;

pub use default::DefaultSource;

/// The global static holding the active configuration source for this project.
pub static CONFIGURATION: ActiveConfiguration = ActiveConfiguration {
    _private: (),
};

static mut SOURCE: Option<&'static (Fn(&'static str) -> Box<DynamicDeserializer> + Send + Sync + 'static)> = None;

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
    pub fn set<F, D>(&'static self, initializer: F)
    where
        F: Fn(&'static str) -> D + Send + Sync + 'static,
        D: for<'de> Deserializer<'de> + 'static,
    {
        INIT.call_once(||  {
            let init = Box::new(move |s| {
                let deserializer = initializer(s);
                Box::new(DynamicDeserializer::erase(deserializer)) as Box<DynamicDeserializer>
            });
            unsafe { SOURCE = Some(&*Box::into_raw(init)) }
        });
    }

    /// Get the active configuration.
    ///
    /// Libraries which need to construct configuration can use this to get 
    /// the active source of configuration. Normally they would derive
    /// Configure for their config struct, which will call this method.
    pub fn get(&'static self, package: &'static str) -> Box<DynamicDeserializer> {
        INIT.call_once(|| {
            let source = DefaultSource::init();
            let init = Box::new(move |s| source.prepare(s));
            unsafe { SOURCE = Some(&*Box::into_raw(init)) }
        });
        unsafe { SOURCE.unwrap()(package) }
    }
}
