//! IVI Client Library
//!
//! A Rust library with C FFI bindings for interacting with the Weston IVI Controller.
//! This library provides both a safe Rust API and C-compatible FFI bindings for
//! controlling IVI (In-Vehicle Infotainment) surfaces and layers through a JSON-RPC
//! interface over UNIX domain sockets.
//!
//! # Features
//!
//! - **Dual API**: Native Rust API and C-compatible FFI bindings
//! - **Type-safe**: Strongly typed data structures for surfaces, layers, and operations
//! - **Error handling**: Comprehensive error types with detailed messages
//! - **Connection management**: Persistent socket connections with automatic cleanup
//! - **JSON-RPC protocol**: Full implementation of the IVI controller protocol
//!
//! # Quick Start
//!
//! ## Rust API
//!
//! ```no_run
//! use ivi_client::{IviClient, Result};
//!
//! fn main() -> Result<()> {
//!     // Connect to the IVI controller
//!     let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
//!     
//!     // List all surfaces
//!     let surfaces = client.list_surfaces()?;
//!     for surface in &surfaces {
//!         println!("Surface {}: {}x{} at {}",
//!             surface.id,
//!             surface.dest_size.width,
//!             surface.dest_size.height,
//!             surface.dest_position
//!         );
//!     }
//!     
//!     // Modify a surface
//!     if let Some(surface) = surfaces.first() {
//!         client.set_surface_visibility(surface.id, true)?;
//!         client.set_surface_opacity(surface.id, 0.8)?;
//!         client.commit()?;
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## C API
//!
//! ```c
//! #include "ivi_client.h"
//!
//! int main() {
//!     char error_buf[256];
//!     
//!     // Connect to the IVI controller
//!     IviClient* client = ivi_client_connect(
//!         "/tmp/weston-ivi-controller.sock",
//!         error_buf,
//!         sizeof(error_buf)
//!     );
//!     
//!     if (client == NULL) {
//!         fprintf(stderr, "Connection failed: %s\n", error_buf);
//!         return 1;
//!     }
//!     
//!     // List all surfaces
//!     IviSurface* surfaces = NULL;
//!     size_t count = 0;
//!     IviErrorCode result = ivi_list_surfaces(
//!         client,
//!         &surfaces,
//!         &count,
//!         error_buf,
//!         sizeof(error_buf)
//!     );
//!     
//!     if (result == IVI_OK) {
//!         printf("Found %zu surfaces\n", count);
//!         ivi_free_surfaces(surfaces);
//!     }
//!     
//!     ivi_client_disconnect(client);
//!     return 0;
//! }
//! ```
//!
//! # Modules
//!
//! - [`client`] - Main client implementation for connecting and communicating
//! - [`types`] - Data structures for surfaces, layers, and properties
//! - [`error`] - Error types and result aliases
//! - [`protocol`] - JSON-RPC protocol structures
//! - [`ffi`] - C FFI bindings for C language integration
//!
//! # Examples
//!
//! See the `examples/` directory for complete working examples:
//! - `rust_example.rs` - Comprehensive Rust API usage
//! - `c_example.c` - Comprehensive C API usage

// Module declarations
pub mod client;
pub mod error;
pub mod ffi;
pub mod protocol;

// Re-export main types for convenience
pub use client::IviClient;
pub use error::{IviError, Result};
pub use ffi::*;
pub use protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
