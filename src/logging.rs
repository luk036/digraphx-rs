//! Logging module for digraphx-rs.
//!
//! This module provides optional logging capabilities via `env_logger`.
//! It is only available when the `std` feature is enabled.
//!
//! ## Usage
//!
//! ```rust
//! use digraphx_rs::logging::init_logger;
//!
//! fn main() {
//!     init_logger();
//!     log::info!("Application started");
//! }
//! ```
//!
//! Or with custom filter:
//!
//! ```rust
//! use digraphx_rs::logging::init_logger_with_filter;
//!
//! fn main() {
//!     init_logger_with_filter("debug");
//!     log::debug!("Debug message");
//! }
//! ```
//!
//! ## Environment Variables
//!
//! - `RUST_LOG`: Controls log level (debug, info, warn, error)
//! - `RUST_LOG_STYLE`: Controls colored output
//!
//! Example:
//! ```bash
//! RUST_LOG=debug cargo run --features std
//! ```

#[cfg(feature = "std")]
use log::LevelFilter;

#[cfg(feature = "std")]
use std::sync::OnceLock;

#[cfg(feature = "std")]
static LOGGER_INITIALIZED: OnceLock<()> = OnceLock::new();

/// Initialize the logger with the default filter.
///
/// Reads the log level from the `RUST_LOG` environment variable.
/// If not set, defaults to `info` level.
///
/// # Panics
///
/// Panics if the logger has already been initialized.
#[cfg(feature = "std")]
pub fn init_logger() {
    LOGGER_INITIALIZED.get_or_init(|| ());
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();
}

/// Initialize the logger with a custom filter string.
///
/// The filter string follows `env_logger`'s format:
/// - `debug` - Debug and above
/// - `info` - Info and above
/// - `warn` - Warnings and above
/// - `error` - Errors only
///
/// # Panics
///
/// Panics if the logger has already been initialized.
#[cfg(feature = "std")]
pub fn init_logger_with_filter(filter: &str) {
    LOGGER_INITIALIZED.get_or_init(|| ());
    env_logger::Builder::from_default_env()
        .filter_level(filter.parse().unwrap_or(LevelFilter::Info))
        .init();
}

/// Try to initialize the logger without panicking.
///
/// Reads the log level from the `RUST_LOG` environment variable.
/// If not set, defaults to `info` level.
///
/// # Errors
///
/// Returns an error if the logger has already been initialized.
#[cfg(feature = "std")]
pub fn try_init_logger() -> Result<(), log::SetLoggerError> {
    LOGGER_INITIALIZED.get_or_init(|| ());
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .try_init()
}

/// Try to initialize the logger with a custom filter without panicking.
///
/// # Errors
///
/// Returns an error if the logger has already been initialized.
#[cfg(feature = "std")]
pub fn try_init_logger_with_filter(filter: &str) -> Result<(), log::SetLoggerError> {
    LOGGER_INITIALIZED.get_or_init(|| ());
    env_logger::Builder::from_default_env()
        .filter_level(filter.parse().unwrap_or(LevelFilter::Info))
        .try_init()
}

/// Check if the logger has been initialized.
///
/// # Returns
///
/// `true` if the logger is active, `false` otherwise.
#[cfg(feature = "std")]
pub fn is_logger_initialized() -> bool {
    LOGGER_INITIALIZED.get().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_init_logger() {
        let _ = try_init_logger();
    }

    #[test]
    fn test_try_init_logger_with_filter() {
        let _ = try_init_logger_with_filter("debug");
    }

    #[test]
    fn test_is_logger_initialized() {
        let _ = try_init_logger();
        let initialized = is_logger_initialized();
        assert!(initialized);
    }
}
