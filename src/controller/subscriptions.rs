// Subscription management for event notifications

use crate::rpc::protocol::{EventType, RpcNotification};
use crate::rpc::transport::ClientId;
#[allow(unused)]
use jlogger_tracing::{jdebug, jerror, jinfo, jtrace, jwarn};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

/// Default notification buffer size per client
pub const DEFAULT_BUFFER_SIZE: usize = 100;

/// Per-client subscription state
struct ClientSubscription {
    event_types: HashSet<EventType>,
    event_buffer: VecDeque<RpcNotification>,
    buffer_size: usize,
}

impl ClientSubscription {
    fn new(buffer_size: usize) -> Self {
        Self {
            event_types: HashSet::new(),
            event_buffer: VecDeque::with_capacity(buffer_size),
            buffer_size,
        }
    }

    fn subscribe(&mut self, event_types: Vec<EventType>) {
        for event_type in event_types {
            self.event_types.insert(event_type);
        }
    }

    fn unsubscribe(&mut self, event_types: Vec<EventType>) {
        for event_type in event_types {
            self.event_types.remove(&event_type);
        }
    }

    fn is_subscribed(&self, event_type: &EventType) -> bool {
        self.event_types.contains(event_type)
    }

    fn queue_notification(&mut self, notification: RpcNotification) {
        // If buffer is full, drop oldest (FIFO)
        if self.event_buffer.len() >= self.buffer_size {
            let dropped = self.event_buffer.pop_front();
            if dropped.is_some() {
                jdebug!("Dropped oldest notification due to buffer overflow");
            }
        }
        self.event_buffer.push_back(notification);
    }

    fn drain_notifications(&mut self) -> Vec<RpcNotification> {
        self.event_buffer.drain(..).collect()
    }

    fn get_subscriptions(&self) -> Vec<EventType> {
        self.event_types.iter().copied().collect()
    }
}

