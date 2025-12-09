//! C FFI bindings for the IVI client library
//!
//! This module provides C-compatible types and functions for using the IVI client
//! library from C applications.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::client::IviClient;
use crate::error::IviError;
use crate::types::{Layer, Orientation, Position, Size, Surface};

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

/// C-compatible orientation enum
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IviOrientation {
    Normal = 0,
    Rotate90 = 1,
    Rotate180 = 2,
    Rotate270 = 3,
}

impl From<Orientation> for IviOrientation {
    fn from(orientation: Orientation) -> Self {
        match orientation {
            Orientation::Normal => IviOrientation::Normal,
            Orientation::Rotate90 => IviOrientation::Rotate90,
            Orientation::Rotate180 => IviOrientation::Rotate180,
            Orientation::Rotate270 => IviOrientation::Rotate270,
        }
    }
}

impl From<IviOrientation> for Orientation {
    fn from(orientation: IviOrientation) -> Self {
        match orientation {
            IviOrientation::Normal => Orientation::Normal,
            IviOrientation::Rotate90 => Orientation::Rotate90,
            IviOrientation::Rotate180 => Orientation::Rotate180,
            IviOrientation::Rotate270 => Orientation::Rotate270,
        }
    }
}

/// C-compatible position structure
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IviPosition {
    pub x: i32,
    pub y: i32,
}

impl From<Position> for IviPosition {
    fn from(position: Position) -> Self {
        IviPosition {
            x: position.x,
            y: position.y,
        }
    }
}

impl From<IviPosition> for Position {
    fn from(position: IviPosition) -> Self {
        Position {
            x: position.x,
            y: position.y,
        }
    }
}

/// C-compatible size structure
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IviSize {
    pub width: u32,
    pub height: u32,
}

impl From<Size> for IviSize {
    fn from(size: Size) -> Self {
        IviSize {
            width: size.width,
            height: size.height,
        }
    }
}

impl From<IviSize> for Size {
    fn from(size: IviSize) -> Self {
        Size {
            width: size.width,
            height: size.height,
        }
    }
}

/// C-compatible surface structure
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct IviSurface {
    pub id: u32,
    pub orig_size: IviSize,
    pub src_position: IviPosition,
    pub src_size: IviSize,
    pub dest_position: IviPosition,
    pub dest_size: IviSize,
    pub visibility: bool,
    pub opacity: f32,
    pub orientation: IviOrientation,
    pub z_order: i32,
}

impl From<Surface> for IviSurface {
    fn from(surface: Surface) -> Self {
        IviSurface {
            id: surface.id,
            orig_size: surface.orig_size.into(),
            src_position: surface.src_position.into(),
            src_size: surface.src_size.into(),
            dest_position: surface.dest_position.into(),
            dest_size: surface.dest_size.into(),
            visibility: surface.visibility,
            opacity: surface.opacity,
            orientation: surface.orientation.into(),
            z_order: surface.z_order,
        }
    }
}

impl From<IviSurface> for Surface {
    fn from(surface: IviSurface) -> Self {
        Surface {
            id: surface.id,
            orig_size: surface.orig_size.into(),
            src_position: surface.src_position.into(),
            src_size: surface.src_size.into(),
            dest_position: surface.dest_position.into(),
            dest_size: surface.dest_size.into(),
            visibility: surface.visibility,
            opacity: surface.opacity,
            orientation: surface.orientation.into(),
            z_order: surface.z_order,
        }
    }
}

/// C-compatible layer structure
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct IviLayer {
    pub id: u32,
    pub visibility: bool,
    pub opacity: f32,
}

impl From<Layer> for IviLayer {
    fn from(layer: Layer) -> Self {
        IviLayer {
            id: layer.id,
            visibility: layer.visibility,
            opacity: layer.opacity,
        }
    }
}

impl From<IviLayer> for Layer {
    fn from(layer: IviLayer) -> Self {
        Layer {
            id: layer.id,
            visibility: layer.visibility,
            opacity: layer.opacity,
        }
    }
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

/// Helper function to convert IviError to IviErrorCode
fn error_to_code(error: &IviError) -> IviErrorCode {
    match error {
        IviError::ConnectionFailed(_) => IviErrorCode::ConnectionFailed,
        IviError::RequestFailed { .. } => IviErrorCode::RequestFailed,
        IviError::SerializationError(_) => IviErrorCode::Serialization,
        IviError::DeserializationError(_) => IviErrorCode::Deserialization,
        IviError::IoError(_) => IviErrorCode::Io,
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
            error_to_code(&err)
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
            error_to_code(&err)
        }
    }
}

/// Set surface position
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_surface_position(
    client: *mut IviClient,
    id: u32,
    x: i32,
    y: i32,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_surface_position(id, x, y) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            error_to_code(&err)
        }
    }
}

/// Set surface size
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_surface_size(
    client: *mut IviClient,
    id: u32,
    width: u32,
    height: u32,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_surface_size(id, width, height) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            error_to_code(&err)
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
            error_to_code(&err)
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
            error_to_code(&err)
        }
    }
}

