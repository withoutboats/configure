//! Configuration management.
//!
//! This crate is intended to automatically bridge the gap from
//! envionmental configuration into your project. By deriving the
//! `Configure` trait for your configuration, you can get an automatic
//! system for managing your configuration at runtime.
#![deny(missing_docs)]
#[macro_use] extern crate serde;

extern crate failure;
extern crate toml;

pub mod source;
mod value;

use failure::Error;

/// A configuration struct which can be generated from the environment.
///
/// This trait is normally derived using the `derive-configure` crate.
pub trait Configure: Default {
    /// Generate this configuration from the ambient environment.
    fn generate() -> Result<Self, Error>;
    /// Regenerate this configuration.
    fn regenerate(&mut self) -> Result<(), Error>;
}
