// RPC request handler

use super::protocol::{EventType, RpcError, RpcMethod, RpcRequest, RpcResponse};
use super::transport::{ClientId, MessageHandler, Transport, TransportError};
use crate::controller::state::{StateManager, SurfaceState};
use crate::controller::subscriptions::SubscriptionManager;
use crate::controller::validation;
#[allow(unused)]
use jlogger_tracing::{jdebug, jerror, jinfo, jtrace, jwarn, JloggerBuilder, LevelFilter};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Handles RPC requests and generates responses
pub struct RpcHandler {
    state_manager: Arc<Mutex<StateManager>>,
    transport: Arc<Mutex<Option<Box<dyn Transport>>>>,
    subscription_manager: Arc<Mutex<SubscriptionManager>>,
}

impl RpcHandler {
    /// Create a new RPC handler wrapped in Arc for shared ownership
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Arc<Self> {
        Arc::new(Self {
            state_manager,
            transport: Arc::new(Mutex::new(None)),
            subscription_manager: Arc::new(Mutex::new(SubscriptionManager::new())),
        })
    }

    /// Get a reference to the subscription manager (for testing and integration)
    pub fn subscription_manager(&self) -> Arc<Mutex<SubscriptionManager>> {
        Arc::clone(&self.subscription_manager)
    }

    /// Register a transport implementation
    pub fn register_transport(
        self: &Arc<Self>,
        mut transport: Box<dyn Transport>,
    ) -> Result<(), TransportError> {
        // Create a handler for this RPC handler
        let handler = RpcMessageHandler {
            rpc_handler: Arc::clone(self),
        };

        // Register the handler with the transport
        transport.register_handler(Box::new(handler));

        // Store the transport
        let mut transport_lock = self.transport.lock().unwrap();
        *transport_lock = Some(transport);

        Ok(())
    }

    /// Start the registered transport
    pub fn start_transport(&self) -> Result<(), TransportError> {
        let mut transport_lock = self.transport.lock().unwrap();
        if let Some(transport) = transport_lock.as_mut() {
            transport.start()
        } else {
            Err(TransportError::InitError(
                "No transport registered".to_string(),
            ))
        }
    }

    /// Stop the registered transport
    pub fn stop_transport(&self) -> Result<(), TransportError> {
        let mut transport_lock = self.transport.lock().unwrap();
        if let Some(transport) = transport_lock.as_mut() {
            transport.stop()
        } else {
            Ok(())
        }
    }

