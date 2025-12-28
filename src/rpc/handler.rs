// RPC request handler

use super::protocol::{EventType, RpcError, RpcMethod, RpcRequest, RpcResponse};
use super::transport::{ClientId, MessageHandler, Transport, TransportError};
use crate::controller::state::{StateManager, SurfaceState};
use crate::controller::subscriptions::SubscriptionManager;
use crate::controller::validation;
use crate::ffi::bindings::ivi_surface::IviSurface;
use crate::ffi::bindings::weston_output_m::ScreenInfo;
use crate::ffi::bindings::Rectangle;
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
                            .drain_notifications(&client_id);

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
                                        if let Err(e) = t.send(&client_id, &json) {
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
    pub fn handle_request(&self, client_id: &ClientId, request: RpcRequest) -> RpcResponse {
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
            RpcMethod::SetSurfaceSourceRectangle {
                id,
                x,
                y,
                width,
                height,
            } => self.handle_set_surface_source_rectangle(id, x, y, width, height, auto_commit),
            RpcMethod::SetSurfaceDestinationRectangle {
                id,
                x,
                y,
                width,
                height,
            } => {
                self.handle_set_surface_destination_rectangle(id, x, y, width, height, auto_commit)
            }
            RpcMethod::SetSurfaceVisibility { id, visible } => {
                self.handle_set_surface_visibility(id, visible, auto_commit)
            }
            RpcMethod::SetSurfaceOpacity { id, opacity } => {
                self.handle_set_surface_opacity(id, opacity, auto_commit)
            }
            RpcMethod::SetSurfaceZOrder { id, z_order } => {
                self.handle_set_surface_z_order(id, z_order, auto_commit)
            }
            RpcMethod::SetSurfaceFocus { id } => self.handle_set_surface_focus(id, auto_commit),
            RpcMethod::Commit => self.handle_commit(),

            // Subscription methods
            RpcMethod::Subscribe { event_types } => self.handle_subscribe(client_id, event_types),
            RpcMethod::Unsubscribe { event_types } => {
                self.handle_unsubscribe(client_id, event_types)
            }
            RpcMethod::ListSubscriptions => self.handle_list_subscriptions(client_id),

            // Layer methods
            RpcMethod::ListLayers => self.handle_list_layers(),
            RpcMethod::CreateLayer { id, width, height } => {
                self.handle_create_layer(id, width, height, auto_commit)
            }
            RpcMethod::DestroyLayer { id } => self.handle_destroy_layer(id, auto_commit),
            RpcMethod::GetLayer { id } => self.handle_get_layer(id),
            RpcMethod::SetLayerSourceRectangle {
                id,
                x,
                y,
                width,
                height,
            } => self.handle_set_layer_source_rectangle(id, x, y, width, height, auto_commit),
            RpcMethod::SetLayerDestinationRectangle {
                id,
                x,
                y,
                width,
                height,
            } => self.handle_set_layer_destination_rectangle(id, x, y, width, height, auto_commit),
            RpcMethod::SetLayerVisibility { id, visible } => {
                self.handle_set_layer_visibility(id, visible, auto_commit)
            }
            RpcMethod::SetLayerOpacity { id, opacity } => {
                self.handle_set_layer_opacity(id, opacity, auto_commit)
            }
            // Layer-surface assignment operations
            RpcMethod::SetLayerSurfaces {
                layer_id,
                surface_ids,
                auto_commit,
            } => self.handle_set_layer_surfaces(layer_id, surface_ids, auto_commit),
            RpcMethod::AddSurfaceToLayer {
                layer_id,
                surface_id,
                auto_commit,
            } => self.handle_add_surface_to_layer(layer_id, surface_id, auto_commit),
            RpcMethod::RemoveSurfaceFromLayer {
                layer_id,
                surface_id,
                auto_commit,
            } => self.handle_remove_surface_from_layer(layer_id, surface_id, auto_commit),
            RpcMethod::GetLayerSurfaces { layer_id } => self.handle_get_layer_surfaces(layer_id),
            // Screen operations
            RpcMethod::ListScreens => self.handle_list_screens(),
            RpcMethod::GetScreen { name } => self.handle_get_screen(name),
            RpcMethod::GetScreenLayers { screen_name } => {
                self.handle_get_screen_layers(screen_name)
            }
            RpcMethod::GetLayerScreens { layer_id } => self.handle_get_layer_screens(layer_id),
            RpcMethod::AddLayersToScreen {
                screen_name,
                layer_ids,
                auto_commit,
            } => self.handle_add_layers_to_screen(screen_name, layer_ids, auto_commit),
            RpcMethod::RemoveLayerFromScreen {
                screen_name,
                layer_id,
                auto_commit,
            } => self.handle_remove_layer_from_screen(screen_name, layer_id, auto_commit),
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
            surfaces.iter().map(surface_state_to_json).collect();

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

    fn id_to_surface(&self, id: u32) -> Option<IviSurface> {
        let state_manager = self.state_manager.lock().unwrap();

        let ivi_api = state_manager.ivi_api().clone();
        ivi_api.get_surface_from_id(id)
    }

    fn commit_surface_changes(&self, id: u32) -> Result<(), RpcError> {
        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager);

        ivi_api
            .commit_changes()
            .map_err(|e| RpcError::internal_error(e.to_string()))?;

        // Update internal state
        let mut state_manager = self.state_manager.lock().unwrap();
        state_manager.handle_surface_configured(id);

        Ok(())
    }

    fn id_to_layer(&self, id: u32) -> Option<crate::ffi::bindings::ivi_layer::IviLayer> {
        let state_manager = self.state_manager.lock().unwrap();

        let ivi_api = state_manager.ivi_api().clone();
        ivi_api.get_layer_from_id(id)
    }

    fn commit_layer_changes(&self, id: u32) -> Result<(), RpcError> {
        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager);

        ivi_api
            .commit_changes()
            .map_err(|e| RpcError::internal_error(e.to_string()))?;

        // Update internal state
        let mut state_manager = self.state_manager.lock().unwrap();
        state_manager.handle_layer_configured(id);

        Ok(())
    }

    /// Handle set_surface_source_rectangle request
    fn handle_set_surface_source_rectangle(
        &self,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        jdebug!(
            "Setting source region for surface {}: {}x{}@({}, {}) [auto_commit={}]",
            id,
            width,
            height,
            x,
            y,
            auto_commit
        );

        // Validate position
        validation::validate_position(x, y).map_err(|e| {
            jwarn!("Invalid position for surface {}: {}", id, e);
            RpcError::invalid_params(e.to_string())
        })?;

        let mut surface = self
            .id_to_surface(id)
            .ok_or_else(|| RpcError::surface_not_found(id))?;

        surface
            .set_source_rectangle(Rectangle {
                x,
                y,
                width,
                height,
            })
            .map_err(RpcError::internal_error)?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            self.commit_surface_changes(id)?;
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle set_surface_destination_rectangle request
    fn handle_set_surface_destination_rectangle(
        &self,
        id: u32,
        x: i32,
        y: i32,
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

        // Validate position
        validation::validate_position(x, y).map_err(|e| RpcError::invalid_params(e.to_string()))?;
        // Validate size
        validation::validate_size(width, height)
            .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let mut surface = self
            .id_to_surface(id)
            .ok_or_else(|| RpcError::surface_not_found(id))?;

        surface
            .set_destination_rectangle(Rectangle {
                x,
                y,
                width,
                height,
            })
            .map_err(RpcError::internal_error)?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            self.commit_surface_changes(id)?;
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle set_surface_visibility request
    fn handle_set_surface_visibility(
        &self,
        id: u32,
        visible: bool,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        let mut surface = self
            .id_to_surface(id)
            .ok_or_else(|| RpcError::surface_not_found(id))?;

        surface
            .set_visibility(visible)
            .map_err(RpcError::internal_error)?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            self.commit_surface_changes(id)?;
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle set_surface_opacity request
    fn handle_set_surface_opacity(
        &self,
        id: u32,
        opacity: f32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        // Validate opacity
        validation::validate_opacity(opacity)
            .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let mut surface = self
            .id_to_surface(id)
            .ok_or_else(|| RpcError::surface_not_found(id))?;

        surface
            .set_opacity(opacity)
            .map_err(RpcError::internal_error)?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            self.commit_surface_changes(id)?;
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle set_surface_z_order request
    fn handle_set_surface_z_order(
        &self,
        id: u32,
        z_order: i32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        // Validate z_order (using a reasonable range for now)
        validation::validate_z_order(z_order, 0, 1000)
            .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let mut surface = self
            .id_to_surface(id)
            .ok_or_else(|| RpcError::surface_not_found(id))?;

        surface
            .set_z_order(z_order, 0, 1000)
            .map_err(RpcError::internal_error)?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            let state_manager = self.state_manager.lock().unwrap();
            let ivi_api = state_manager.ivi_api().clone();
            drop(state_manager);

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

    /// Handle set_surface_focus request
    fn handle_set_surface_focus(
        &self,
        id: u32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        jdebug!(
            "Setting focus for surface {} [auto_commit={}]",
            id,
            auto_commit
        );

        let mut surface = self
            .id_to_surface(id)
            .ok_or_else(|| RpcError::surface_not_found(id))?;

        // Set both keyboard and pointer focus
        surface
            .set_keyboard_focus()
            .map_err(RpcError::internal_error)?;
        surface
            .set_pointer_focus()
            .map_err(RpcError::internal_error)?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            let state_manager = self.state_manager.lock().unwrap();
            let ivi_api = state_manager.ivi_api().clone();
            drop(state_manager);
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
        client_id: &ClientId,
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
            .map_err(RpcError::internal_error)?;

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
        client_id: &ClientId,
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
            .map_err(RpcError::internal_error)?;

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
        client_id: &ClientId,
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
                    "src_rect": {
                        "x": layer.src_rect.0,
                        "y": layer.src_rect.1,
                        "width": layer.src_rect.2,
                        "height": layer.src_rect.3,
                    },
                    "dest_rect": {
                        "x": layer.dest_rect.0,
                        "y": layer.dest_rect.1,
                        "width": layer.dest_rect.2,
                        "height": layer.dest_rect.3,
                    },
                    "visibility": layer.visibility,
                    "opacity": layer.opacity,
                    "orientation": layer.orientation,
                })
            })
            .collect();

        Ok(json!({ "layers": layer_list }))
    }

    fn handle_create_layer(
        &self,
        id: u32,
        width: i32,
        height: i32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        jdebug!("Creating new layer with size {}x{}", width, height);

        // Validate size
        validation::validate_size(width, height)
            .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager);

        // Create the layer via IVI API
        let layer = ivi_api
            .layer_create_with_dimension(id, width, height)
            .map_err(|e| RpcError::internal_error(e.to_string()))?;

        jinfo!("Created new layer with ID {}", layer.id());

        // Commit changes only if auto_commit is true
        if auto_commit {
            self.commit_layer_changes(id)?;
        }

        Ok(json!({
            "id": layer.id(),
            "committed": auto_commit,
        }))
    }

    /// Handle destroy_layer request
    fn handle_destroy_layer(
        &self,
        id: u32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        jdebug!("Destroying layer {} [auto_commit={}]", id, auto_commit);

        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();

        // Verify layer exists before attempting to destroy
        if !state_manager.has_layer(id) {
            jwarn!("Layer not found: {}", id);
            return Err(RpcError::layer_not_found(id));
        }

        drop(state_manager);

        // Get the layer from the IVI API
        let layer = ivi_api
            .get_layer_from_id(id)
            .ok_or_else(|| RpcError::internal_error(format!("Failed to get IVI layer {}", id)))?;

        // Destroy the layer (consumes self)
        layer
            .destroy()
            .map_err(|e| RpcError::internal_error(format!("Failed to destroy layer: {}", e)))?;

        jinfo!("Layer {} destroyed", id);

        // Commit changes if auto_commit is true
        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(e.to_string()))?;

            // Update internal state - the layer is now destroyed
            let mut state_manager = self.state_manager.lock().unwrap();
            state_manager.handle_layer_destroyed(id);

            jinfo!("Layer {} destruction committed", id);
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle get_layer request
    fn handle_get_layer(&self, id: u32) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();

        match state_manager.get_layer(id) {
            Some(layer) => {
                jdebug!("Retrieved layer {}", id);
                Ok(json!({
                    "id": layer.id,
                    "src_rect": {
                        "x": layer.src_rect.0,
                        "y": layer.src_rect.1,
                        "width": layer.src_rect.2,
                        "height": layer.src_rect.3,
                    },
                    "dest_rect": {
                        "x": layer.dest_rect.0,
                        "y": layer.dest_rect.1,
                        "width": layer.dest_rect.2,
                        "height": layer.dest_rect.3,
                    },
                    "visibility": layer.visibility,
                    "opacity": layer.opacity,
                    "orientation": layer.orientation,
                }))
            }
            None => {
                jwarn!("Layer not found: {}", id);
                Err(RpcError::invalid_params(format!("Layer {} not found", id)))
            }
        }
    }

    /// Handle set_layer_source_rectangle request
    fn handle_set_layer_source_rectangle(
        &self,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        validation::validate_position(x, y).map_err(|e| RpcError::invalid_params(e.to_string()))?;
        validation::validate_size(width, height)
            .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let mut layer = self
            .id_to_layer(id)
            .ok_or_else(|| RpcError::invalid_params(format!("Layer {} not found", id)))?;

        layer
            .set_source_rectangle(Rectangle {
                x,
                y,
                width,
                height,
            })
            .map_err(RpcError::internal_error)?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            self.commit_layer_changes(id)?;
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
    }

    /// Handle set_layer_destination_rectangle request
    fn handle_set_layer_destination_rectangle(
        &self,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        validation::validate_position(x, y).map_err(|e| RpcError::invalid_params(e.to_string()))?;
        validation::validate_size(width, height)
            .map_err(|e| RpcError::invalid_params(e.to_string()))?;

        let mut layer = self
            .id_to_layer(id)
            .ok_or_else(|| RpcError::invalid_params(format!("Layer {} not found", id)))?;

        layer
            .set_destination_rectangle(Rectangle {
                x,
                y,
                width,
                height,
            })
            .map_err(RpcError::internal_error)?;

        // Commit changes only if auto_commit is true
        if auto_commit {
            self.commit_layer_changes(id)?;
        }

        Ok(json!({ "success": true, "committed": auto_commit }))
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

        layer
            .set_visibility(visible)
            .map_err(RpcError::internal_error)?;

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
            .map_err(RpcError::internal_error)?;

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

    /// Handle set_layer_surfaces: Replace all surfaces on a layer
    fn handle_set_layer_surfaces(
        &self,
        layer_id: u32,
        surface_ids: Vec<u32>,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        jdebug!(
            "Setting layer {} surfaces to {:?} [auto_commit={}]",
            layer_id,
            surface_ids,
            auto_commit
        );

        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();

        // Get layer and verify it exists
        let layer = ivi_api
            .get_layer_from_id(layer_id)
            .ok_or_else(|| RpcError::internal_error(format!("Layer {} not found", layer_id)))?;

        // Build surface vector by getting each surface from ID
        let surfaces: Vec<_> = surface_ids
            .iter()
            .filter_map(|&id| ivi_api.get_surface_from_id(id))
            .collect();

        // Verify all surfaces were found
        if surfaces.len() != surface_ids.len() {
            return Err(RpcError::internal_error(
                "Some surfaces not found".to_string(),
            ));
        }

        drop(state_manager);

        // Build reference slice: first = bottommost, last = topmost
        let surface_refs: Vec<&_> = surfaces.iter().collect();

        // Set render order
        ivi_api
            .layer_set_render_order(&layer, &surface_refs)
            .map_err(|e| RpcError::internal_error(format!("Failed to set render order: {}", e)))?;

        // Commit if requested
        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(e.to_string()))?;
        }

        Ok(json!({
            "layer_id": layer_id,
            "surface_ids": surface_ids,
            "committed": auto_commit
        }))
    }

    /// Handle add_surface_to_layer: Add a surface as topmost
    fn handle_add_surface_to_layer(
        &self,
        layer_id: u32,
        surface_id: u32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        jdebug!(
            "Adding surface {} to layer {} as topmost [auto_commit={}]",
            surface_id,
            layer_id,
            auto_commit
        );

        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();

        // Get layer and verify it exists
        let layer = ivi_api
            .get_layer_from_id(layer_id)
            .ok_or_else(|| RpcError::internal_error(format!("Layer {} not found", layer_id)))?;

        // Get current surfaces on the layer
        let mut surfaces = ivi_api.get_surfaces_on_layer(&layer);

        // Get new surface and verify it exists
        let new_surface = ivi_api
            .get_surface_from_id(surface_id)
            .ok_or_else(|| RpcError::internal_error(format!("Surface {} not found", surface_id)))?;

        drop(state_manager);

        // Append new surface to end (topmost position)
        surfaces.push(new_surface);

        // Build reference slice
        let surface_refs: Vec<&_> = surfaces.iter().collect();

        // Set render order
        ivi_api
            .layer_set_render_order(&layer, &surface_refs)
            .map_err(|e| RpcError::internal_error(format!("Failed to set render order: {}", e)))?;

        // Commit if requested
        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(e.to_string()))?;
        }

        Ok(json!({
            "layer_id": layer_id,
            "surface_id": surface_id,
            "committed": auto_commit
        }))
    }

    /// Handle remove_surface_from_layer: Remove a surface from a layer
    fn handle_remove_surface_from_layer(
        &self,
        layer_id: u32,
        surface_id: u32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        jdebug!(
            "Removing surface {} from layer {} [auto_commit={}]",
            surface_id,
            layer_id,
            auto_commit
        );

        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();

        // Get layer and verify it exists
        let layer = ivi_api
            .get_layer_from_id(layer_id)
            .ok_or_else(|| RpcError::internal_error(format!("Layer {} not found", layer_id)))?;

        // Get surface to remove and verify it exists
        let surface = ivi_api
            .get_surface_from_id(surface_id)
            .ok_or_else(|| RpcError::internal_error(format!("Surface {} not found", surface_id)))?;

        drop(state_manager);

        // Remove surface from layer
        ivi_api
            .layer_remove_surface(&layer, &surface)
            .map_err(|e| {
                RpcError::internal_error(format!("Failed to remove surface from layer: {}", e))
            })?;

        // Commit if requested
        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(e.to_string()))?;
        }

        Ok(json!({
            "layer_id": layer_id,
            "surface_id": surface_id,
            "committed": auto_commit
        }))
    }

    /// Handle get_layer_surfaces: Get surfaces assigned to a layer
    fn handle_get_layer_surfaces(&self, layer_id: u32) -> Result<serde_json::Value, RpcError> {
        jdebug!("Getting surfaces for layer {}", layer_id);

        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();

        // Get layer and verify it exists
        let layer = ivi_api
            .get_layer_from_id(layer_id)
            .ok_or_else(|| RpcError::internal_error(format!("Layer {} not found", layer_id)))?;

        drop(state_manager);

        // Get surfaces on the layer
        let surfaces = ivi_api.get_surfaces_on_layer(&layer);

        // Extract surface IDs (order preserved: first = bottommost, last = topmost)
        let surface_ids: Vec<u32> = surfaces.iter().map(|s| s.id()).collect();

        Ok(json!({ "surface_ids": surface_ids }))
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
        "src_rect": {
            "x": surface.src_rect.x,
            "y": surface.src_rect.y,
            "width": surface.src_rect.width,
            "height": surface.src_rect.height,
        },
        "dest_rect": {
            "x": surface.dest_rect.x,
            "y": surface.dest_rect.y,
            "width": surface.dest_rect.width,
            "height": surface.dest_rect.height,
        },
        "visibility": surface.visibility,
        "opacity": surface.opacity,
        "orientation": surface.orientation,
        "z_order": surface.z_order,
    })
}

