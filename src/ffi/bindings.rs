// Generated IVI bindings from ivi-layout-export.h

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use jlogger_tracing::jdebug;
use std::ffi::CStr;

include!(concat!(env!("OUT_DIR"), "/ivi_bindings.rs"));

// override IVI_SUCCEEDED to i32 instead of u32
pub const IVI_SUCCEEDED: i32 = 0;

/// Get the IVI layout API from the Weston compositor
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
/// The caller must ensure that the compositor pointer is valid.
///
/// # Arguments
/// * `compositor` - Pointer to the Weston compositor
///
/// # Returns
/// Pointer to the IVI layout interface, or null pointer if:
/// - The compositor pointer is null
/// - The API name is invalid
/// - The Weston plugin API retrieval fails
pub fn ivi_layout_get_api(
    compositor: *mut super::weston_compositor,
) -> *const ivi_layout_interface {
    // Validate compositor pointer
    if compositor.is_null() {
        jdebug!("ivi_layout_get_api: compositor pointer is null");
        return std::ptr::null();
    }

    unsafe {
        // Create API name string

        jdebug!(
            "ivi_layout_get_api: requesting API '{}', interface size: {} bytes",
            CStr::from_bytes_with_nul(IVI_LAYOUT_API_NAME)
                .unwrap()
                .to_str()
                .unwrap(),
            std::mem::size_of::<ivi_layout_interface>()
        );

        // Request the IVI layout API from Weston
        let api_ptr = super::weston_plugin_api_get(
            compositor,
            IVI_LAYOUT_API_NAME.as_ptr() as *const libc::c_char,
            std::mem::size_of::<ivi_layout_interface>() as libc::size_t,
        );

        if api_ptr.is_null() {
            jdebug!(
                "ivi_layout_get_api: weston_plugin_api_get returned null - \
                 IVI shell may not be loaded or API version mismatch"
            );
            return std::ptr::null();
        }

        jdebug!("ivi_layout_get_api: successfully retrieved IVI layout API");
        api_ptr as *const ivi_layout_interface
    }
}
