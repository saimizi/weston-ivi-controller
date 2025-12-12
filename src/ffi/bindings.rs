// Generated IVI bindings from ivi-layout-export.h

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use jlogger_tracing::jdebug;
use serde::{Deserialize, Serialize};
use std::ffi::CStr;
use std::fmt::Display;
use std::sync::Arc;

include!(concat!(env!("OUT_DIR"), "/ivi_bindings.rs"));

pub mod ivi_layer;
pub mod ivi_layout_api;
pub mod ivi_layout_layer_properties_m;
pub mod ivi_layout_surface_properties_m;
pub mod ivi_surface;
pub mod weston_output_m;
pub mod weston_surface_m;

pub enum NotificationMask {
    NoneMask,
    Opacity,
    SourceRect,
    DestRect,
    Dimension,
    Position,
    Orientation,
    Visibility,
    PixelFormat,
    Add,
    Remove,
    Configure,
    All,
}

impl From<ivi_layout_notification_mask> for NotificationMask {
    fn from(value: ivi_layout_notification_mask) -> Self {
        match value {
            ivi_layout_notification_mask_IVI_NOTIFICATION_NONE => NotificationMask::NoneMask,
            ivi_layout_notification_mask_IVI_NOTIFICATION_OPACITY => NotificationMask::Opacity,
            ivi_layout_notification_mask_IVI_NOTIFICATION_SOURCE_RECT => {
                NotificationMask::SourceRect
            }
            ivi_layout_notification_mask_IVI_NOTIFICATION_DEST_RECT => NotificationMask::DestRect,
            ivi_layout_notification_mask_IVI_NOTIFICATION_DIMENSION => NotificationMask::Dimension,
            ivi_layout_notification_mask_IVI_NOTIFICATION_POSITION => NotificationMask::Position,
            ivi_layout_notification_mask_IVI_NOTIFICATION_ORIENTATION => {
                NotificationMask::Orientation
            }
            ivi_layout_notification_mask_IVI_NOTIFICATION_VISIBILITY => {
                NotificationMask::Visibility
            }
            ivi_layout_notification_mask_IVI_NOTIFICATION_PIXELFORMAT => {
                NotificationMask::PixelFormat
            }
            ivi_layout_notification_mask_IVI_NOTIFICATION_ADD => NotificationMask::Add,
            ivi_layout_notification_mask_IVI_NOTIFICATION_REMOVE => NotificationMask::Remove,
            ivi_layout_notification_mask_IVI_NOTIFICATION_CONFIGURE => NotificationMask::Configure,
            ivi_layout_notification_mask_IVI_NOTIFICATION_ALL => NotificationMask::All,
            _ => NotificationMask::NoneMask, // Fallback for unknown values
        }
    }
}

impl From<NotificationMask> for ivi_layout_notification_mask {
    fn from(value: NotificationMask) -> Self {
        match value {
            NotificationMask::NoneMask => ivi_layout_notification_mask_IVI_NOTIFICATION_NONE,
            NotificationMask::Opacity => ivi_layout_notification_mask_IVI_NOTIFICATION_OPACITY,
            NotificationMask::SourceRect => {
                ivi_layout_notification_mask_IVI_NOTIFICATION_SOURCE_RECT
            }
            NotificationMask::DestRect => ivi_layout_notification_mask_IVI_NOTIFICATION_DEST_RECT,
            NotificationMask::Dimension => ivi_layout_notification_mask_IVI_NOTIFICATION_DIMENSION,
            NotificationMask::Position => ivi_layout_notification_mask_IVI_NOTIFICATION_POSITION,
            NotificationMask::Orientation => {
                ivi_layout_notification_mask_IVI_NOTIFICATION_ORIENTATION
            }
            NotificationMask::Visibility => {
                ivi_layout_notification_mask_IVI_NOTIFICATION_VISIBILITY
            }
            NotificationMask::PixelFormat => {
                ivi_layout_notification_mask_IVI_NOTIFICATION_PIXELFORMAT
            }
            NotificationMask::Add => ivi_layout_notification_mask_IVI_NOTIFICATION_ADD,
            NotificationMask::Remove => ivi_layout_notification_mask_IVI_NOTIFICATION_REMOVE,
            NotificationMask::Configure => ivi_layout_notification_mask_IVI_NOTIFICATION_CONFIGURE,
            NotificationMask::All => ivi_layout_notification_mask_IVI_NOTIFICATION_ALL,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

// override IVI_SUCCEEDED to i32 instead of u32
pub const IVI_SUCCEEDED: i32 = 0;

pub fn f32_to_wl_fixed_t(value: f32) -> wl_fixed_t {
    (value * 256.0) as wl_fixed_t
}

pub fn f64_to_wl_fixed_t(value: f32) -> wl_fixed_t {
    (value * 256.0) as wl_fixed_t
}

pub fn wl_fixed_t_to_f32(value: wl_fixed_t) -> f32 {
    (value as f32) / 256.0
}

pub fn wl_fixed_t_to_f64(value: wl_fixed_t) -> f32 {
    (value as f32) / 256.0
}

/// Orientation of a surface
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    Normal,    // 0 degrees
    Rotate90,  // 90 degrees
    Rotate180, // 180 degrees
    Rotate270, // 270 degrees
}

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let degrees = match self {
            Orientation::Normal => "Normal",
            Orientation::Rotate90 => "Rotate90",
            Orientation::Rotate180 => "Rotate180",
            Orientation::Rotate270 => "Rotate270",
        };
        write!(f, "{}", degrees)
    }
}

