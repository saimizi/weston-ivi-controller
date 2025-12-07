// FFI module - C-compatible interface for Weston integration

pub mod bindings;
pub mod weston;

// Re-export commonly used types
pub use bindings::*;
pub use weston::{
    weston_compositor, weston_compositor_add_destroy_listener_once, weston_plugin_api_get,
    wl_listener, wl_notify_func_t,
};