/// Manages client subscriptions and event buffering
pub struct SubscriptionManager {
    subscriptions: Arc<Mutex<HashMap<ClientId, ClientSubscription>>>,
    buffer_size: usize,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            buffer_size: DEFAULT_BUFFER_SIZE,
        }
    }

    /// Create a new subscription manager with custom buffer size
    pub fn with_buffer_size(buffer_size: usize) -> Self {
        Self {
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            buffer_size,
        }
    }

    /// Subscribe a client to event types
    pub fn subscribe(
        &self,
        client_id: ClientId,
        event_types: Vec<EventType>,
    ) -> Result<Vec<EventType>, String> {
        let mut subs = self.subscriptions.lock().unwrap();
        let client_sub = subs
            .entry(client_id)
            .or_insert_with(|| ClientSubscription::new(self.buffer_size));

        client_sub.subscribe(event_types.clone());

        jinfo!("Client {} subscribed to {:?}", client_id, event_types);

        Ok(event_types)
    }

    /// Unsubscribe a client from event types
    pub fn unsubscribe(
        &self,
        client_id: ClientId,
        event_types: Vec<EventType>,
    ) -> Result<Vec<EventType>, String> {
        let mut subs = self.subscriptions.lock().unwrap();

        if let Some(client_sub) = subs.get_mut(&client_id) {
            client_sub.unsubscribe(event_types.clone());
            jinfo!("Client {} unsubscribed from {:?}", client_id, event_types);
            Ok(event_types)
        } else {
            Err(format!("Client {} has no subscriptions", client_id))
        }
    }

    /// Get a client's current subscriptions
    pub fn get_subscriptions(&self, client_id: ClientId) -> Vec<EventType> {
        let subs = self.subscriptions.lock().unwrap();
        subs.get(&client_id)
            .map(|client_sub| client_sub.get_subscriptions())
            .unwrap_or_default()
    }

    /// Queue a notification for all subscribed clients
    pub fn queue_notification(&self, event_type: EventType, notification: RpcNotification) {
        let mut subs = self.subscriptions.lock().unwrap();

        let subscribed_clients: Vec<ClientId> = subs
            .iter()
            .filter(|(_, client_sub)| client_sub.is_subscribed(&event_type))
            .map(|(client_id, _)| *client_id)
            .collect();

        for client_id in subscribed_clients {
            if let Some(client_sub) = subs.get_mut(&client_id) {
                client_sub.queue_notification(notification.clone());
            }
        }

        jdebug!(
            "Queued notification for event type {:?} to {} clients",
            event_type,
            subs.len()
        );
    }

    /// Drain all pending notifications for a client
    pub fn drain_notifications(&self, client_id: ClientId) -> Vec<RpcNotification> {
        let mut subs = self.subscriptions.lock().unwrap();
        subs.get_mut(&client_id)
            .map(|client_sub| client_sub.drain_notifications())
            .unwrap_or_default()
    }

    /// Remove a client (called on disconnect)
    pub fn remove_client(&self, client_id: ClientId) {
        let mut subs = self.subscriptions.lock().unwrap();
        if subs.remove(&client_id).is_some() {
            jinfo!(
                "Removed subscriptions for disconnected client {}",
                client_id
            );
        }
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        let subs = self.subscriptions.lock().unwrap();
        subs.len()
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_subscribe() {
        let manager = SubscriptionManager::new();
        let client_id = 1;
        let events = vec![EventType::SurfaceCreated, EventType::SurfaceDestroyed];

        let result = manager.subscribe(client_id, events.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), events);

        let subs = manager.get_subscriptions(client_id);
        assert_eq!(subs.len(), 2);
    }

    #[test]
    fn test_unsubscribe() {
        let manager = SubscriptionManager::new();
        let client_id = 1;

        manager
            .subscribe(
                client_id,
                vec![EventType::SurfaceCreated, EventType::SourceGeometryChanged],
            )
            .unwrap();

        let result = manager.unsubscribe(client_id, vec![EventType::SurfaceCreated]);
        assert!(result.is_ok());

        let subs = manager.get_subscriptions(client_id);
        assert_eq!(subs.len(), 1);
        assert!(subs.contains(&EventType::SourceGeometryChanged));
    }

    #[test]
    fn test_queue_and_drain_notifications() {
        let manager = SubscriptionManager::new();
        let client_id = 1;

        manager
            .subscribe(client_id, vec![EventType::SurfaceCreated])
            .unwrap();

        let notification = RpcNotification::new(
            "notification".to_string(),
            json!({"event_type": "SurfaceCreated", "surface_id": 1000}),
        );

        manager.queue_notification(EventType::SurfaceCreated, notification.clone());

        let drained = manager.drain_notifications(client_id);
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0], notification);

        // Should be empty after drain
        let drained2 = manager.drain_notifications(client_id);
        assert_eq!(drained2.len(), 0);
    }

    #[test]
    fn test_buffer_overflow() {
        let manager = SubscriptionManager::with_buffer_size(2);
        let client_id = 1;

        manager
            .subscribe(client_id, vec![EventType::SurfaceCreated])
            .unwrap();

        // Queue 3 notifications (exceeds buffer size of 2)
        for i in 1..=3 {
            let notification = RpcNotification::new(
                "notification".to_string(),
                json!({"event_type": "SurfaceCreated", "surface_id": i}),
            );
            manager.queue_notification(EventType::SurfaceCreated, notification);
        }

        let drained = manager.drain_notifications(client_id);
        // Should only have the last 2 notifications
        assert_eq!(drained.len(), 2);
    }

    #[test]
    fn test_remove_client() {
        let manager = SubscriptionManager::new();
        let client_id = 1;

        manager
            .subscribe(client_id, vec![EventType::SurfaceCreated])
            .unwrap();

        assert_eq!(manager.subscriber_count(), 1);

        manager.remove_client(client_id);

        assert_eq!(manager.subscriber_count(), 0);
        assert_eq!(manager.get_subscriptions(client_id).len(), 0);
    }

    #[test]
    fn test_only_subscribed_clients_receive_notifications() {
        let manager = SubscriptionManager::new();
        let client1 = 1;
        let client2 = 2;

        manager
            .subscribe(client1, vec![EventType::SurfaceCreated])
            .unwrap();
        manager
            .subscribe(client2, vec![EventType::SourceGeometryChanged])
            .unwrap();

        let notification = RpcNotification::new(
            "notification".to_string(),
            json!({"event_type": "SurfaceCreated"}),
        );

        manager.queue_notification(EventType::SurfaceCreated, notification);

        // Client 1 should receive it
        let drained1 = manager.drain_notifications(client1);
        assert_eq!(drained1.len(), 1);

        // Client 2 should not
        let drained2 = manager.drain_notifications(client2);
        assert_eq!(drained2.len(), 0);
    }
}
