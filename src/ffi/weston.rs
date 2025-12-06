// Weston compositor FFI bindings

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// Include generated Weston bindings
include!(concat!(env!("OUT_DIR"), "/weston_bindings.rs"));

// IVI Layout API name constant
pub const IVI_LAYOUT_API_NAME: &[u8] = b"ivi_layout_api_v1\0";