    /// Start the notification delivery loop in a background thread
    /// This should be called after register_transport() and start_transport()
    pub fn start_notification_delivery(self: &Arc<Self>) {
        let subscription_manager = Arc::clone(&self.subscription_manager);
        let transport = Arc::clone(&self.transport);

        jinfo!("Starting notification delivery loop");

        thread::spawn(move || {
            loop {
                // Small sleep to avoid busy-waiting
                thread::sleep(Duration::from_millis(10));

                // Get connected clients
                let transport_lock = transport.lock().unwrap();
                if let Some(ref t) = *transport_lock {
                    let clients = t.get_connected_clients();
                    drop(transport_lock);

                    // Drain and send notifications for each client
                    for client_id in clients {
                        let notifications = subscription_manager
                            .lock()
                            .unwrap()
                            .drain_notifications(client_id);

                        if notifications.is_empty() {
                            continue;
                        }

                        jtrace!(
                            "Sending {} notifications to client {}",
                            notifications.len(),
                            client_id
                        );

                        for notification in notifications {
                            // Serialize notification to JSON
                            match serde_json::to_vec(&notification) {
                                Ok(json) => {
                                    // Transport handles length-prefix framing
                                    // Send to client
                                    let transport_lock = transport.lock().unwrap();
                                    if let Some(ref t) = *transport_lock {
                                        if let Err(e) = t.send(client_id, &json) {
                                            jwarn!(
                                                "Failed to send notification to client {}: {:?}",
                                                client_id,
                                                e
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    jerror!(
                                        "Failed to serialize notification for client {}: {:?}",
                                        client_id,
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        });

        jinfo!("Notification delivery loop started");
    }

    /// Handle an RPC request
    pub fn handle_request(&self, client_id: ClientId, request: RpcRequest) -> RpcResponse {
        jdebug!(
            "Handling RPC request from client {}: method={}, id={}",
            client_id,
            request.method,
            request.id
        );

        // Parse the method from the request
        let method = match RpcMethod::from_request(&request) {
            Ok(m) => m,
            Err(e) => {
                jwarn!("Invalid RPC method: {}, error: {}", request.method, e);
                return RpcResponse::error(request.id, e);
            }
        };

        // Check if auto_commit is requested (default: false for batching)
        let auto_commit = request
            .params
            .get("auto_commit")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Route to the appropriate handler
        let result = match method {
            RpcMethod::ListSurfaces => self.handle_list_surfaces(),
            RpcMethod::GetSurface { id } => self.handle_get_surface(id),
            RpcMethod::SetPosition { id, x, y } => self.handle_set_position(id, x, y, auto_commit),
            RpcMethod::SetSize { id, width, height } => {
                self.handle_set_size(id, width, height, auto_commit)
            }
            RpcMethod::SetVisibility { id, visible } => {
                self.handle_set_visibility(id, visible, auto_commit)
            }
            RpcMethod::SetOpacity { id, opacity } => {
                self.handle_set_opacity(id, opacity, auto_commit)
            }
            RpcMethod::SetOrientation { id, orientation } => {
                self.handle_set_orientation(id, orientation, auto_commit)
            }
            RpcMethod::SetZOrder { id, z_order } => {
                self.handle_set_z_order(id, z_order, auto_commit)
            }
            RpcMethod::SetFocus { id } => self.handle_set_focus(id, auto_commit),
            RpcMethod::Commit => self.handle_commit(),

            // Subscription methods
            RpcMethod::Subscribe { event_types } => self.handle_subscribe(client_id, event_types),
            RpcMethod::Unsubscribe { event_types } => {
                self.handle_unsubscribe(client_id, event_types)
            }
            RpcMethod::ListSubscriptions => self.handle_list_subscriptions(client_id),

            // Layer methods
            RpcMethod::ListLayers => self.handle_list_layers(),
            RpcMethod::GetLayer { id } => self.handle_get_layer(id),
            RpcMethod::SetLayerVisibility { id, visible } => {
                self.handle_set_layer_visibility(id, visible, auto_commit)
            }
            RpcMethod::SetLayerOpacity { id, opacity } => {
                self.handle_set_layer_opacity(id, opacity, auto_commit)
            }
        };

        // Generate response
        match result {
            Ok(value) => {
                jdebug!("RPC request successful: id={}", request.id);
                RpcResponse::success(request.id, value)
            }
            Err(error) => {
                jerror!("RPC request failed: id={}, error: {}", request.id, error);
                RpcResponse::error(request.id, error)
            }
        }
    }

    /// Handle list_surfaces request
    fn handle_list_surfaces(&self) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();
        let surfaces = state_manager.get_all_surfaces();

        let surface_list: Vec<serde_json::Value> =
            surfaces.iter().map(|s| surface_state_to_json(s)).collect();

        Ok(json!({ "surfaces": surface_list }))
    }

    /// Handle get_surface request
    fn handle_get_surface(&self, id: u32) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();

        match state_manager.get_surface(id) {
            Some(surface) => {
                jdebug!("Retrieved surface {}", id);
                Ok(surface_state_to_json(&surface))
            }
            None => {
                jwarn!("Surface not found: {}", id);
                Err(RpcError::surface_not_found(id))
            }
        }
    }

    /// Handle set_position request
    fn handle_set_position(
        &self,
        id: u32,
        x: i32,
        y: i32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        jdebug!(
            "Setting position for surface {}: ({}, {}) [auto_commit={}]",
            id,
            x,
            y,
            auto_commit
        );

        // Validate position
        validation::validate_position(x, y).map_err(|e| {
            jwarn!("Invalid position for surface {}: {}", id, e);
            RpcError::invalid_params(e.to_string())
        })?;

        let state_manager = self.state_manager.lock().unwrap();

        // Check if surface exists
        if !state_manager.has_surface(id) {
            jwarn!("Surface not found: {}", id);
            return Err(RpcError::surface_not_found(id));
        }

        // Get the IVI API and update the surface
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager); // Release lock before calling IVI API

        let mut surface = ivi_api
            .get_surface_from_id(id)
            .ok_or_else(|| RpcError::internal_error(format!("Failed to get IVI surface {}", id)))?;

        surface
            .set_position(x, y)
            .map_err(|e| RpcError::internal_error(e))?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(e.to_string()))?;

            // Update internal state
            let mut state_manager = self.state_manager.lock().unwrap();
            state_manager.handle_surface_configured(id);
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle set_size request
    fn handle_set_size(
        &self,
        id: u32,
        width: i32,
        height: i32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        jdebug!(
            "Setting size for surface {}: {}x{} [auto_commit={}]",
            id,
            width,
            height,
            auto_commit
        );

        // Validate size
        validation::validate_size(width, height)
            .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let state_manager = self.state_manager.lock().unwrap();

        // Check if surface exists
        if !state_manager.has_surface(id) {
            return Err(RpcError::surface_not_found(id));
        }

        // Get the IVI API and update the surface
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager); // Release lock before calling IVI API

        let mut surface = ivi_api
            .get_surface_from_id(id)
            .ok_or_else(|| RpcError::internal_error(format!("Failed to get IVI surface {}", id)))?;

        surface
            .set_size(width, height)
            .map_err(|e| RpcError::internal_error(e))?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(e.to_string()))?;

            // Update internal state
            let mut state_manager = self.state_manager.lock().unwrap();
            state_manager.handle_surface_configured(id);
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle set_visibility request
    fn handle_set_visibility(
        &self,
        id: u32,
        visible: bool,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();

        // Check if surface exists
        if !state_manager.has_surface(id) {
            return Err(RpcError::surface_not_found(id));
        }

        // Get the IVI API and update the surface
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager); // Release lock before calling IVI API

        let mut surface = ivi_api
            .get_surface_from_id(id)
            .ok_or_else(|| RpcError::internal_error(format!("Failed to get IVI surface {}", id)))?;

        surface.set_visibility(visible);

        // Commit changes only if auto_commit is true
        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(e.to_string()))?;

            // Update internal state
            let mut state_manager = self.state_manager.lock().unwrap();
            state_manager.handle_surface_configured(id);
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle set_opacity request
    fn handle_set_opacity(
        &self,
        id: u32,
        opacity: f32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        // Validate opacity
        validation::validate_opacity(opacity)
            .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let state_manager = self.state_manager.lock().unwrap();

        // Check if surface exists
        if !state_manager.has_surface(id) {
            return Err(RpcError::surface_not_found(id));
        }

        // Get the IVI API and update the surface
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager); // Release lock before calling IVI API

        let mut surface = ivi_api
            .get_surface_from_id(id)
            .ok_or_else(|| RpcError::internal_error(format!("Failed to get IVI surface {}", id)))?;

        surface
            .set_opacity(opacity)
            .map_err(|e| RpcError::internal_error(e))?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(e.to_string()))?;

            // Update internal state
            let mut state_manager = self.state_manager.lock().unwrap();
            state_manager.handle_surface_configured(id);
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle set_orientation request
    fn handle_set_orientation(
        &self,
        id: u32,
        _orientation: crate::controller::state::Orientation,
        _auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();

        // Check if surface exists
        if !state_manager.has_surface(id) {
            return Err(RpcError::surface_not_found(id));
        }

        // Orientation control is not supported by current IVI API
        drop(state_manager);
        Err(RpcError::internal_error(
            "Orientation control not supported by current IVI API".to_string(),
        ))
    }

    /// Handle set_z_order request
    fn handle_set_z_order(
        &self,
        id: u32,
        z_order: i32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        // Validate z_order (using a reasonable range for now)
        validation::validate_z_order(z_order, 0, 1000)
            .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let state_manager = self.state_manager.lock().unwrap();

        // Check if surface exists
        if !state_manager.has_surface(id) {
            return Err(RpcError::surface_not_found(id));
        }

        // Get the IVI API and update the surface
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager); // Release lock before calling IVI API

        let mut surface = ivi_api
            .get_surface_from_id(id)
            .ok_or_else(|| RpcError::internal_error(format!("Failed to get IVI surface {}", id)))?;

        surface
            .set_z_order(z_order, 0, 1000)
            .map_err(|e| RpcError::internal_error(e))?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(e.to_string()))?;

            // Update internal state and emit notification
            // Capture old z-order first
            let mut state_manager = self.state_manager.lock().unwrap();
            let old_z = state_manager.get_surface(id).map(|s| s.z_order);
            if let Some(mut surface_state) = state_manager.get_surface(id) {
                surface_state.z_order = z_order;
                state_manager.update_surface(id, surface_state);
            }

            // Emit z-order change if we have an old value
            if let Some(old_z_order) = old_z {
                let nm = state_manager.notification_manager().clone();
                drop(state_manager);
                let nm = nm.lock().unwrap();
                nm.emit_z_order_change(id, old_z_order, z_order);
            }
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle set_focus request
    fn handle_set_focus(&self, id: u32, auto_commit: bool) -> Result<serde_json::Value, RpcError> {
        jdebug!(
            "Setting focus for surface {} [auto_commit={}]",
            id,
            auto_commit
        );

        let mut state_manager = self.state_manager.lock().unwrap();

        // Check if surface exists
        if !state_manager.has_surface(id) {
            jwarn!("Surface not found: {}", id);
            return Err(RpcError::surface_not_found(id));
        }

        // Get the IVI API and update the surface
        let ivi_api = state_manager.ivi_api().clone();

        // Update focused surface in state manager (this will emit focus change notification)
        state_manager.set_focused_surface(Some(id));

        drop(state_manager); // Release lock before calling IVI API

        let mut surface = ivi_api
            .get_surface_from_id(id)
            .ok_or_else(|| RpcError::internal_error(format!("Failed to get IVI surface {}", id)))?;

        // Set both keyboard and pointer focus
        surface
            .set_keyboard_focus()
            .map_err(|e| RpcError::internal_error(e))?;
        surface
            .set_pointer_focus()
            .map_err(|e| RpcError::internal_error(e))?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(e.to_string()))?;
            jinfo!("Focus set to surface {} and committed", id);
        } else {
            jinfo!("Focus set to surface {} (pending commit)", id);
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle commit request - commits all pending changes
    fn handle_commit(&self) -> Result<serde_json::Value, RpcError> {
        jdebug!("Committing all pending changes");

        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager);

        // Commit all pending changes
        ivi_api
            .commit_changes()
            .map_err(|e| RpcError::internal_error(e.to_string()))?;

        jinfo!("All pending changes committed");

        Ok(json!({ "success": true }))
    }

    /// Handle subscribe request - subscribe to event types
    fn handle_subscribe(
        &self,
        client_id: ClientId,
        event_types: Vec<EventType>,
    ) -> Result<serde_json::Value, RpcError> {
        jinfo!(
            "Client {} subscribing to {} event types",
            client_id,
            event_types.len()
        );

        let subscription_manager = self.subscription_manager.lock().unwrap();
        let subscribed = subscription_manager
            .subscribe(client_id, event_types)
            .map_err(|e| RpcError::internal_error(e))?;

        jinfo!(
            "Client {} successfully subscribed to {} event types",
            client_id,
            subscribed.len()
        );

        Ok(json!({
            "success": true,
            "subscribed": subscribed
        }))
    }

    /// Handle unsubscribe request - unsubscribe from event types
    fn handle_unsubscribe(
        &self,
        client_id: ClientId,
        event_types: Vec<EventType>,
    ) -> Result<serde_json::Value, RpcError> {
        jinfo!(
            "Client {} unsubscribing from {} event types",
            client_id,
            event_types.len()
        );

        let subscription_manager = self.subscription_manager.lock().unwrap();
        let unsubscribed = subscription_manager
            .unsubscribe(client_id, event_types)
            .map_err(|e| RpcError::internal_error(e))?;

        jinfo!(
            "Client {} successfully unsubscribed from {} event types",
            client_id,
            unsubscribed.len()
        );

        Ok(json!({
            "success": true,
            "unsubscribed": unsubscribed
        }))
    }

    /// Handle list_subscriptions request - list all active subscriptions for a client
    fn handle_list_subscriptions(
        &self,
        client_id: ClientId,
    ) -> Result<serde_json::Value, RpcError> {
        jdebug!("Listing subscriptions for client {}", client_id);

        let subscription_manager = self.subscription_manager.lock().unwrap();
        let subscriptions = subscription_manager.get_subscriptions(client_id);

        jdebug!(
            "Client {} has {} active subscriptions",
            client_id,
            subscriptions.len()
        );

        Ok(json!({
            "subscriptions": subscriptions
        }))
    }

    /// Handle list_layers request
    fn handle_list_layers(&self) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();
        let layers = state_manager.get_all_layers();

        let layer_list: Vec<serde_json::Value> = layers
            .iter()
            .map(|layer| {
                json!({
                    "id": layer.id,
                    "visibility": layer.visibility,
                    "opacity": layer.opacity,
                })
            })
            .collect();

        Ok(json!({ "layers": layer_list }))
    }

    /// Handle get_layer request
    fn handle_get_layer(&self, id: u32) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();

        match state_manager.get_layer(id) {
            Some(layer) => {
                jdebug!("Retrieved layer {}", id);
                Ok(json!({
                    "id": layer.id,
                    "visibility": layer.visibility,
                    "opacity": layer.opacity,
                }))
            }
            None => {
                jwarn!("Layer not found: {}", id);
                Err(RpcError::invalid_params(format!("Layer {} not found", id)))
            }
        }
    }

