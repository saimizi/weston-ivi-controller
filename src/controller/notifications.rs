// Notification system for surface and focus changes

use crate::ffi::bindings::*;
#[allow(unused)]
use jlogger_tracing::{jdebug, jerror, jinfo, jwarn, JloggerBuilder, LevelFilter};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Type of notification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationType {
    // Surface events
    /// Geometry changed (position or size)
    GeometryChanged,
    /// Focus changed (gained or lost focus)
    FocusChanged,
    /// Surface created
    SurfaceCreated,
    /// Surface destroyed
    SurfaceDestroyed,
    /// Surface visibility changed
    VisibilityChanged,
    /// Surface opacity changed
    OpacityChanged,
    /// Surface orientation changed
    OrientationChanged,
    /// Surface z-order changed
    ZOrderChanged,

    // Layer events
    /// Layer created
    LayerCreated,
    /// Layer destroyed
    LayerDestroyed,
    /// Layer visibility changed
    LayerVisibilityChanged,
    /// Layer opacity changed
    LayerOpacityChanged,
}

/// Notification data for geometry changes
#[derive(Debug, Clone)]
pub struct GeometryChangeNotification {
    pub surface_id: u32,
    pub old_rect: Rectangle,
    pub new_rect: Rectangle,
}

/// Notification data for focus changes
#[derive(Debug, Clone)]
pub struct FocusChangeNotification {
    pub old_focused_surface: Option<u32>,
    pub new_focused_surface: Option<u32>,
}

/// Notification data for visibility changes
#[derive(Debug, Clone)]
pub struct VisibilityChangeNotification {
    pub surface_id: u32,
    pub old_visibility: bool,
    pub new_visibility: bool,
}

/// Notification data for opacity changes
#[derive(Debug, Clone)]
pub struct OpacityChangeNotification {
    pub surface_id: u32,
    pub old_opacity: f32,
    pub new_opacity: f32,
}

/// Notification data for orientation changes
#[derive(Debug, Clone)]
pub struct OrientationChangeNotification {
    pub surface_id: u32,
    pub old_orientation: Orientation,
    pub new_orientation: Orientation,
}

/// Notification data for z-order changes
#[derive(Debug, Clone)]
pub struct ZOrderChangeNotification {
    pub surface_id: u32,
    pub old_z_order: i32,
    pub new_z_order: i32,
}

/// Notification data for layer visibility changes
#[derive(Debug, Clone)]
pub struct LayerVisibilityChangeNotification {
    pub layer_id: u32,
    pub old_visibility: bool,
    pub new_visibility: bool,
}

/// Notification data for layer opacity changes
#[derive(Debug, Clone)]
pub struct LayerOpacityChangeNotification {
    pub layer_id: u32,
    pub old_opacity: f32,
    pub new_opacity: f32,
}

pub enum GeometryType {
    Source,
    Destination,
}

/// Notification data
#[derive(Debug, Clone)]
pub enum NotificationData {
    // Surface notifications
    SourceGeometryChange(GeometryChangeNotification),
    DestinationGeometryChange(GeometryChangeNotification),
    FocusChange(FocusChangeNotification),
    SurfaceCreated { surface_id: u32 },
    SurfaceDestroyed { surface_id: u32 },
    VisibilityChange(VisibilityChangeNotification),
    OpacityChange(OpacityChangeNotification),
    OrientationChange(OrientationChangeNotification),
    ZOrderChange(ZOrderChangeNotification),

    // Layer notifications
    LayerCreated { layer_id: u32 },
    LayerDestroyed { layer_id: u32 },
    LayerVisibilityChange(LayerVisibilityChangeNotification),
    LayerOpacityChange(LayerOpacityChangeNotification),
}

/// A notification event
#[derive(Debug, Clone)]
pub struct Notification {
    pub notification_type: NotificationType,
    pub data: NotificationData,
}

/// Callback function type for notifications
pub type NotificationCallback = Arc<dyn Fn(&Notification) + Send + Sync>;

/// Manages notifications for surface and focus changes
pub struct NotificationManager {
    callbacks: Arc<Mutex<HashMap<NotificationType, Vec<NotificationCallback>>>>,
}

impl NotificationManager {
    /// Create a new notification manager
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a callback for a specific notification type
    pub fn register_callback(
        &mut self,
        notification_type: NotificationType,
        callback: NotificationCallback,
    ) {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks
            .entry(notification_type)
            .or_insert_with(Vec::new)
            .push(callback);
    }

    /// Emit a notification to all registered callbacks
    pub fn emit(&self, notification: Notification) {
        let callbacks = self.callbacks.lock().unwrap();

        if let Some(callback_list) = callbacks.get(&notification.notification_type) {
            for callback in callback_list {
                callback(&notification);
            }
        }
    }

    /// Emit a geometry change notification
    pub fn emit_geometry_change(
        &self,
        surface_id: u32,
        geometry_type: GeometryType,
        old_rect: Rectangle,
        new_rect: Rectangle,
    ) {
        let notification = match geometry_type {
            GeometryType::Source => Notification {
                notification_type: NotificationType::GeometryChanged,
                data: NotificationData::SourceGeometryChange(GeometryChangeNotification {
                    surface_id,
                    old_rect,
                    new_rect,
                }),
            },
            GeometryType::Destination => Notification {
                notification_type: NotificationType::GeometryChanged,
                data: NotificationData::DestinationGeometryChange(GeometryChangeNotification {
                    surface_id,
                    old_rect,
                    new_rect,
                }),
            },
        };

        self.emit(notification);
    }

