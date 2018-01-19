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
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};

use erased_serde::Deserializer as DynamicDeserializer;

pub use default::DefaultSource;
use null_deserializer::NullDeserializer;

/// The global static holding the active configuration source for this project.
pub static CONFIGURATION: ActiveConfiguration = ActiveConfiguration {
    init: ONCE_INIT,
    is_overriden: ATOMIC_BOOL_INIT,
};

static mut SOURCE: Option<&'static (Fn(&'static str) -> Box<DynamicDeserializer> + Send + Sync + 'static)> = None;

/// A source for configuration.
/// 
/// If an end user wishes to pull configuration from the environment, they must
/// specify their source, which is a type that implements ConfigSource. The
/// source can be specified using the `use_config_from!` macro.
///
/// This crate ships a default source, called DefaultSource, which implements
/// this trait.
pub trait ConfigSource: Send + Sync + 'static {
    /// Initialize this source. This will be called once when the program
    /// begins and then never called again.
    fn init() -> Self;
    /// Prepare a deserializer for a particular package. This will be called
    /// every time we generate configuration for that package.
    fn prepare(&self, package: &'static str) -> Box<DynamicDeserializer<'static>>;
}

/// The active configuration source.
///
/// The only value of this type is the CONFIGURATION global static, which
/// controls what the source of configuration values is. End users can set
/// the configuration source using the `set` method, while libraries which
/// need to be configured can use the `get` method.
pub struct ActiveConfiguration {
    init: Once,
    is_overriden: AtomicBool,
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
    pub fn set<T: ConfigSource>(&'static self, source: T) {
        self.init.call_once(||  {
            self.is_overriden.store(true, Ordering::Relaxed);
            let init = Box::new(move |s| source.prepare(s));
            unsafe { SOURCE = Some(&*Box::into_raw(init)) }
        });
    }

    /// Get the active configuration.
    ///
    /// Libraries which need to construct configuration can use this to get 
    /// the active source of configuration. Normally they would derive
    /// Configure for their config struct, which will call this method.
    pub fn get(&'static self, package: &'static str) -> Box<DynamicDeserializer> {
        self.init.call_once(|| {
            fn null_deserializer(_package: &'static str) -> Box<DynamicDeserializer> {
                Box::new(DynamicDeserializer::erase(NullDeserializer))
            }
            unsafe { SOURCE = Some(&null_deserializer) }
        });
        unsafe { SOURCE.unwrap()(package) }
    }

    /// Returns true if the configuration source is the default source.
    ///
    /// The opposite of `CONFIGURATION.is_overriden()`
    pub fn is_default(&'static self) -> bool {
        !self.is_overriden()
    }

    /// Returns true if the configuration source has been overriden.
    ///
    /// The opposite of `CONFIGURATION.is_default()`
    pub fn is_overriden(&'static self) -> bool {
        self.is_overriden.load(Ordering::Relaxed)
    }
}