    /// Handle set_layer_visibility request
    fn handle_set_layer_visibility(
        &self,
        id: u32,
        visible: bool,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        jdebug!(
            "Setting layer {} visibility to {} [auto_commit={}]",
            id,
            visible,
            auto_commit
        );

        let state_manager = self.state_manager.lock().unwrap();

        // Check if layer exists
        if !state_manager.has_layer(id) {
            jwarn!("Layer not found: {}", id);
            return Err(RpcError::invalid_params(format!("Layer {} not found", id)));
        }

        // Get the IVI API and update the layer
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager); // Release lock before calling IVI API

        let mut layer = ivi_api
            .get_layer_from_id(id)
            .ok_or_else(|| RpcError::internal_error(format!("Failed to get IVI layer {}", id)))?;

        layer.set_visibility(visible);

        // Commit changes only if auto_commit is true
        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(e.to_string()))?;

            // Update internal state
            let mut state_manager = self.state_manager.lock().unwrap();
            state_manager.handle_layer_configured(id);
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle set_layer_opacity request
    fn handle_set_layer_opacity(
        &self,
        id: u32,
        opacity: f32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        // Validate opacity
        validation::validate_opacity(opacity)
            .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        jdebug!(
            "Setting layer {} opacity to {} [auto_commit={}]",
            id,
            opacity,
            auto_commit
        );

        let state_manager = self.state_manager.lock().unwrap();

        // Check if layer exists
        if !state_manager.has_layer(id) {
            jwarn!("Layer not found: {}", id);
            return Err(RpcError::invalid_params(format!("Layer {} not found", id)));
        }

        // Get the IVI API and update the layer
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager); // Release lock before calling IVI API

        let mut layer = ivi_api
            .get_layer_from_id(id)
            .ok_or_else(|| RpcError::internal_error(format!("Failed to get IVI layer {}", id)))?;

        layer
            .set_opacity(opacity)
            .map_err(|e| RpcError::internal_error(e))?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(e.to_string()))?;

            // Update internal state
            let mut state_manager = self.state_manager.lock().unwrap();
            state_manager.handle_layer_configured(id);
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }
}