impl From<wl_output_transform> for Orientation {
    fn from(value: wl_output_transform) -> Self {
        match value {
            0 => Orientation::Normal,
            1 => Orientation::Rotate90,
            2 => Orientation::Rotate180,
            3 => Orientation::Rotate270,
            _ => Orientation::Normal, // Default to normal for unknown values
        }
    }
}

impl From<Orientation> for wl_output_transform {
    fn from(value: Orientation) -> Self {
        match value {
            Orientation::Normal => 0,
            Orientation::Rotate90 => 1,
            Orientation::Rotate180 => 2,
            Orientation::Rotate270 => 3,
        }
    }
}

impl Orientation {
    /// Create Orientation from degrees with validation
    pub fn from_degrees(degrees: i32) -> Result<Self, String> {
        // Validate orientation
        crate::controller::validation::validate_orientation(degrees).map_err(|e| e.to_string())?;

        // Normalize to 0-359 range
        let normalized = ((degrees % 360) + 360) % 360;

        match normalized {
            0 => Ok(Orientation::Normal),
            90 => Ok(Orientation::Rotate90),
            180 => Ok(Orientation::Rotate180),
            270 => Ok(Orientation::Rotate270),
            _ => Ok(Orientation::Normal), // Should not happen after validation
        }
    }

    /// Convert to degrees
    pub fn to_degrees(&self) -> i32 {
        match self {
            Orientation::Normal => 0,
            Orientation::Rotate90 => 90,
            Orientation::Rotate180 => 180,
            Orientation::Rotate270 => 270,
        }
    }
}

pub enum IviLayoutTransitionType {
    NoneTransition,
    ViewDefault,
    ViewDestRectOnly,
    ViewFadeOnly,
    LayerFade,
    LayerMove,
    LayerViewOrder,
    ViewMoveResize,
    ViewResize,
    ViewFade,
    Max,
}

impl From<ivi_layout_transition_type> for IviLayoutTransitionType {
    fn from(value: ivi_layout_transition_type) -> Self {
        match value {
            ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_NONE => {
                IviLayoutTransitionType::NoneTransition
            }
            ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_VIEW_DEFAULT => {
                IviLayoutTransitionType::ViewDefault
            }
            ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_VIEW_DEST_RECT_ONLY => {
                IviLayoutTransitionType::ViewDestRectOnly
            }
            ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_VIEW_FADE_ONLY => {
                IviLayoutTransitionType::ViewFadeOnly
            }
            ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_LAYER_FADE => {
                IviLayoutTransitionType::LayerFade
            }
            ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_LAYER_MOVE => {
                IviLayoutTransitionType::LayerMove
            }
            ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_LAYER_VIEW_ORDER => {
                IviLayoutTransitionType::LayerViewOrder
            }
            ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_VIEW_MOVE_RESIZE => {
                IviLayoutTransitionType::ViewMoveResize
            }
            ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_VIEW_RESIZE => {
                IviLayoutTransitionType::ViewResize
            }
            ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_VIEW_FADE => {
                IviLayoutTransitionType::ViewFade
            }
            ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_MAX => IviLayoutTransitionType::Max,
            _ => IviLayoutTransitionType::Max, // Fallback for unknown values
        }
    }
}

impl From<IviLayoutTransitionType> for ivi_layout_transition_type {
    fn from(value: IviLayoutTransitionType) -> Self {
        match value {
            IviLayoutTransitionType::NoneTransition => {
                ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_NONE
            }
            IviLayoutTransitionType::ViewDefault => {
                ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_VIEW_DEFAULT
            }
            IviLayoutTransitionType::ViewDestRectOnly => {
                ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_VIEW_DEST_RECT_ONLY
            }
            IviLayoutTransitionType::ViewFadeOnly => {
                ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_VIEW_FADE_ONLY
            }
            IviLayoutTransitionType::LayerFade => {
                ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_LAYER_FADE
            }
            IviLayoutTransitionType::LayerMove => {
                ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_LAYER_MOVE
            }
            IviLayoutTransitionType::LayerViewOrder => {
                ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_LAYER_VIEW_ORDER
            }
            IviLayoutTransitionType::ViewMoveResize => {
                ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_VIEW_MOVE_RESIZE
            }
            IviLayoutTransitionType::ViewResize => {
                ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_VIEW_RESIZE
            }
            IviLayoutTransitionType::ViewFade => {
                ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_VIEW_FADE
            }
            IviLayoutTransitionType::Max => ivi_layout_transition_type_IVI_LAYOUT_TRANSITION_MAX,
        }
    }
}

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
fn ivi_layout_get_api(
    compositor: *mut super::weston::weston_compositor,
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
