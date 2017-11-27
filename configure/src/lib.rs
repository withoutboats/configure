//! Configuration management.
//!
//! This crate is intended to automatically bridge the gap from
//! envionmental configuration into your project. By deriving the
//! `Configure` trait for your configuration, you can get an automatic
//! system for managing your configuration at runtime.
#![deny(missing_docs)]
#[macro_use] extern crate serde;
extern crate erased_serde;
extern crate heck;
extern crate toml;

#[cfg(test)]
#[macro_use] extern crate serde_derive;

pub mod source;
mod default;

pub use serde::de::DeserializeOwned;
pub use erased_serde::Error as DeserializeError;

/// A configuration struct which can be generated from the environment.
///
/// This trait is normally derived using the `configure_derive` crate.
///
/// ```rust,ignore
/// #[derive(Configure)]
/// pub struct Config {
///     /// This can be set through the LIBNAME_HOST variable
///     pub host: SocketAddr,
///     /// This can be set through the LIBNAME_THREADS variable
///     pub threads: usize,
/// }
///
/// // To generate your configuration from the environment:
/// let cfg = Config::generate()?;
/// ```
pub trait Configure: Default + DeserializeOwned {
    /// Generate this configuration from the ambient environment.
    fn generate() -> Result<Self, DeserializeError>;

    /// Regenerate this configuration.
    fn regenerate(&mut self) -> Result<(), DeserializeError> {
        *self = Self::generate()?;
        Ok(())
    }
}
