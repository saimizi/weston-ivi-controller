// FFI module - C-compatible interface for Weston integration

pub mod bindings;
pub mod weston;

// Re-export commonly used types
pub use bindings::*;
pub use weston::{weston_compositor, weston_plugin_api_get, IVI_LAYOUT_API_NAME};
