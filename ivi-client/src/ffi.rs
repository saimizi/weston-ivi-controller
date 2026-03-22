//! C FFI bindings for the IVI client library
//!
//! This module provides C-compatible types and functions for using the IVI client
//! library from C applications.

use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::sync::Arc;

use crate::client::{IviClient, NotificationCallback, NotificationListener};
use crate::error::IviError;
use crate::protocol::{EventType, Notification};

pub type SurfaceId = u32;
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
            IviErrorCode::Io => IviError::IoError(std::io::Error::other("I/O error")),
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
    Flipped = 4,
    Flipped90 = 5,
    Flipped180 = 6,
    Flipped270 = 7,
}

impl Display for IviOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IviOrientation::Normal => write!(f, "0 degrees"),
            IviOrientation::Rotate90 => write!(f, "90 degrees"),
            IviOrientation::Rotate180 => write!(f, "180 degrees"),
            IviOrientation::Rotate270 => write!(f, "270 degrees"),
            IviOrientation::Flipped => write!(f, "Flipped"),
            IviOrientation::Flipped90 => write!(f, "Flipped 90 degrees"),
            IviOrientation::Flipped180 => write!(f, "Flipped 180 degrees"),
            IviOrientation::Flipped270 => write!(f, "Flipped 270 degrees"),
        }
    }
}

/// C-compatible surface structure
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IviSurface {
    pub id: SurfaceId,
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
    pub src_rect: Rectangle,
    pub dest_rect: Rectangle,
    pub visibility: bool,
    pub opacity: f32,
    pub orientation: IviOrientation,
}

