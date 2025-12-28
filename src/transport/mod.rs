//! Transport layer implementations
//!
//! This module provides different IPC transport mechanisms for the RPC system.
//! The transport implementation is selected at compile time using feature flags.
//!
//! # Available Transports
//!
//! ## Unix Domain Sockets (default)
//!
//! The default transport uses Unix domain sockets for local IPC.
//!
//! ```toml
//! [dependencies]
//! weston-ivi-controller = "0.1"
//! ```
//!
//! ## IPCON (optional)
//!
//! IPCON is a message-based IPC mechanism with multicast support.
//! Enable it with the `enable-ipcon` feature:
//!
//! ```toml
//! [dependencies]
//! weston-ivi-controller = { version = "0.1", features = ["enable-ipcon"] }
//! ```
//!
//! # Feature Flags
//!
//! - `enable-ipcon`: Use IPCON transport instead of Unix domain sockets
//!
//! **Note**: Only one transport can be active at a time.

#[cfg(not(feature = "enable-ipcon"))]
pub mod unix_socket;

#[cfg(feature = "enable-ipcon")]
pub mod ipcon;

#[cfg(not(feature = "enable-ipcon"))]
pub use unix_socket::UnixSocketTransport;

#[cfg(feature = "enable-ipcon")]
pub use ipcon::IpconTransport;
