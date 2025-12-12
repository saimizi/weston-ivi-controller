//! C FFI bindings for the IVI client library
//!
//! This module provides C-compatible types and functions for using the IVI client
//! library from C applications.

use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::os::raw::c_char;
use std::ptr;

use crate::client::IviClient;
use crate::error::IviError;

pub type SuffaceId = u32;
pub type LayerId = u32;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IviSize {
    pub width: i32,
    pub height: i32,
}

impl Display for IviSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}@({}, {})", self.width, self.height, self.x, self.y,)
    }
}

/// C-compatible error codes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IviErrorCode {
    /// Operation succeeded
    Ok = 0,
    /// Connection to IVI controller failed
    ConnectionFailed = -1,
    /// JSON-RPC request failed
    RequestFailed = -2,
    /// Serialization error
    Serialization = -3,
    /// Deserialization error
    Deserialization = -4,
    /// I/O error
    Io = -5,
    /// Invalid parameter (null pointer, etc.)
    InvalidParam = -6,
}

impl From<IviError> for IviErrorCode {
    fn from(error: IviError) -> Self {
        match error {
            IviError::ConnectionFailed(_) => IviErrorCode::ConnectionFailed,
            IviError::RequestFailed { .. } => IviErrorCode::RequestFailed,
            IviError::SerializationError(_) => IviErrorCode::Serialization,
            IviError::DeserializationError(_) => IviErrorCode::Deserialization,
            IviError::IoError(_) => IviErrorCode::Io,
        }
    }
}

impl From<IviErrorCode> for IviError {
    fn from(code: IviErrorCode) -> Self {
        match code {
            IviErrorCode::ConnectionFailed => {
                IviError::ConnectionFailed("Connection failed".to_string())
            }
            IviErrorCode::RequestFailed => IviError::RequestFailed {
                code: -1,
                message: "Request failed".to_string(),
            },
            IviErrorCode::Serialization => {
                IviError::SerializationError("Serialization error".to_string())
            }
            IviErrorCode::Deserialization => {
                IviError::DeserializationError("Deserialization error".to_string())
            }
            IviErrorCode::Io => {
                IviError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "I/O error"))
            }
            IviErrorCode::InvalidParam => IviError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid parameter",
            )),
            IviErrorCode::Ok => panic!("Cannot convert Ok to IviError"),
        }
    }
}

/// C-compatible orientation enum
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IviOrientation {
    Normal = 0,
    Rotate90 = 1,
    Rotate180 = 2,
    Rotate270 = 3,
}

impl Display for IviOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let degrees = match self {
            IviOrientation::Normal => 0,
            IviOrientation::Rotate90 => 90,
            IviOrientation::Rotate180 => 180,
            IviOrientation::Rotate270 => 270,
        };
        write!(f, "{} degrees", degrees)
    }
}

/// C-compatible surface structure
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IviSurface {
    pub id: SuffaceId,
    pub orig_size: IviSize,
    pub src_rect: Rectangle,
    pub dest_rect: Rectangle,
    pub visibility: bool,
    pub opacity: f32,
    pub orientation: IviOrientation,
    pub z_order: i32,
}

/// C-compatible layer structure
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IviLayer {
    pub id: LayerId,
    pub visibility: bool,
    pub opacity: f32,
}

/// Helper function to write error message to C buffer
fn write_error_to_buffer(error: &IviError, error_buf: *mut c_char, error_buf_len: usize) {
    if error_buf.is_null() || error_buf_len == 0 {
        return;
    }

    let error_msg = error.to_string();
    let c_string = match CString::new(error_msg) {
        Ok(s) => s,
        Err(_) => return,
    };

    let bytes = c_string.as_bytes_with_nul();
    let copy_len = std::cmp::min(bytes.len(), error_buf_len);

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, error_buf, copy_len);
        if copy_len < error_buf_len {
            *error_buf.add(copy_len - 1) = 0;
        } else {
            *error_buf.add(error_buf_len - 1) = 0;
        }
    }
}