/// C-compatible screen structure
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IviScreen {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub x: f64,
    pub y: f64,
    pub transform: IviOrientation,
    pub enabled: bool,
    pub scale: i32,
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
    remote: *const c_char,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> *mut IviClient {
    let remote = if remote.is_null() {
        None
    } else {
        match CStr::from_ptr(remote).to_str() {
            Ok(s) => Some(s),
            Err(_) => {
                write_error_to_buffer(
                    &IviError::ConnectionFailed("Invalid remote encoding".to_string()),
                    error_buf,
                    error_buf_len,
                );
                return ptr::null_mut();
            }
        }
    };

    match IviClient::new(remote) {
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
            let ivi_surfaces: Vec<IviSurface> = surface_list.into_iter().collect();
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
            *surface = surf;
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

    match client.set_surface_source_rectangle(id, x, y, width, height, true) {
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

    match client.set_surface_destination_rectangle(id, x, y, width, height, true) {
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

    match client.set_surface_visibility(id, visible, true) {
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

    match client.set_surface_opacity(id, opacity, true) {
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

    match client.set_surface_z_order(id, z_order, true) {
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

    match client.set_surface_focus(id, true) {
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
            let ivi_layers: Vec<IviLayer> = layer_list.into_iter().collect();
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
            *layer = lyr;
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

    match client.set_layer_visibility(id, visible, true) {
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

    match client.set_layer_opacity(id, opacity, true) {
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

    match client.set_layer_source_rectangle(id, x, y, width, height, true) {
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

    match client.set_layer_destination_rectangle(id, x, y, width, height, true) {
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

// ============================================================================
// Notification types
// ============================================================================

/// Indicates whether a notification refers to a surface or a layer.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IviObjectType {
    Surface = 0,
    Layer = 1,
}

/// Event type enum for C consumers.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IviEventType {
    SurfaceCreated = 0,
    SurfaceDestroyed = 1,
    SourceGeometryChanged = 2,
    DestinationGeometryChanged = 3,
    VisibilityChanged = 4,
    OpacityChanged = 5,
    OrientationChanged = 6,
    ZOrderChanged = 7,
    FocusChanged = 8,
    LayerCreated = 9,
    LayerDestroyed = 10,
    LayerVisibilityChanged = 11,
    LayerOpacityChanged = 12,
}

impl From<&EventType> for IviEventType {
    fn from(et: &EventType) -> Self {
        match et {
            EventType::SurfaceCreated => IviEventType::SurfaceCreated,
            EventType::SurfaceDestroyed => IviEventType::SurfaceDestroyed,
            EventType::SourceGeometryChanged => IviEventType::SourceGeometryChanged,
            EventType::DestinationGeometryChanged => IviEventType::DestinationGeometryChanged,
            EventType::VisibilityChanged => IviEventType::VisibilityChanged,
            EventType::OpacityChanged => IviEventType::OpacityChanged,
            EventType::OrientationChanged => IviEventType::OrientationChanged,
            EventType::ZOrderChanged => IviEventType::ZOrderChanged,
            EventType::FocusChanged => IviEventType::FocusChanged,
            EventType::LayerCreated => IviEventType::LayerCreated,
            EventType::LayerDestroyed => IviEventType::LayerDestroyed,
            EventType::LayerVisibilityChanged => IviEventType::LayerVisibilityChanged,
            EventType::LayerOpacityChanged => IviEventType::LayerOpacityChanged,
        }
    }
}

impl From<IviEventType> for EventType {
    fn from(et: IviEventType) -> Self {
        match et {
            IviEventType::SurfaceCreated => EventType::SurfaceCreated,
            IviEventType::SurfaceDestroyed => EventType::SurfaceDestroyed,
            IviEventType::SourceGeometryChanged => EventType::SourceGeometryChanged,
            IviEventType::DestinationGeometryChanged => EventType::DestinationGeometryChanged,
            IviEventType::VisibilityChanged => EventType::VisibilityChanged,
            IviEventType::OpacityChanged => EventType::OpacityChanged,
            IviEventType::OrientationChanged => EventType::OrientationChanged,
            IviEventType::ZOrderChanged => EventType::ZOrderChanged,
            IviEventType::FocusChanged => EventType::FocusChanged,
            IviEventType::LayerCreated => EventType::LayerCreated,
            IviEventType::LayerDestroyed => EventType::LayerDestroyed,
            IviEventType::LayerVisibilityChanged => EventType::LayerVisibilityChanged,
            IviEventType::LayerOpacityChanged => EventType::LayerOpacityChanged,
        }
    }
}

/// Visibility change data (old and new state).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IviVisibilityChange {
    pub old_visibility: bool,
    pub new_visibility: bool,
}

/// Opacity change data (old and new value).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IviOpacityChange {
    pub old_opacity: f32,
    pub new_opacity: f32,
}

/// Geometry (rectangle) change data (old and new rectangle).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IviGeometryChange {
    pub old_rect: Rectangle,
    pub new_rect: Rectangle,
}

/// Z-order change data (old and new value).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IviZOrderChange {
    pub old_z_order: i32,
    pub new_z_order: i32,
}

/// Orientation change data (old and new value).
#[repr(C)]
#[derive(Debug, Clone)]
pub struct IviOrientationChange {
    pub old_orientation: IviOrientation,
    pub new_orientation: IviOrientation,
}

impl Default for IviOrientationChange {
    fn default() -> Self {
        Self {
            old_orientation: IviOrientation::Normal,
            new_orientation: IviOrientation::Normal,
        }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self { x: 0, y: 0, width: 0, height: 0 }
    }
}

/// A notification event delivered to C callbacks.
///
/// Only the fields relevant to `event_type` are populated; all others are
/// zero-initialised. Check `object_type` to determine whether `object_id`
/// refers to a surface or a layer.
///
/// For `FocusChanged` events:
///   - `object_type`   = `SURFACE`
///   - `object_id`     = new focused surface ID (0 if none)
///   - `object_old_id` = previous focused surface ID (0 if none)
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct IviNotification {
    pub event_type: IviEventType,
    pub object_type: IviObjectType,
    /// Current object ID (surface or layer). For focus events: new focused surface.
    pub object_id: u32,
    /// Previous object ID. Only meaningful for `FocusChanged` events.
    pub object_old_id: u32,
    pub visibility: IviVisibilityChange,
    pub opacity: IviOpacityChange,
    pub src_geometry: IviGeometryChange,
    pub dest_geometry: IviGeometryChange,
    pub z_order: IviZOrderChange,
    pub orientation: IviOrientationChange,
}

impl Default for IviEventType {
    fn default() -> Self {
        IviEventType::SurfaceCreated
    }
}

impl Default for IviObjectType {
    fn default() -> Self {
        IviObjectType::Surface
    }
}

fn parse_rect(params: &serde_json::Value, key: &str) -> Rectangle {
    let r = &params[key];
    Rectangle {
        x: r["x"].as_i64().unwrap_or(0) as i32,
        y: r["y"].as_i64().unwrap_or(0) as i32,
        width: r["width"].as_i64().unwrap_or(0) as i32,
        height: r["height"].as_i64().unwrap_or(0) as i32,
    }
}

fn parse_orientation(params: &serde_json::Value, key: &str) -> IviOrientation {
    match params[key].as_str().unwrap_or("Normal") {
        "Rotate90" => IviOrientation::Rotate90,
        "Rotate180" => IviOrientation::Rotate180,
        "Rotate270" => IviOrientation::Rotate270,
        "Flipped" => IviOrientation::Flipped,
        "Flipped90" => IviOrientation::Flipped90,
        "Flipped180" => IviOrientation::Flipped180,
        "Flipped270" => IviOrientation::Flipped270,
        _ => IviOrientation::Normal,
    }
}

fn notification_to_ffi(notif: &Notification) -> IviNotification {
    let p = &notif.params;
    let mut result = IviNotification {
        event_type: IviEventType::from(&notif.event_type),
        ..Default::default()
    };

    match &notif.event_type {
        EventType::SurfaceCreated | EventType::SurfaceDestroyed => {
            result.object_type = IviObjectType::Surface;
            result.object_id = p["surface_id"].as_u64().unwrap_or(0) as u32;
        }
        EventType::SourceGeometryChanged => {
            result.object_type = IviObjectType::Surface;
            result.object_id = p["surface_id"].as_u64().unwrap_or(0) as u32;
            result.src_geometry = IviGeometryChange {
                old_rect: parse_rect(p, "old_rect"),
                new_rect: parse_rect(p, "new_rect"),
            };
        }
        EventType::DestinationGeometryChanged => {
            result.object_type = IviObjectType::Surface;
            result.object_id = p["surface_id"].as_u64().unwrap_or(0) as u32;
            result.dest_geometry = IviGeometryChange {
                old_rect: parse_rect(p, "old_rect"),
                new_rect: parse_rect(p, "new_rect"),
            };
        }
        EventType::VisibilityChanged => {
            result.object_type = IviObjectType::Surface;
            result.object_id = p["surface_id"].as_u64().unwrap_or(0) as u32;
            result.visibility = IviVisibilityChange {
                old_visibility: p["old_visibility"].as_bool().unwrap_or(false),
                new_visibility: p["new_visibility"].as_bool().unwrap_or(false),
            };
        }
        EventType::OpacityChanged => {
            result.object_type = IviObjectType::Surface;
            result.object_id = p["surface_id"].as_u64().unwrap_or(0) as u32;
            result.opacity = IviOpacityChange {
                old_opacity: p["old_opacity"].as_f64().unwrap_or(0.0) as f32,
                new_opacity: p["new_opacity"].as_f64().unwrap_or(0.0) as f32,
            };
        }
        EventType::OrientationChanged => {
            result.object_type = IviObjectType::Surface;
            result.object_id = p["surface_id"].as_u64().unwrap_or(0) as u32;
            result.orientation = IviOrientationChange {
                old_orientation: parse_orientation(p, "old_orientation"),
                new_orientation: parse_orientation(p, "new_orientation"),
            };
        }
        EventType::ZOrderChanged => {
            result.object_type = IviObjectType::Surface;
            result.object_id = p["surface_id"].as_u64().unwrap_or(0) as u32;
            result.z_order = IviZOrderChange {
                old_z_order: p["old_z_order"].as_i64().unwrap_or(0) as i32,
                new_z_order: p["new_z_order"].as_i64().unwrap_or(0) as i32,
            };
        }
        EventType::FocusChanged => {
            result.object_type = IviObjectType::Surface;
            result.object_id = p["new_focused_surface"].as_u64().unwrap_or(0) as u32;
            result.object_old_id = p["old_focused_surface"].as_u64().unwrap_or(0) as u32;
        }
        EventType::LayerCreated | EventType::LayerDestroyed => {
            result.object_type = IviObjectType::Layer;
            result.object_id = p["layer_id"].as_u64().unwrap_or(0) as u32;
        }
        EventType::LayerVisibilityChanged => {
            result.object_type = IviObjectType::Layer;
            result.object_id = p["layer_id"].as_u64().unwrap_or(0) as u32;
            result.visibility = IviVisibilityChange {
                old_visibility: p["old_visibility"].as_bool().unwrap_or(false),
                new_visibility: p["new_visibility"].as_bool().unwrap_or(false),
            };
        }
        EventType::LayerOpacityChanged => {
            result.object_type = IviObjectType::Layer;
            result.object_id = p["layer_id"].as_u64().unwrap_or(0) as u32;
            result.opacity = IviOpacityChange {
                old_opacity: p["old_opacity"].as_f64().unwrap_or(0.0) as f32,
                new_opacity: p["new_opacity"].as_f64().unwrap_or(0.0) as f32,
            };
        }
    }

    result
}

/// C callback type for notification events.
pub type IviNotificationCCallback =
    unsafe extern "C" fn(notif: *const IviNotification, user_data: *mut c_void);

// ============================================================================
// C API Functions - Notification Listener
// ============================================================================

/// Create a notification listener with its own connection to the IVI controller.
///
/// # Safety
///
/// - `socket_path` must be a valid null-terminated C string or NULL
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
///
/// # Returns
///
/// Returns a pointer to a `NotificationListener` on success, or NULL on failure.
#[no_mangle]
pub unsafe extern "C" fn ivi_notification_listener_new(
    socket_path: *const c_char,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> *mut NotificationListener {
    let remote = if socket_path.is_null() {
        None
    } else {
        match CStr::from_ptr(socket_path).to_str() {
            Ok(s) => Some(s),
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

    match NotificationListener::new(remote) {
        Ok(listener) => Box::into_raw(Box::new(listener)),
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            ptr::null_mut()
        }
    }
}

/// Stop and free a notification listener.
///
/// # Safety
///
/// - `listener` must be a valid pointer returned from `ivi_notification_listener_new`, or NULL
/// - After calling this function, `listener` must not be used again
#[no_mangle]
pub unsafe extern "C" fn ivi_notification_listener_free(listener: *mut NotificationListener) {
    if !listener.is_null() {
        let _ = Box::from_raw(listener);
    }
}

/// Register a C callback for a specific event type.
///
/// Multiple callbacks per event type are allowed.
///
/// # Safety
///
/// - `listener` must be a valid pointer returned from `ivi_notification_listener_new`
/// - `callback` must remain valid for the lifetime of the listener
/// - `user_data` is passed to the callback as-is; the caller is responsible for its lifetime
#[no_mangle]
pub unsafe extern "C" fn ivi_notification_listener_on(
    listener: *mut NotificationListener,
    event_type: IviEventType,
    callback: IviNotificationCCallback,
    user_data: *mut c_void,
) -> IviErrorCode {
    if listener.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let listener = &mut *listener;
    let user_data_ptr = user_data as usize; // make Send-safe

    let cb: NotificationCallback = Arc::new(move |notif: &Notification| {
        let ffi_notif = notification_to_ffi(notif);
        unsafe { callback(&ffi_notif, user_data_ptr as *mut c_void) };
    });

    listener.on(EventType::from(event_type), move |notif| cb(notif));
    IviErrorCode::Ok
}

/// Register a C catch-all callback invoked for every event type.
///
/// # Safety
///
/// - `listener` must be a valid pointer returned from `ivi_notification_listener_new`
/// - `callback` must remain valid for the lifetime of the listener
/// - `user_data` is passed to the callback as-is; the caller is responsible for its lifetime
#[no_mangle]
pub unsafe extern "C" fn ivi_notification_listener_on_all(
    listener: *mut NotificationListener,
    callback: IviNotificationCCallback,
    user_data: *mut c_void,
) -> IviErrorCode {
    if listener.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let listener = &mut *listener;
    let user_data_ptr = user_data as usize;

    listener.on_all(move |notif: &Notification| {
        let ffi_notif = notification_to_ffi(notif);
        unsafe { callback(&ffi_notif, user_data_ptr as *mut c_void) };
    });
    IviErrorCode::Ok
}

/// Subscribe to the specified event types and start the background reader thread.
///
/// # Safety
///
/// - `listener` must be a valid pointer returned from `ivi_notification_listener_new`
/// - `event_types` must be a valid pointer to an array of `count` `IviEventType` values, or NULL
///   if `count` is 0
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_notification_listener_start(
    listener: *mut NotificationListener,
    event_types: *const IviEventType,
    count: usize,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if listener.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let listener = &mut *listener;

    let types: Vec<EventType> = if event_types.is_null() || count == 0 {
        vec![]
    } else {
        std::slice::from_raw_parts(event_types, count)
            .iter()
            .map(|&et| EventType::from(et))
            .collect()
    };

    match listener.start(&types) {
        Ok(()) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            err.into()
        }
    }
}

/// Stop the background reader thread. Registered callbacks will no longer fire.
///
/// # Safety
///
/// - `listener` must be a valid pointer returned from `ivi_notification_listener_new`
#[no_mangle]
pub unsafe extern "C" fn ivi_notification_listener_stop(listener: *mut NotificationListener) {
    if !listener.is_null() {
        (*listener).stop();
    }
}
