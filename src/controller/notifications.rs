// Notification system for surface and focus changes

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Type of notification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationType {
    /// Geometry changed (position or size)
    GeometryChanged,
    /// Focus changed (gained or lost focus)
    FocusChanged,
    /// Surface created
    SurfaceCreated,
    /// Surface destroyed
    SurfaceDestroyed,
}

/// Notification data for geometry changes
#[derive(Debug, Clone)]
pub struct GeometryChangeNotification {
    pub surface_id: u32,
    pub old_position: (i32, i32),
    pub new_position: (i32, i32),
    pub old_size: (i32, i32),
    pub new_size: (i32, i32),
}

/// Notification data for focus changes
#[derive(Debug, Clone)]
pub struct FocusChangeNotification {
    pub old_focused_surface: Option<u32>,
    pub new_focused_surface: Option<u32>,
}

/// Notification data
#[derive(Debug, Clone)]
pub enum NotificationData {
    GeometryChange(GeometryChangeNotification),
    FocusChange(FocusChangeNotification),
    SurfaceCreated { surface_id: u32 },
    SurfaceDestroyed { surface_id: u32 },
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
        old_position: (i32, i32),
        new_position: (i32, i32),
        old_size: (i32, i32),
        new_size: (i32, i32),
    ) {
        let notification = Notification {
            notification_type: NotificationType::GeometryChanged,
            data: NotificationData::GeometryChange(GeometryChangeNotification {
                surface_id,
                old_position,
                new_position,
                old_size,
                new_size,
            }),
        };

        tracing::debug!(
            "Geometry change notification for surface {}: pos ({},{}) -> ({},{}), size ({},{}) -> ({},{})",
            surface_id,
            old_position.0, old_position.1,
            new_position.0, new_position.1,
            old_size.0, old_size.1,
            new_size.0, new_size.1
        );

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

        tracing::info!(
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

        tracing::info!("Surface created notification for surface {}", surface_id);

        self.emit(notification);
    }

    /// Emit a surface destroyed notification
    pub fn emit_surface_destroyed(&self, surface_id: u32) {
        let notification = Notification {
            notification_type: NotificationType::SurfaceDestroyed,
            data: NotificationData::SurfaceDestroyed { surface_id },
        };

        tracing::info!("Surface destroyed notification for surface {}", surface_id);

        self.emit(notification);
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}