/// Set surface orientation
///
/// # Safety
///
/// - `client` must be a valid pointer returned from `ivi_client_connect`
/// - `error_buf` must be a valid pointer to a buffer of at least `error_buf_len` bytes, or NULL
#[no_mangle]
pub unsafe extern "C" fn ivi_set_surface_orientation(
    client: *mut IviClient,
    id: u32,
    orientation: IviOrientation,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() {
        return IviErrorCode::InvalidParam;
    }

    let client = &mut *client;

    match client.set_surface_orientation(id, orientation.into()) {
        Ok(_) => IviErrorCode::Ok,
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            error_to_code(&err)
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
            error_to_code(&err)
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
            error_to_code(&err)
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
            error_to_code(&err)
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
            error_to_code(&err)
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
            error_to_code(&err)
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
            error_to_code(&err)
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
            error_to_code(&err)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orientation_conversion() {
        assert_eq!(
            IviOrientation::from(Orientation::Normal),
            IviOrientation::Normal
        );
        assert_eq!(
            IviOrientation::from(Orientation::Rotate90),
            IviOrientation::Rotate90
        );
        assert_eq!(
            IviOrientation::from(Orientation::Rotate180),
            IviOrientation::Rotate180
        );
        assert_eq!(
            IviOrientation::from(Orientation::Rotate270),
            IviOrientation::Rotate270
        );

        assert_eq!(
            Orientation::from(IviOrientation::Normal),
            Orientation::Normal
        );
        assert_eq!(
            Orientation::from(IviOrientation::Rotate90),
            Orientation::Rotate90
        );
        assert_eq!(
            Orientation::from(IviOrientation::Rotate180),
            Orientation::Rotate180
        );
        assert_eq!(
            Orientation::from(IviOrientation::Rotate270),
            Orientation::Rotate270
        );
    }

    #[test]
    fn test_position_conversion() {
        let pos = Position { x: 100, y: 200 };
        let ivi_pos: IviPosition = pos.into();
        assert_eq!(ivi_pos.x, 100);
        assert_eq!(ivi_pos.y, 200);

        let pos_back: Position = ivi_pos.into();
        assert_eq!(pos, pos_back);
    }

    #[test]
    fn test_size_conversion() {
        let size = Size {
            width: 1920,
            height: 1080,
        };
        let ivi_size: IviSize = size.into();
        assert_eq!(ivi_size.width, 1920);
        assert_eq!(ivi_size.height, 1080);

        let size_back: Size = ivi_size.into();
        assert_eq!(size, size_back);
    }

    #[test]
    fn test_surface_conversion() {
        let surface = Surface {
            id: 1000,
            orig_size: Size {
                width: 1920,
                height: 1080,
            },
            src_position: Position { x: 0, y: 0 },
            src_size: Size {
                width: 1920,
                height: 1080,
            },
            dest_position: Position { x: 100, y: 200 },
            dest_size: Size {
                width: 1920,
                height: 1080,
            },
            visibility: true,
            opacity: 1.0,
            orientation: Orientation::Normal,
            z_order: 0,
        };

        let ivi_surface: IviSurface = surface.clone().into();
        assert_eq!(ivi_surface.id, 1000);
        assert_eq!(ivi_surface.orig_size.width, 1920);
        assert_eq!(ivi_surface.orig_size.height, 1080);
        assert_eq!(ivi_surface.src_position.x, 0);
        assert_eq!(ivi_surface.src_position.y, 0);
        assert_eq!(ivi_surface.src_size.width, 1920);
        assert_eq!(ivi_surface.src_size.height, 1080);
        assert_eq!(ivi_surface.dest_position.x, 100);
        assert_eq!(ivi_surface.dest_position.y, 200);
        assert_eq!(ivi_surface.dest_size.width, 1920);
        assert_eq!(ivi_surface.dest_size.height, 1080);

        assert_eq!(ivi_surface.visibility, true);
        assert_eq!(ivi_surface.opacity, 1.0);
        assert_eq!(ivi_surface.orientation, IviOrientation::Normal);
        assert_eq!(ivi_surface.z_order, 0);

        let surface_back: Surface = ivi_surface.into();
        assert_eq!(surface, surface_back);
    }

    #[test]
    fn test_layer_conversion() {
        let layer = Layer {
            id: 2000,
            visibility: true,
            opacity: 0.75,
        };

        let ivi_layer: IviLayer = layer.clone().into();
        assert_eq!(ivi_layer.id, 2000);
        assert_eq!(ivi_layer.visibility, true);
        assert_eq!(ivi_layer.opacity, 0.75);

        let layer_back: Layer = ivi_layer.into();
        assert_eq!(layer, layer_back);
    }

    #[test]
    fn test_error_to_code() {
        assert_eq!(
            error_to_code(&IviError::ConnectionFailed("test".to_string())),
            IviErrorCode::ConnectionFailed
        );
        assert_eq!(
            error_to_code(&IviError::RequestFailed {
                code: -32000,
                message: "test".to_string()
            }),
            IviErrorCode::RequestFailed
        );
        assert_eq!(
            error_to_code(&IviError::SerializationError("test".to_string())),
            IviErrorCode::Serialization
        );
        assert_eq!(
            error_to_code(&IviError::DeserializationError("test".to_string())),
            IviErrorCode::Deserialization
        );
    }
}