    /// Emit a focus change notification
    pub fn emit_focus_change(&self, old_focused: Option<u32>, new_focused: Option<u32>) {
        let notification = Notification {
            notification_type: NotificationType::FocusChanged,
            data: NotificationData::FocusChange(FocusChangeNotification {
                old_focused_surface: old_focused,
                new_focused_surface: new_focused,
            }),
        };

        jinfo!(
            "Focus change notification: {:?} -> {:?}",
            old_focused,
            new_focused
        );

        self.emit(notification);
    }

    /// Emit a surface created notification
    pub fn emit_surface_created(&self, surface_id: u32) {
        let notification = Notification {
            notification_type: NotificationType::SurfaceCreated,
            data: NotificationData::SurfaceCreated { surface_id },
        };

        jinfo!("Surface created notification for surface {}", surface_id);

        self.emit(notification);
    }

    /// Emit a surface destroyed notification
    pub fn emit_surface_destroyed(&self, surface_id: u32) {
        let notification = Notification {
            notification_type: NotificationType::SurfaceDestroyed,
            data: NotificationData::SurfaceDestroyed { surface_id },
        };

        jinfo!("Surface destroyed notification for surface {}", surface_id);

        self.emit(notification);
    }

    /// Emit a visibility change notification
    pub fn emit_visibility_change(
        &self,
        surface_id: u32,
        old_visibility: bool,
        new_visibility: bool,
    ) {
        let notification = Notification {
            notification_type: NotificationType::VisibilityChanged,
            data: NotificationData::VisibilityChange(VisibilityChangeNotification {
                surface_id,
                old_visibility,
                new_visibility,
            }),
        };

        jdebug!(
            "Visibility change notification for surface {}: {} -> {}",
            surface_id,
            old_visibility,
            new_visibility
        );

        self.emit(notification);
    }

    /// Emit an opacity change notification
    pub fn emit_opacity_change(&self, surface_id: u32, old_opacity: f32, new_opacity: f32) {
        let notification = Notification {
            notification_type: NotificationType::OpacityChanged,
            data: NotificationData::OpacityChange(OpacityChangeNotification {
                surface_id,
                old_opacity,
                new_opacity,
            }),
        };

        jdebug!(
            "Opacity change notification for surface {}: {} -> {}",
            surface_id,
            old_opacity,
            new_opacity
        );

        self.emit(notification);
    }

    /// Emit an orientation change notification
    pub fn emit_orientation_change(
        &self,
        surface_id: u32,
        old_orientation: Orientation,
        new_orientation: Orientation,
    ) {
        let notification = Notification {
            notification_type: NotificationType::OrientationChanged,
            data: NotificationData::OrientationChange(OrientationChangeNotification {
                surface_id,
                old_orientation,
                new_orientation,
            }),
        };

        jdebug!(
            "Orientation change notification for surface {}: {:?} -> {:?}",
            surface_id,
            old_orientation,
            new_orientation
        );

        self.emit(notification);
    }

    /// Emit a z-order change notification
    pub fn emit_z_order_change(&self, surface_id: u32, old_z_order: i32, new_z_order: i32) {
        let notification = Notification {
            notification_type: NotificationType::ZOrderChanged,
            data: NotificationData::ZOrderChange(ZOrderChangeNotification {
                surface_id,
                old_z_order,
                new_z_order,
            }),
        };

        jdebug!(
            "Z-order change notification for surface {}: {} -> {}",
            surface_id,
            old_z_order,
            new_z_order
        );

        self.emit(notification);
    }

    /// Emit a layer created notification
    pub fn emit_layer_created(&self, layer_id: u32) {
        let notification = Notification {
            notification_type: NotificationType::LayerCreated,
            data: NotificationData::LayerCreated { layer_id },
        };

        jinfo!("Layer created notification for layer {}", layer_id);

        self.emit(notification);
    }

    /// Emit a layer destroyed notification
    pub fn emit_layer_destroyed(&self, layer_id: u32) {
        let notification = Notification {
            notification_type: NotificationType::LayerDestroyed,
            data: NotificationData::LayerDestroyed { layer_id },
        };

        jinfo!("Layer destroyed notification for layer {}", layer_id);

        self.emit(notification);
    }

    /// Emit a layer visibility change notification
    pub fn emit_layer_visibility_change(
        &self,
        layer_id: u32,
        old_visibility: bool,
        new_visibility: bool,
    ) {
        let notification = Notification {
            notification_type: NotificationType::LayerVisibilityChanged,
            data: NotificationData::LayerVisibilityChange(LayerVisibilityChangeNotification {
                layer_id,
                old_visibility,
                new_visibility,
            }),
        };

        jdebug!(
            "Layer visibility change notification for layer {}: {} -> {}",
            layer_id,
            old_visibility,
            new_visibility
        );

        self.emit(notification);
    }

    /// Emit a layer opacity change notification
    pub fn emit_layer_opacity_change(&self, layer_id: u32, old_opacity: f32, new_opacity: f32) {
        let notification = Notification {
            notification_type: NotificationType::LayerOpacityChanged,
            data: NotificationData::LayerOpacityChange(LayerOpacityChangeNotification {
                layer_id,
                old_opacity,
                new_opacity,
            }),
        };

        jdebug!(
            "Layer opacity change notification for layer {}: {} -> {}",
            layer_id,
            old_opacity,
            new_opacity
        );

        self.emit(notification);
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}
