// RPC request handler

use super::protocol::{RpcError, RpcMethod, RpcRequest, RpcResponse};
use super::transport::{ClientId, MessageHandler, Transport, TransportError};
use crate::controller::state::{StateManager, SurfaceState};
use crate::controller::validation;
#[allow(unused)]
use jlogger_tracing::{jtrace,jdebug, jerror, jinfo, jwarn, JloggerBuilder, LevelFilter};
use serde_json::json;
use std::sync::{Arc, Mutex};

/// Handles RPC requests and generates responses
pub struct RpcHandler {
    state_manager: Arc<Mutex<StateManager>>,
    transport: Arc<Mutex<Option<Box<dyn Transport>>>>,
}

impl RpcHandler {
    /// Create a new RPC handler wrapped in Arc for shared ownership
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Arc<Self> {
        Arc::new(Self {
            state_manager,
            transport: Arc::new(Mutex::new(None)),
        })
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

    /// Handle an RPC request
    pub fn handle_request(&self, request: RpcRequest) -> RpcResponse {
        jdebug!(
            "Handling RPC request: method={}, id={}",
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
        orientation: crate::controller::state::Orientation,
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

        // Convert orientation to degrees
        let degrees = match orientation {
            crate::controller::state::Orientation::Normal => 0,
            crate::controller::state::Orientation::Rotate90 => 90,
            crate::controller::state::Orientation::Rotate180 => 180,
            crate::controller::state::Orientation::Rotate270 => 270,
        };

        surface
            .set_orientation(degrees)
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

            // Update internal state
            let mut state_manager = self.state_manager.lock().unwrap();
            if let Some(mut surface_state) = state_manager.get_surface(id) {
                surface_state.z_order = z_order;
                state_manager.update_surface(id, surface_state);
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
}

/// Convert a SurfaceState to JSON
fn surface_state_to_json(surface: &SurfaceState) -> serde_json::Value {
    json!({
        "id": surface.id,
        "position": {
            "x": surface.position.0,
            "y": surface.position.1,
        },
        "size": {
            "width": surface.size.0,
            "height": surface.size.1,
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
        let response = self.rpc_handler.handle_request(request);

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
        let transport_lock = self.rpc_handler.transport.lock().unwrap();
        if let Some(transport) = transport_lock.as_ref() {
            if let Err(e) = transport.send(client_id, &response_data) {
                jerror!(
                    "Failed to send RPC response to client {}: {:?}",
                    client_id,
                    e
                );
            }
        }
    }

    fn handle_disconnect(&self, client_id: ClientId) {
        // Log the disconnection
        jinfo!("Client {} disconnected", client_id);
        // No cleanup needed for now, but this is where we could
        // clean up any per-client state if needed
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
