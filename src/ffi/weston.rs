// Weston compositor FFI bindings

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// Re-export wl_listener from IVI bindings
pub use super::bindings::wl_listener;

// Include generated Weston bindings
include!(concat!(env!("OUT_DIR"), "/weston_bindings.rs"));

// IVI Layout API name constant
pub const IVI_LAYOUT_API_NAME: &[u8] = b"ivi_layout_api_v1\0";

// Callback type for destroy listeners
pub type wl_notify_func_t =
    unsafe extern "C" fn(listener: *mut wl_listener, data: *mut libc::c_void);

// Manually declare weston_compositor_add_destroy_listener_once
// This function is part of libweston but not in plugin-registry.h
extern "C" {
    /// Add a destroy listener to the compositor that will be called only once
    ///
    /// # Safety
    /// - compositor must be a valid weston_compositor pointer
    /// - listener must be a valid wl_listener pointer that remains valid
    /// - destroy_handler will be called when the compositor is destroyed
    pub fn weston_compositor_add_destroy_listener_once(
        compositor: *mut weston_compositor,
        listener: *mut wl_listener,
        destroy_handler: unsafe extern "C" fn(*mut wl_listener, *mut libc::c_void),
    ) -> bool;
}
