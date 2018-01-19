//! Configuration management.
//!
//! This crate is intended to automatically bridge the gap from
//! envionmental configuration into your project. By deriving the
//! `Configure` trait for your configuration, you can get an automatic
//! system for managing your configuration at runtime.
//! 
//! # Deriving `Configure`
//!
//! This library provides a custom derive which lets you derive the `Configure`
//! trait. It requires that your type implement `Deserialize`.
//! 
//! We recommend that you implement configuration using these steps:
//!
//! 1. Implement `Default` for your type, which provides the default values for
//!    each configuration field.
//! 2. Derive both `Deserialize` and `Configure` for your type. Use the
//!    `#[serde(default)]` attribute to fall back to the default implementation
//!    when the configurable values are not set.
//!
//! For example:
//!
//! ```ignore
//! #[macro_use]
//! extern crate configure;
//! extern crate serde;
//! #[macro_use]
//! extern crate serde_derive;
//!
//! use std::net::SocketAddr;
//! use std::path::PathBuf;
//!
//! #[derive(Deserialize, Configure)]
//! #[serde(default)]
//! pub struct Config {
//!     addr: SocketAddr,
//!     tls_cert: Option<PathBuf>,
//! }
//!
//! impl Default for Config {
//!     fn default() -> Config {
//!         Config {
//!             addr: "127.0.0.1:7878".parse().unwrap(),
//!             tls_cert: None,
//!         }
//!     }
//! }
//! ```
//!
//! With this code, you can call `Config::generate` to pull you configuration
//! from the environment, falling back to these default values if the end user
//! has not set custom configuration for it.
#![deny(missing_docs)]
#[macro_use] extern crate serde;
extern crate erased_serde;
extern crate heck;
extern crate toml;

#[allow(unused_imports)]
#[macro_use] extern crate configure_derive;

#[cfg(test)]
#[macro_use] extern crate serde_derive;

pub mod source;
mod null_deserializer;
mod default;

pub use erased_serde::Error as DeserializeError;

#[doc(hidden)]
pub use configure_derive::*;

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
pub trait Configure: Sized {
    /// Generate this configuration from the ambient environment.
    fn generate() -> Result<Self, DeserializeError>;

    /// Regenerate this configuration.
    fn regenerate(&mut self) -> Result<(), DeserializeError> {
        *self = Self::generate()?;
        Ok(())
    }
}

/// 
#[macro_export]
macro_rules! use_config_from {
    ($source:ty)  => {
        $crate::source::CONFIGURATION.set(<$source as $crate::source::ConfigSource>::init())
    }
}

#[macro_export]
macro_rules! use_default_config {
    ()  => {
        use_config_from!($crate::source::DefaultSource)
    }
}