impl RpcHandler {
    /// List all screens
    fn handle_list_screens(&self) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager);

        let screens = ivi_api.get_screens();
        let screen_infos: Vec<serde_json::Value> = screens
            .iter()
            .map(|output| {
                let info = ScreenInfo::from(output.clone());
                json!({
                    "name": info.name,
                    "width": info.width,
                    "height": info.height,
                    "x": info.coord_global.x,
                    "y": info.coord_global.y,
                    "transform": info.transform.to_string(),
                    "enabled": info.enabled,
                    "scale": info.scale,
                })
            })
            .collect();

        Ok(json!({ "screens": screen_infos }))
    }

    /// Get a specific screen by name
    fn handle_get_screen(&self, name: String) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager);

        let screens = ivi_api.get_screens();
        let screen = screens
            .iter()
            .find(|output| output.name() == Some(name.clone()))
            .ok_or_else(|| RpcError::internal_error(format!("Screen '{}' not found", name)))?;

        let screen_info = ScreenInfo::from(screen.clone());

        Ok(json!({
            "name": screen_info.name,
            "width": screen_info.width,
            "height": screen_info.height,
            "x": screen_info.coord_global.x,
            "y": screen_info.coord_global.y,
            "transform": screen_info.transform.to_string(),
            "enabled": screen_info.enabled,
            "scale": screen_info.scale,
        }))
    }

    /// Get layers assigned to a screen
    fn handle_get_screen_layers(&self, screen_name: String) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();
        drop(state_manager);

        let screens = ivi_api.get_screens();
        let screen = screens
            .iter()
            .find(|output| output.name() == Some(screen_name.clone()))
            .ok_or_else(|| {
                RpcError::internal_error(format!("Screen '{}' not found", screen_name))
            })?;

        let layers;
        unsafe {
            layers = ivi_api
                .get_layers_on_screen((*screen).clone().into())
                .map_err(|e| RpcError::internal_error(format!("Failed to get layers: {}", e)))?;
        }
        let layer_ids: Vec<u32> = layers.iter().map(|layer| layer.id()).collect();

        Ok(json!({ "layer_ids": layer_ids }))
    }

    /// Get screens assigned to a layer
    fn handle_get_layer_screens(&self, layer_id: u32) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();

        // Get the layer
        let layer = ivi_api
            .get_layer_from_id(layer_id)
            .ok_or_else(|| RpcError::layer_not_found(layer_id))?;

        drop(state_manager);

        let screens = ivi_api
            .get_screens_under_layer(&layer)
            .map_err(|e| RpcError::internal_error(format!("Failed to get screens: {}", e)))?;

        let screen_names: Vec<String> = screens.iter().filter_map(|output| output.name()).collect();

        Ok(json!({ "screen_names": screen_names }))
    }

    /// Add layers to a screen (sets render order, replaces existing)
    fn handle_add_layers_to_screen(
        &self,
        screen_name: String,
        layer_ids: Vec<u32>,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();

        // Find the screen
        let screens = ivi_api.get_screens();
        let screen = screens
            .iter()
            .find(|output| output.name() == Some(screen_name.clone()))
            .ok_or_else(|| {
                RpcError::internal_error(format!("Screen '{}' not found", screen_name))
            })?;

        // Verify all layers exist and build layer array
        let layers: Vec<_> = layer_ids
            .iter()
            .filter_map(|&id| {
                state_manager.get_layer(id)?;
                ivi_api.get_layer_from_id(id)
            })
            .collect();

        // Verify we got all layers
        if layers.len() != layer_ids.len() {
            return Err(RpcError::internal_error(
                "Some layers not found".to_string(),
            ));
        }

        drop(state_manager);

        // Set render order - convert Vec<IviLayer> to &[&IviLayer]
        let layer_refs: Vec<&_> = layers.iter().collect();
        ivi_api
            .screen_set_render_order((*screen).clone(), &layer_refs)
            .map_err(|e| RpcError::internal_error(format!("Failed to set render order: {}", e)))?;

        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(format!("Failed to commit: {}", e)))?;
        }

        Ok(json!({
            "screen_name": screen_name,
            "layer_ids": layer_ids,
            "committed": auto_commit
        }))
    }

    /// Remove a layer from a screen
    fn handle_remove_layer_from_screen(
        &self,
        screen_name: String,
        layer_id: u32,
        auto_commit: bool,
    ) -> Result<serde_json::Value, RpcError> {
        let state_manager = self.state_manager.lock().unwrap();
        let ivi_api = state_manager.ivi_api().clone();

        // Find the screen
        let screens = ivi_api.get_screens();
        let screen = screens
            .iter()
            .find(|output| output.name() == Some(screen_name.clone()))
            .ok_or_else(|| {
                RpcError::internal_error(format!("Screen '{}' not found", screen_name))
            })?;

        // Get the layer using get_layer_from_id
        let layer = ivi_api
            .get_layer_from_id(layer_id)
            .ok_or_else(|| RpcError::layer_not_found(layer_id))?;

        drop(state_manager);

        // Remove layer from screen
        ivi_api
            .screen_remove_layer(screen.clone(), &layer)
            .map_err(|e| RpcError::internal_error(format!("Failed to remove layer: {}", e)))?;

        if auto_commit {
            ivi_api
                .commit_changes()
                .map_err(|e| RpcError::internal_error(format!("Failed to commit: {}", e)))?;
        }

        Ok(json!({
            "screen_name": screen_name,
            "layer_id": layer_id,
            "committed": auto_commit
        }))
    }
}