// ============================================================================
// C API Functions - Connection Management
// ============================================================================

/// Connect to the IVI controller
///
/// # Safety
///
/// - `socket_path` must be a valid null-terminated C string or NULL
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
///
/// # Returns
///
/// Returns a pointer to an IviClient on success, or NULL on failure.
/// If NULL is returned, the error message is written to `error_buf`.
#[no_mangle]
pub unsafe extern "C" fn ivi_client_connect(
    socket_path: *const c_char,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> *mut IviClient {
    // Default socket path
    let default_path = "/tmp/weston-ivi-controller.sock";

    // Get socket path from parameter or use default
    let path = if socket_path.is_null() {
        default_path.to_string()
    } else {
        match CStr::from_ptr(socket_path).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                write_error_to_buffer(
                    &IviError::ConnectionFailed("Invalid socket path encoding".to_string()),
                    error_buf,
                    error_buf_len,
                );
                return ptr::null_mut();
            }
        }
    };

    // Attempt to connect
    match IviClient::connect(&path) {
        Ok(client) => Box::into_raw(Box::new(client)),
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            ptr::null_mut()
        }
    }
}

/// Disconnect from the IVI controller and free the client
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`, or NULL
/// - After calling this function, `client` must not be used again
#[no_mangle]
pub unsafe extern "C" fn ivi_client_disconnect(client: *mut IviClient) {
    if !client.is_null() {
        let _ = Box::from_raw(client);
    }
}

// ============================================================================
// C API Functions - Surface Operations
// ============================================================================

/// List all surfaces
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `surfaces` must be a valid pointer to a pointer that will receive the array
/// - `count` must be a valid pointer to receive the array length
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
///
/// # Returns
///
/// Returns IviErrorCode::Ok on success, or an error code on failure.
/// On success, `surfaces` points to an allocated array and `count` contains the array length.
/// The caller must free the array using `ivi_free_surfaces`.
#[no_mangle]
pub unsafe extern "C" fn ivi_list_surfaces(
    client: *mut IviClient,
    surfaces: *mut *mut IviSurface,
    count: *mut usize,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() || surfaces.is_null() || count.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.list_surfaces() {
        Ok(surface_list) => {
            let ivi_surfaces: Vec<IviSurface> =
                surface_list.into_iter().map(|s| s.into()).collect();
            let boxed_slice = ivi_surfaces.into_boxed_slice();
            *count = boxed_slice.len();
            *surfaces = Box::into_raw(boxed_slice) as *mut IviSurface;
            IviErrorCode::Ok
        }
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Get properties of a specific surface
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `surface` must be a valid pointer to an IviSurface structure
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
///
/// # Returns
///
/// Returns IviErrorCode::Ok on success, or an error code on failure.
/// On success, `surface` is populated with the surface properties.
#[no_mangle]
pub unsafe extern "C" fn ivi_get_surface(
    client: *mut IviClient,
    id: u32,
    surface: *mut IviSurface,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() || surface.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.get_surface(id) {
        Ok(surf) => {
            *surface = surf.into();
            IviErrorCode::Ok
        }
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Set surface source rectangle (which part of buffer to display)
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_surface_source_rectangle(
    client: *mut IviClient,
    id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_surface_source_rectangle(id, x, y, width, height) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Set surface destination rectangle (where and how to display on screen)
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_surface_destination_rectangle(
    client: *mut IviClient,
    id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_surface_destination_rectangle(id, x, y, width, height) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Set surface visibility
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_surface_visibility(
    client: *mut IviClient,
    id: u32,
    visible: bool,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_surface_visibility(id, visible) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Set surface opacity
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_surface_opacity(
    client: *mut IviClient,
    id: u32,
    opacity: f32,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_surface_opacity(id, opacity) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Set surface z-order
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_surface_z_order(
    client: *mut IviClient,
    id: u32,
    z_order: i32,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_surface_z_order(id, z_order) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Set surface focus
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_surface_focus(
    client: *mut IviClient,
    id: u32,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_surface_focus(id) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

// ============================================================================
// C API Functions - Layer Operations
// ============================================================================

/// List all layers
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `layers` must be a valid pointer to a pointer that will receive the array
/// - `count` must be a valid pointer to receive the array length
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
///
/// # Returns
///
/// Returns IviErrorCode::Ok on success, or an error code on failure.
/// On success, `layers` points to an allocated array and `count` contains the array length.
/// The caller must free the array using `ivi_free_layers`.
#[no_mangle]
pub unsafe extern "C" fn ivi_list_layers(
    client: *mut IviClient,
    layers: *mut *mut IviLayer,
    count: *mut usize,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() || layers.is_null() || count.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.list_layers() {
        Ok(layer_list) => {
            let ivi_layers: Vec<IviLayer> = layer_list.into_iter().map(|l| l.into()).collect();
            let boxed_slice = ivi_layers.into_boxed_slice();
            *count = boxed_slice.len();
            *layers = Box::into_raw(boxed_slice) as *mut IviLayer;
            IviErrorCode::Ok
        }
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Get properties of a specific layer
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `layer` must be a valid pointer to an IviLayer structure
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
///
/// # Returns
///
/// Returns IviErrorCode::Ok on success, or an error code on failure.
/// On success, `layer` is populated with the layer properties.
#[no_mangle]
pub unsafe extern "C" fn ivi_get_layer(
    client: *mut IviClient,
    id: u32,
    layer: *mut IviLayer,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() || layer.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.get_layer(id) {
        Ok(lyr) => {
            *layer = lyr.into();
            IviErrorCode::Ok
        }
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Set layer visibility
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_layer_visibility(
    client: *mut IviClient,
    id: u32,
    visible: bool,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_layer_visibility(id, visible) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Set layer opacity
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_layer_opacity(
    client: *mut IviClient,
    id: u32,
    opacity: f32,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_layer_opacity(id, opacity) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Set layer source rectangle
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_layer_source_rectangle(
    client: *mut IviClient,
    id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_layer_source_rectangle(id, x, y, width, height) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Set layer destination rectangle
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_layer_destination_rectangle(
    client: *mut IviClient,
    id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_layer_destination_rectangle(id, x, y, width, height) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

// ============================================================================
// C API Functions - Commit Operation
// ============================================================================

/// Commit pending changes
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_commit(
    client: *mut IviClient,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.commit() {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

// ============================================================================
// C API Functions - Memory Management
// ============================================================================

/// Free surfaces array allocated by ivi_list_surfaces
///
/// # Safety
///
/// - `surfaces` must be a pointer returned from `ivi_list_surfaces`, or NULL
/// - `count` must be the same value that was returned by `ivi_list_surfaces`
/// - After calling this function, `surfaces` must not be used again
#[no_mangle]
pub unsafe extern "C" fn ivi_free_surfaces(surfaces: *mut IviSurface, count: usize) {
    if !surfaces.is_null() && count > 0 {
        // Reconstruct the Box with the correct length
        let slice = std::slice::from_raw_parts_mut(surfaces, count);
        let _ = Box::from_raw(slice);
    }
}

/// Free layers array allocated by ivi_list_layers
///
/// # Safety
///
/// - `layers` must be a pointer returned from `ivi_list_layers`, or NULL
/// - `count` must be the same value that was returned by `ivi_list_layers`
/// - After calling this function, `layers` must not be used again
#[no_mangle]
pub unsafe extern "C" fn ivi_free_layers(layers: *mut IviLayer, count: usize) {
    if !layers.is_null() && count > 0 {
        // Reconstruct the Box with the correct length
        let slice = std::slice::from_raw_parts_mut(layers, count);
        let _ = Box::from_raw(slice);
    }
}