/// Convert a SurfaceState to JSON
fn surface_state_to_json(surface: &SurfaceState) -> serde_json::Value {
    json!({
        "id": surface.id,
        "orig_size": {
            "width": surface.orig_size.0,
            "height": surface.orig_size.1,
        },
        "src_position": {
            "x": surface.src_position.0,
            "y": surface.src_position.1,
        },
        "src_size": {
            "width": surface.src_size.0,
            "height": surface.src_size.1,
        },
        "dest_position": {
            "x": surface.dest_position.0,
            "y": surface.dest_position.1,
        },
        "dest_size": {
            "width": surface.dest_size.0,
            "height": surface.dest_size.1,
        },
        "visibility": surface.visibility,
        "opacity": surface.opacity,
        "orientation": surface.orientation,
        "z_order": surface.z_order,
    })
}

/// Message handler implementation that bridges transport and RPC handler
struct RpcMessageHandler {
    rpc_handler: Arc<RpcHandler>,
}

impl MessageHandler for RpcMessageHandler {
    fn handle_message(&self, client_id: ClientId, data: &[u8]) {
        jtrace!("Received message from client {}", client_id);

        // Parse the incoming message as an RPC request
        let request = match RpcRequest::from_json(data) {
            Ok(req) => req,
            Err(e) => {
                // If we can't parse the request, we can't send a proper response
                // because we don't have a request ID
                jerror!(
                    "Failed to parse RPC request from client {}: {:?}",
                    client_id,
                    e
                );
                return;
            }
        };

        // Handle the request
        let response = self.rpc_handler.handle_request(client_id, request);

        // Serialize the response
        let response_data = match response.to_json() {
            Ok(data) => data,
            Err(e) => {
                jerror!(
                    "Failed to serialize RPC response for client {}: {:?}",
                    client_id,
                    e
                );
                return;
            }
        };

        // Send the response back to the client
        jdebug!(
            "Sending response to client {}, {} bytes",
            client_id,
            response_data.len()
        );
        let transport_lock = self.rpc_handler.transport.lock().unwrap();
        if let Some(transport) = transport_lock.as_ref() {
            match transport.send(client_id, &response_data) {
                Ok(_) => {
                    jdebug!("Successfully sent response to client {}", client_id);
                }
                Err(e) => {
                    jerror!(
                        "Failed to send RPC response to client {}: {:?}",
                        client_id,
                        e
                    );
                }
            }
        } else {
            jwarn!(
                "No transport available to send response to client {}",
                client_id
            );
        }
    }