/// Message handler implementation that bridges transport and RPC handler
struct RpcMessageHandler {
    rpc_handler: Arc<RpcHandler>,
}

impl MessageHandler for RpcMessageHandler {
    fn handle_message(&self, client_id: &ClientId, data: &[u8]) {
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

    fn handle_disconnect(&self, client_id: &ClientId) {
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
    use crate::ffi::bindings::ivi_layout_api::IviLayoutApi;
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

        fn send(&self, client_id: &ClientId, data: &[u8]) -> Result<(), TransportError> {
            if let Some(id) = client_id.unix_domain_id() {
                self.last_client_id.store(id, Ordering::SeqCst);
                *self.last_message.lock().unwrap() = data.to_vec();
                Ok(())
            } else {
                Err(TransportError::SendError("Invalid client ID".to_string()))
            }
        }

        fn send_to_clients(
            &self,
            client_ids: &[&ClientId],
            data: &[u8],
        ) -> Result<(), TransportError> {
            for &client_id in client_ids {
                self.send(client_id, data)?;
            }
            Ok(())
        }

        fn get_connected_clients(&self) -> Vec<ClientId> {
            vec![ClientId::from_u64(1)]
        }

        fn register_handler(&mut self, handler: Box<dyn MessageHandler>) {
            *self.handler.lock().unwrap() = Some(handler);
        }
    }

    // Helper to create a mock state manager for testing
    fn create_mock_state_manager() -> Arc<Mutex<StateManager>> {
        // Create a mock IVI API with a null pointer (safe for testing as we won't call IVI functions)
        let ivi_api = {
            // For testing, we create a dummy API pointer
            // This is safe because our tests don't actually call IVI functions
            Arc::new(IviLayoutApi::from_raw(std::ptr::dangling()).unwrap())
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
