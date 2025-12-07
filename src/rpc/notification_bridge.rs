// Notification bridge - converts internal notifications to RPC format

use crate::controller::notifications::{
    FocusChangeNotification, GeometryChangeNotification, LayerOpacityChangeNotification,
    LayerVisibilityChangeNotification, Notification, NotificationData, OpacityChangeNotification,
    OrientationChangeNotification, VisibilityChangeNotification, ZOrderChangeNotification,
};
use crate::controller::state::Orientation;
use crate::controller::subscriptions::SubscriptionManager;
use crate::rpc::protocol::{EventType, RpcNotification};
use serde_json::json;
use std::sync::{Arc, Mutex};

/// Bridges internal notifications to RPC client delivery
pub struct NotificationBridge {
    subscription_manager: Arc<Mutex<SubscriptionManager>>,
}

impl NotificationBridge {
    /// Create a new notification bridge
    pub fn new(subscription_manager: Arc<Mutex<SubscriptionManager>>) -> Self {
        Self {
            subscription_manager,
        }
    }

    /// Convert internal Notification to RPC format
    /// Returns (EventType, RpcNotification) for queueing
    fn convert_notification(&self, notification: &Notification) -> (EventType, RpcNotification) {
        let (event_type, params) = match &notification.data {
            // Surface events
            NotificationData::SurfaceCreated { surface_id } => (
                EventType::SurfaceCreated,
                json!({
                    "event_type": "SurfaceCreated",
                    "surface_id": surface_id
                }),
            ),

            NotificationData::SurfaceDestroyed { surface_id } => (
                EventType::SurfaceDestroyed,
                json!({
                    "event_type": "SurfaceDestroyed",
                    "surface_id": surface_id
                }),
            ),

            NotificationData::GeometryChange(GeometryChangeNotification {
                surface_id,
                old_position,
                new_position,
                old_size,
                new_size,
            }) => (
                EventType::GeometryChanged,
                json!({
                    "event_type": "GeometryChanged",
                    "surface_id": surface_id,
                    "old_position": {"x": old_position.0, "y": old_position.1},
                    "new_position": {"x": new_position.0, "y": new_position.1},
                    "old_size": {"width": old_size.0, "height": old_size.1},
                    "new_size": {"width": new_size.0, "height": new_size.1}
                }),
            ),

            NotificationData::VisibilityChange(VisibilityChangeNotification {
                surface_id,
                old_visibility,
                new_visibility,
            }) => (
                EventType::VisibilityChanged,
                json!({
                    "event_type": "VisibilityChanged",
                    "surface_id": surface_id,
                    "old_visibility": old_visibility,
                    "new_visibility": new_visibility
                }),
            ),

            NotificationData::OpacityChange(OpacityChangeNotification {
                surface_id,
                old_opacity,
                new_opacity,
            }) => (
                EventType::OpacityChanged,
                json!({
                    "event_type": "OpacityChanged",
                    "surface_id": surface_id,
                    "old_opacity": old_opacity,
                    "new_opacity": new_opacity
                }),
            ),

            NotificationData::OrientationChange(OrientationChangeNotification {
                surface_id,
                old_orientation,
                new_orientation,
            }) => (
                EventType::OrientationChanged,
                json!({
                    "event_type": "OrientationChanged",
                    "surface_id": surface_id,
                    "old_orientation": orientation_to_string(*old_orientation),
                    "new_orientation": orientation_to_string(*new_orientation)
                }),
            ),

            NotificationData::ZOrderChange(ZOrderChangeNotification {
                surface_id,
                old_z_order,
                new_z_order,
            }) => (
                EventType::ZOrderChanged,
                json!({
                    "event_type": "ZOrderChanged",
                    "surface_id": surface_id,
                    "old_z_order": old_z_order,
                    "new_z_order": new_z_order
                }),
            ),

            NotificationData::FocusChange(FocusChangeNotification {
                old_focused_surface,
                new_focused_surface,
            }) => (
                EventType::FocusChanged,
                json!({
                    "event_type": "FocusChanged",
                    "old_focused_surface": old_focused_surface,
                    "new_focused_surface": new_focused_surface
                }),
            ),

            // Layer events
            NotificationData::LayerCreated { layer_id } => (
                EventType::LayerCreated,
                json!({
                    "event_type": "LayerCreated",
                    "layer_id": layer_id
                }),
            ),

            NotificationData::LayerDestroyed { layer_id } => (
                EventType::LayerDestroyed,
                json!({
                    "event_type": "LayerDestroyed",
                    "layer_id": layer_id
                }),
            ),

            NotificationData::LayerVisibilityChange(LayerVisibilityChangeNotification {
                layer_id,
                old_visibility,
                new_visibility,
            }) => (
                EventType::LayerVisibilityChanged,
                json!({
                    "event_type": "LayerVisibilityChanged",
                    "layer_id": layer_id,
                    "old_visibility": old_visibility,
                    "new_visibility": new_visibility
                }),
            ),

            NotificationData::LayerOpacityChange(LayerOpacityChangeNotification {
                layer_id,
                old_opacity,
                new_opacity,
            }) => (
                EventType::LayerOpacityChanged,
                json!({
                    "event_type": "LayerOpacityChanged",
                    "layer_id": layer_id,
                    "old_opacity": old_opacity,
                    "new_opacity": new_opacity
                }),
            ),
        };

        let rpc_notification = RpcNotification {
            method: "notification".to_string(),
            params,
        };

        (event_type, rpc_notification)
    }