    fn handle_disconnect(&self, client_id: ClientId) {
        // Log the disconnection
        jinfo!("Client {} disconnected", client_id);

        // Clean up subscriptions for this client
        self.rpc_handler
            .subscription_manager
            .lock()
            .unwrap()
            .remove_client(client_id);

        jdebug!("Cleaned up subscriptions for client {}", client_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::ivi_wrapper::IviLayoutApi;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    /// Mock transport for testing
    struct MockTransport {
        started: AtomicBool,
        stopped: AtomicBool,
        last_client_id: AtomicU64,
        last_message: Mutex<Vec<u8>>,
        handler: Mutex<Option<Box<dyn MessageHandler>>>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                started: AtomicBool::new(false),
                stopped: AtomicBool::new(false),
                last_client_id: AtomicU64::new(0),
                last_message: Mutex::new(Vec::new()),
                handler: Mutex::new(None),
            }
        }

        fn get_last_message(&self) -> Vec<u8> {
            self.last_message.lock().unwrap().clone()
        }

        fn simulate_message(&self, client_id: ClientId, data: &[u8]) {
            let handler = self.handler.lock().unwrap();
            if let Some(h) = handler.as_ref() {
                h.handle_message(client_id, data);
            }
        }
    }

    impl Transport for MockTransport {
        fn start(&mut self) -> Result<(), TransportError> {
            self.started.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn stop(&mut self) -> Result<(), TransportError> {
            self.stopped.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn send(&self, client_id: ClientId, data: &[u8]) -> Result<(), TransportError> {
            self.last_client_id.store(client_id, Ordering::SeqCst);
            *self.last_message.lock().unwrap() = data.to_vec();
            Ok(())
        }

        fn send_to_clients(
            &self,
            client_ids: &[ClientId],
            data: &[u8],
        ) -> Result<(), TransportError> {
            for &client_id in client_ids {
                self.send(client_id, data)?;
            }
            Ok(())
        }

        fn get_connected_clients(&self) -> Vec<ClientId> {
            vec![1]
        }

        fn register_handler(&mut self, handler: Box<dyn MessageHandler>) {
            *self.handler.lock().unwrap() = Some(handler);
        }
    }

    // Helper to create a mock state manager for testing
    fn create_mock_state_manager() -> Arc<Mutex<StateManager>> {
        // Create a mock IVI API with a null pointer (safe for testing as we won't call IVI functions)
        let ivi_api = unsafe {
            // For testing, we create a dummy API pointer
            // This is safe because our tests don't actually call IVI functions
            Arc::new(IviLayoutApi::from_raw(1 as *const _).unwrap())
        };
        Arc::new(Mutex::new(StateManager::new(ivi_api)))
    }

    #[test]
    fn test_transport_registration() {
        let state_manager = create_mock_state_manager();

        // Create RPC handler
        let rpc_handler = RpcHandler::new(state_manager);

        // Create and register mock transport
        let transport = Box::new(MockTransport::new());
        let result = rpc_handler.register_transport(transport);

        assert!(result.is_ok());
    }

    #[test]
    fn test_transport_start_stop() {
        let state_manager = create_mock_state_manager();

        // Create RPC handler
        let rpc_handler = RpcHandler::new(state_manager);

        // Create and register mock transport
        let transport = Box::new(MockTransport::new());
        rpc_handler.register_transport(transport).unwrap();

        // Start transport
        let result = rpc_handler.start_transport();
        assert!(result.is_ok());

        // Stop transport
        let result = rpc_handler.stop_transport();
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_handler_integration() {
        let state_manager = create_mock_state_manager();

        // Create RPC handler
        let rpc_handler = RpcHandler::new(state_manager);

        // Create and register mock transport
        let transport = Box::new(MockTransport::new());
        rpc_handler.register_transport(transport).unwrap();

        // Create a list_surfaces request
        let request = RpcRequest::new(1, "list_surfaces".to_string(), json!({}));
        let _request_data = request.to_json().unwrap();

        // Verify the transport is registered
        let transport_lock = rpc_handler.transport.lock().unwrap();
        assert!(transport_lock.is_some());
    }
}
