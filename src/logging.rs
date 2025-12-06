//! Logging configuration for the Weston IVI Controller
//!
//! This module provides logging initialization and utilities.

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Initialize the logging system
///
/// This should be called once during plugin initialization.
/// It sets up the tracing subscriber with appropriate formatting.
pub fn init_logging() -> Result<(), String> {
    // Set up environment filter (defaults to INFO level)
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Try to initialize with jlogger-tracing if available
    // Otherwise fall back to stderr logging
    let result = tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .try_init();

    match result {
        Ok(_) => {
            tracing::info!("Logging system initialized");
            Ok(())
        }
        Err(e) => Err(format!("Failed to initialize logging: {}", e)),
    }
}

/// Log an error with context
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

/// Log a warning with context
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

/// Log an info message
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

/// Log a debug message
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

/// Log a trace message
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*)
    };
}