    /// Handle a notification from the NotificationManager
    /// Converts it to RPC format and queues it to the SubscriptionManager
    pub fn handle_notification(&self, notification: &Notification) {
        let (event_type, rpc_notification) = self.convert_notification(notification);

        self.subscription_manager
            .lock()
            .unwrap()
            .queue_notification(event_type, rpc_notification);
    }
}

/// Helper function to convert Orientation to string representation
fn orientation_to_string(orientation: Orientation) -> &'static str {
    match orientation {
        Orientation::Normal => "Normal",
        Orientation::Rotate90 => "Rotate90",
        Orientation::Rotate180 => "Rotate180",
        Orientation::Rotate270 => "Rotate270",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::notifications::NotificationType;
    use crate::controller::state::Orientation;

    #[test]
    fn test_convert_surface_created() {
        let subscription_manager = Arc::new(Mutex::new(SubscriptionManager::new()));
        let bridge = NotificationBridge::new(subscription_manager);

        let notification = Notification {
            notification_type: NotificationType::SurfaceCreated,
            data: NotificationData::SurfaceCreated { surface_id: 1000 },
        };

        let (event_type, rpc_notification) = bridge.convert_notification(&notification);

        assert_eq!(event_type, EventType::SurfaceCreated);
        assert_eq!(rpc_notification.method, "notification");

        let params = rpc_notification.params.as_object().unwrap();
        assert_eq!(
            params.get("event_type").unwrap().as_str().unwrap(),
            "SurfaceCreated"
        );
        assert_eq!(params.get("surface_id").unwrap().as_u64().unwrap(), 1000);
    }

    #[test]
    fn test_convert_geometry_change() {
        let subscription_manager = Arc::new(Mutex::new(SubscriptionManager::new()));
        let bridge = NotificationBridge::new(subscription_manager);

        let notification = Notification {
            notification_type: NotificationType::GeometryChanged,
            data: NotificationData::GeometryChange(GeometryChangeNotification {
                surface_id: 1000,
                old_position: (0, 0),
                new_position: (100, 100),
                old_size: (1920, 1080),
                new_size: (1280, 720),
            }),
        };

        let (event_type, rpc_notification) = bridge.convert_notification(&notification);

        assert_eq!(event_type, EventType::GeometryChanged);
        assert_eq!(rpc_notification.method, "notification");

        let params = rpc_notification.params.as_object().unwrap();
        assert_eq!(
            params.get("event_type").unwrap().as_str().unwrap(),
            "GeometryChanged"
        );
        assert_eq!(params.get("surface_id").unwrap().as_u64().unwrap(), 1000);
    }

    #[test]
    fn test_convert_focus_change() {
        let subscription_manager = Arc::new(Mutex::new(SubscriptionManager::new()));
        let bridge = NotificationBridge::new(subscription_manager);

        let notification = Notification {
            notification_type: NotificationType::FocusChanged,
            data: NotificationData::FocusChange(FocusChangeNotification {
                old_focused_surface: Some(1000),
                new_focused_surface: Some(2000),
            }),
        };

        let (event_type, rpc_notification) = bridge.convert_notification(&notification);

        assert_eq!(event_type, EventType::FocusChanged);
        let params = rpc_notification.params.as_object().unwrap();
        assert_eq!(
            params.get("old_focused_surface").unwrap().as_u64().unwrap(),
            1000
        );
        assert_eq!(
            params.get("new_focused_surface").unwrap().as_u64().unwrap(),
            2000
        );
    }

    #[test]
    fn test_convert_layer_created() {
        let subscription_manager = Arc::new(Mutex::new(SubscriptionManager::new()));
        let bridge = NotificationBridge::new(subscription_manager);

        let notification = Notification {
            notification_type: NotificationType::LayerCreated,
            data: NotificationData::LayerCreated { layer_id: 5000 },
        };

        let (event_type, rpc_notification) = bridge.convert_notification(&notification);

        assert_eq!(event_type, EventType::LayerCreated);
        let params = rpc_notification.params.as_object().unwrap();
        assert_eq!(
            params.get("event_type").unwrap().as_str().unwrap(),
            "LayerCreated"
        );
        assert_eq!(params.get("layer_id").unwrap().as_u64().unwrap(), 5000);
    }

    #[test]
    fn test_handle_notification_queues_to_manager() {
        let subscription_manager = Arc::new(Mutex::new(SubscriptionManager::new()));
        let bridge = NotificationBridge::new(Arc::clone(&subscription_manager));

        // Subscribe client 1 to SurfaceCreated events
        subscription_manager
            .lock()
            .unwrap()
            .subscribe(1, vec![EventType::SurfaceCreated])
            .unwrap();

        // Handle a surface created notification
        let notification = Notification {
            notification_type: NotificationType::SurfaceCreated,
            data: NotificationData::SurfaceCreated { surface_id: 1000 },
        };

        bridge.handle_notification(&notification);

        // Drain notifications for client 1
        let notifications = subscription_manager.lock().unwrap().drain_notifications(1);

        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].method, "notification");
    }

    #[test]
    fn test_orientation_to_string() {
        assert_eq!(orientation_to_string(Orientation::Normal), "Normal");
        assert_eq!(orientation_to_string(Orientation::Rotate90), "Rotate90");
        assert_eq!(orientation_to_string(Orientation::Rotate180), "Rotate180");
        assert_eq!(orientation_to_string(Orientation::Rotate270), "Rotate270");
    }
}
