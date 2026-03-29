//! IVI Client implementation for connecting to and communicating with the IVI controller.
//!
//! This module provides the main `IviClient` struct that manages the connection to the
//! Weston IVI controller via UNIX domain sockets and handles JSON-RPC communication.
//!
#[cfg(not(feature = "enable-ipcon"))]
pub mod unix_domain;

#[cfg(feature = "enable-ipcon")]
pub mod ipcon;

use crate::error::{IviError, Result};
use crate::ffi::*;
use crate::protocol::{EventType, JsonRpcRequest, JsonRpcResponse, Notification};
#[allow(unused)]
use jlogger_tracing::{jdebug, jerror, jinfo, jtrace, jwarn};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

#[cfg(not(feature = "enable-ipcon"))]
use unix_domain::UnixDomainIviClient;

#[cfg(feature = "enable-ipcon")]
pub use ipcon::IpconIviClient;

pub enum IviRequestResult {
    /// Result of creating a layer, returns the new layer ID
    CreateLayer(LayerId),
    GetLayer(IviLayer),
}

trait IviClientTransport: Send {
    fn send_request(&mut self, request: &[u8]) -> Result<()>;
    fn receive_response(&mut self) -> Result<Vec<u8>>;
    fn disconnect(&mut self) -> Result<()>;
    fn set_read_timeout(&mut self, timeout: Option<Duration>) -> Result<()>;
}

pub struct IviClient {
    transport: Option<Box<dyn IviClientTransport>>,

    /// Atomic counter for generating unique request IDs
    request_id: AtomicU64,
}

impl IviClient {
    pub fn new(remote: Option<&str>) -> Result<Self> {
        let mut client = IviClient {
            transport: None,
            request_id: AtomicU64::new(1),
        };

        #[cfg(not(feature = "enable-ipcon"))]
        client.ud_connect(remote)?;

        #[cfg(feature = "enable-ipcon")]
        client.ipcon_connect(None, remote)?;

        Ok(client)
    }

    #[cfg(not(feature = "enable-ipcon"))]
    /// Connect to the IVI controller via a UNIX domain socket at the specified path.
    /// Ipcon connection is not supported via this method.
    fn ud_connect(&mut self, socket_path: Option<&str>) -> Result<()> {
        let ud_client = UnixDomainIviClient::connect(socket_path)?;
        self.transport = Some(Box::new(ud_client));
        Ok(())
    }

    #[cfg(feature = "enable-ipcon")]
    fn ipcon_connect(&mut self, peer: Option<&str>, server: Option<&str>) -> Result<()> {
        let ipcon_client = IpconIviClient::ipcon_connect(peer, server)?;
        self.transport = Some(Box::new(ipcon_client));
        Ok(())
    }

    /// Disconnects from the IVI controller and closes the socket.
    ///
    /// This method consumes the client and ensures the connection is properly closed.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful disconnection, or an error if closing the socket fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.disconnect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(mut transport) = self.transport.take() {
            transport.disconnect()
        } else {
            Ok(())
        }
    }

    /// Generates the next unique request ID.
    ///
    /// This method uses an atomic counter to ensure thread-safe ID generation.
    ///
    /// # Returns
    ///
    /// Returns a unique u64 request ID.
    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Sends a JSON-RPC request to the IVI controller and receives the response.
    ///
    /// This is an internal helper method that handles the low-level communication:
    /// 1. Generates a unique request ID
    /// 2. Serializes the request to JSON
    /// 3. Sends the request over the socket with newline termination
    /// 4. Receives the response from the socket
    /// 5. Deserializes and validates the response
    ///
    /// # Arguments
    ///
    /// * `method` - The JSON-RPC method name to invoke
    /// * `params` - The parameters to pass to the method as a JSON value
    ///
    /// # Returns
    ///
    /// Returns the result value from a successful response, or an error if:
    /// - Serialization fails
    /// - Network communication fails
    /// - Deserialization fails
    /// - The server returns an error response
    ///
    /// # Errors
    ///
    /// - `IviError::SerializationError` - Failed to serialize the request
    /// - `IviError::IoError` - Network communication error
    /// - `IviError::DeserializationError` - Failed to deserialize the response
    /// - `IviError::RequestFailed` - The server returned an error response
    pub(crate) fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        // Generate unique request ID
        let request_id = self.next_request_id();

        // Create JSON-RPC request
        let request = JsonRpcRequest::new(request_id, method, params);

        jtrace!(
            event = "ivi_client_send_request",
            request_id = request_id,
            method = method,
            request = format!("{:?}", request.params)
        );

        // Serialize request to JSON (as bytes for length-prefix protocol)
        let request_json = serde_json::to_vec(&request)
            .map_err(|e| IviError::SerializationError(e.to_string()))?;

        let transport = self.transport.as_mut().ok_or_else(|| {
            IviError::ConnectionFailed("No active connection to send request.".to_string())
        })?;

        transport.send_request(&request_json)?;
        let response_buf = transport.receive_response()?;

        // Deserialize response
        let response: JsonRpcResponse = serde_json::from_slice(&response_buf)
            .map_err(|e| IviError::DeserializationError(e.to_string()))?;

        jtrace!(
            event = "ivi_client_receive_response",
            request_id = request_id,
            result = response
                .result
                .as_ref()
                .map(|r| format!("{:?}", r))
                .unwrap_or("None".to_string()),
            error = response
                .error
                .as_ref()
                .map(|e| format!("{:?}", e))
                .unwrap_or("None".to_string()),
        );

        // Verify response ID matches request ID
        if response.id != request_id {
            return Err(IviError::DeserializationError(format!(
                "Response ID mismatch: expected {}, got {}",
                request_id, response.id
            )));
        }

        // Check for error response
        if let Some(error) = response.error {
            return Err(IviError::RequestFailed {
                code: error.code,
                message: error.message,
            });
        }

        // Return the result value
        response.result.ok_or_else(|| {
            IviError::DeserializationError("Response missing both result and error".to_string())
        })
    }

    /// Lists all available surfaces in the IVI compositor.
    ///
    /// # Returns
    ///
    /// Returns a vector of `Surface` objects representing all surfaces currently
    /// managed by the IVI controller.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Communication with the controller fails
    /// - The response cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// let surfaces = client.list_surfaces()?;
    /// for surface in surfaces {
    ///     println!("Surface ID: {}", surface.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_surfaces(&mut self) -> Result<Vec<IviSurface>> {
        let result = self.send_request("list_surfaces", json!({}))?;
        jdebug!("list_surfaces result: {}", result);

        // Extract the "surfaces" array from the result object
        let surfaces: Vec<IviSurface> = serde_json::from_value(
            result
                .get("surfaces")
                .ok_or_else(|| {
                    IviError::DeserializationError(
                        "Missing 'surfaces' field in response".to_string(),
                    )
                })?
                .clone(),
        )
        .map_err(|e| IviError::DeserializationError(format!("Failed to parse surfaces: {}", e)))?;

        Ok(surfaces)
    }

    /// Gets detailed properties of a specific surface.
    ///
    /// # Arguments
    ///
    /// * `id` - The surface ID to query
    ///
    /// # Returns
    ///
    /// Returns a `Surface` object containing all properties of the specified surface.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The surface ID does not exist
    /// - Communication with the controller fails
    /// - The response cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// let surface = client.get_surface(1000)?;
    /// println!("Surface destination rectangle: {}", surface.dest_rect);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_surface(&mut self, id: u32) -> Result<IviSurface> {
        let result = self.send_request("get_surface", json!({ "id": id }))?;

        // Parse the result as a surface
        let surface: IviSurface = serde_json::from_value(result).map_err(|e| {
            IviError::DeserializationError(format!("Failed to parse surface: {}", e))
        })?;

        Ok(surface)
    }

    /// Sets the source rectangle of a surface (which part of the application buffer to display).
    ///
    /// # Arguments
    ///
    /// * `id` - The surface ID to modify
    /// * `x` - The X coordinate in the source buffer
    /// * `y` - The Y coordinate in the source buffer
    /// * `width` - The width of the source rectangle in pixels
    /// * `height` - The height of the source rectangle in pixels
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The surface ID does not exist
    /// - Communication with the controller fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.set_surface_source_rectangle(1000, 0, 0, 1920, 1080, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_source_rectangle(
        &mut self,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        auto_commit: bool,
    ) -> Result<()> {
        use serde_json::json;

        let value = json!({
            "id": id,
            "x": x,
            "y": y,
            "width": width,
            "height": height,
            "auto_commit": auto_commit
        });

        self.send_request("set_surface_source_rectangle", value)?;
        Ok(())
    }

    /// Sets the destination rectangle of a surface (where and at what size to display on screen).
    ///
    /// # Arguments
    ///
    /// * `id` - The surface ID to modify
    /// * `x` - The X coordinate on screen
    /// * `y` - The Y coordinate on screen
    /// * `width` - The width on screen in pixels
    /// * `height` - The height on screen in pixels
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The surface ID does not exist
    /// - Communication with the controller fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.set_surface_destination_rectangle(1000, 100, 200, 1280, 720, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_destination_rectangle(
        &mut self,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        auto_commit: bool,
    ) -> Result<()> {
        let value = json!({ "id": id, "x": x, "y": y, "width": width, "height": height, "auto_commit": auto_commit });

        self.send_request("set_surface_destination_rectangle", value)
            .map(|_| ())
    }

    /// Sets the visibility of a surface.
    ///
    /// # Arguments
    ///
    /// * `id` - The surface ID to modify
    /// * `visible` - Whether the surface should be visible (true) or hidden (false)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The surface ID does not exist
    /// - Communication with the controller fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.set_surface_visibility(1000, true, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_visibility(
        &mut self,
        id: u32,
        visible: bool,
        auto_commit: bool,
    ) -> Result<()> {
        let value = json!({ "id": id, "visible": visible , "auto_commit": auto_commit});

        self.send_request("set_surface_visibility", value)
            .map(|_| ())
    }

    /// Sets the opacity of a surface.
    ///
    /// # Arguments
    ///
    /// * `id` - The surface ID to modify
    /// * `opacity` - The opacity value (0.0 = fully transparent, 1.0 = fully opaque)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The surface ID does not exist
    /// - The opacity value is outside the range 0.0 to 1.0
    /// - Communication with the controller fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.set_surface_opacity(1000, 0.75, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_opacity(&mut self, id: u32, opacity: f32, auto_commit: bool) -> Result<()> {
        let value = json!({ "id": id, "opacity": opacity, "auto_commit": auto_commit });

        self.send_request("set_surface_opacity", value).map(|_| ())
    }

    /// Sets the z-order (stacking order) of a surface.
    ///
    /// # Arguments
    ///
    /// * `id` - The surface ID to modify
    /// * `z_order` - The z-order value (higher values appear on top)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The surface ID does not exist
    /// - Communication with the controller fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.set_surface_z_order(1000, 10, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_z_order(&mut self, id: u32, z_order: i32, auto_commit: bool) -> Result<()> {
        let value = json!({ "id": id, "z_order": z_order , "auto_commit": auto_commit });
        self.send_request("set_surface_z_order", value).map(|_| ())
    }

    /// Sets the input focus to a specific surface.
    ///
    /// # Arguments
    ///
    /// * `id` - The surface ID to receive focus
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The surface ID does not exist
    /// - Communication with the controller fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.set_surface_focus(1000, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_focus(&mut self, id: u32, auto_commit: bool) -> Result<()> {
        let value = json!({ "id": id , "auto_commit": auto_commit });
        self.send_request("set_surface_focus", value).map(|_| ())
    }

    /// Lists all available layers in the IVI compositor.
    ///
    /// # Returns
    ///
    /// Returns a vector of `Layer` objects representing all layers currently
    /// managed by the IVI controller.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Communication with the controller fails
    /// - The response cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// let layers = client.list_layers()?;
    /// for layer in layers {
    ///     println!("Layer ID: {}", layer.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_layers(&mut self) -> Result<Vec<IviLayer>> {
        let result = self.send_request("list_layers", json!({}))?;

        // Extract the "layers" array from the result object
        let layers: Vec<IviLayer> = serde_json::from_value(
            result
                .get("layers")
                .ok_or_else(|| {
                    IviError::DeserializationError("Missing 'layers' field in response".to_string())
                })?
                .clone(),
        )
        .map_err(|e| IviError::DeserializationError(format!("Failed to parse layers: {}", e)))?;

        Ok(layers)
    }

    /// Gets detailed properties of a specific layer.
    ///
    /// # Arguments
    ///
    /// * `id` - The layer ID to query
    ///
    /// # Returns
    ///
    /// Returns a `Layer` object containing all properties of the specified layer.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The layer ID does not exist
    /// - Communication with the controller fails
    /// - The response cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// let layer = client.get_layer(2000)?;
    /// println!("Layer visibility: {}", layer.visibility);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_layer(&mut self, id: u32) -> Result<IviLayer> {
        let result = self.send_request("get_layer", json!({ "id": id }))?;

        // Parse the result as a layer
        let layer: IviLayer = serde_json::from_value(result)
            .map_err(|e| IviError::DeserializationError(format!("Failed to parse layer: {}", e)))?;

        Ok(layer)
    }

    /// Creates a new layer in the IVI compositor.
    ///
    /// # Arguments
    ///
    /// * `id` - The desired layer ID
    /// * `width` - The width of the layer in pixels
    /// * `height` - The height of the layer in pixels
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The layer ID already exists
    /// - Communication with the controller fails
    /// - Invalid parameter
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.create_layer(2000, 1920, 1080, true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_layer(
        &mut self,
        id: u32,
        width: i32,
        height: i32,
        auto_commit: bool,
    ) -> Result<IviRequestResult> {
        let value =
            json!({ "id": id, "width": width, "height": height , "auto_commit": auto_commit });
        let result = self.send_request("create_layer", value)?;

        let id = result
            .get("id")
            .ok_or_else(|| {
                IviError::DeserializationError("Missing 'id' field in response".to_string())
            })
            .and_then(|value| {
                value.as_u64().map(|v| v as u32).ok_or_else(|| {
                    IviError::DeserializationError(
                        "Invalid 'id' field type in response".to_string(),
                    )
                })
            })?;

        Ok(IviRequestResult::CreateLayer(id))
    }

    /// Destroys an existing layer in the IVI compositor.
    ///
    /// # Arguments
    ///
    /// * `id` - The layer ID to destroy
    /// * `auto_commit` - If true, automatically commits the changes
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The layer ID does not exist
    /// - Communication with the controller fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.destroy_layer(2000, true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn destroy_layer(&mut self, id: u32, auto_commit: bool) -> Result<()> {
        self.send_request(
            "destroy_layer",
            json!({ "id": id, "auto_commit": auto_commit }),
        )?;
        Ok(())
    }

    /// Sets the source rectangle of a layer.
    ///
    /// # Arguments
    ///
    /// * `id` - The layer ID to modify
    /// * `x` - The X coordinate in the source
    /// * `y` - The Y coordinate in the source
    /// * `width` - The width of the source rectangle in pixels
    /// * `height` - The height of the source rectangle in pixels
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The layer ID does not exist
    /// - Communication with the controller fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.set_layer_source_rectangle(2000, 0, 0, 1920, 1080, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_layer_source_rectangle(
        &mut self,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        auto_commit: bool,
    ) -> Result<()> {
        let value = json!({ "id": id, "x": x, "y": y, "width": width, "height": height, "auto_commit": auto_commit });

        self.send_request("set_layer_source_rectangle", value)
            .map(|_| ())
    }

    /// Sets the destination rectangle of a layer.
    ///
    /// # Arguments
    ///
    /// * `id` - The layer ID to modify
    /// * `x` - The X coordinate on screen
    /// * `y` - The Y coordinate on screen
    /// * `width` - The width on screen in pixels
    /// * `height` - The height on screen in pixels
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The layer ID does not exist
    /// - Communication with the controller fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.set_layer_destination_rectangle(2000, 0, 0, 1920, 1080, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_layer_destination_rectangle(
        &mut self,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        auto_commit: bool,
    ) -> Result<()> {
        let value = json!({ "id": id, "x": x, "y": y, "width": width, "height": height, "auto_commit": auto_commit });

        self.send_request("set_layer_destination_rectangle", value)?;
        Ok(())
    }

    /// Sets the visibility of a layer.
    ///
    /// # Arguments
    ///
    /// * `id` - The layer ID to modify
    /// * `visible` - Whether the layer should be visible (true) or hidden (false)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The layer ID does not exist
    /// - Communication with the controller fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.set_layer_visibility(2000, true, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_layer_visibility(
        &mut self,
        id: u32,
        visible: bool,
        auto_commit: bool,
    ) -> Result<()> {
        let value = json!({ "id": id, "visible": visible , "auto_commit": auto_commit });

        self.send_request("set_layer_visibility", value).map(|_| ())
    }

    /// Sets the opacity of a layer.
    ///
    /// # Arguments
    ///
    /// * `id` - The layer ID to modify
    /// * `opacity` - The opacity value (0.0 = fully transparent, 1.0 = fully opaque)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The layer ID does not exist
    /// - The opacity value is outside the range 0.0 to 1.0
    /// - Communication with the controller fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// client.set_layer_opacity(2000, 0.75, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_layer_opacity(&mut self, id: u32, opacity: f32, auto_commit: bool) -> Result<()> {
        let value = json!({ "id": id, "opacity": opacity, "auto_commit": auto_commit });

        self.send_request("set_layer_opacity", value).map(|_| ())
    }

    /// Lists all available screens (outputs) in the IVI compositor.
    ///
    /// # Returns
    ///
    /// Returns a vector of `IviScreen` structures containing information about each screen.
    ///
    /// # Errors
    ///
    /// Returns an error if communication with the controller fails.
    pub fn list_screens(&mut self) -> Result<Vec<IviScreen>> {
        let response = self.send_request("list_screens", json!({}))?;
        let screens: Vec<IviScreen> = serde_json::from_value(response["screens"].clone())
            .map_err(|e| IviError::DeserializationError(e.to_string()))?;
        Ok(screens)
    }

    /// Gets information about a specific screen by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The screen name (e.g., "HDMI-A-1")
    ///
    /// # Returns
    ///
    /// Returns an `IviScreen` structure with screen information.
    ///
    /// # Errors
    ///
    /// Returns an error if the screen is not found or communication fails.
    pub fn get_screen(&mut self, name: &str) -> Result<IviScreen> {
        let response = self.send_request("get_screen", json!({ "name": name }))?;
        let screen: IviScreen = serde_json::from_value(response)
            .map_err(|e| IviError::DeserializationError(e.to_string()))?;
        Ok(screen)
    }

    /// Gets the list of layer IDs assigned to a screen.
    ///
    /// # Arguments
    ///
    /// * `screen_name` - The screen name
    ///
    /// # Returns
    ///
    /// Returns a vector of layer IDs in render order (first = topmost).
    ///
    /// # Errors
    ///
    /// Returns an error if the screen is not found or communication fails.
    pub fn get_screen_layers(&mut self, screen_name: &str) -> Result<Vec<u32>> {
        let response =
            self.send_request("get_screen_layers", json!({ "screen_name": screen_name }))?;
        let layer_ids: Vec<u32> = serde_json::from_value(response["layer_ids"].clone())
            .map_err(|e| IviError::DeserializationError(e.to_string()))?;
        Ok(layer_ids)
    }

    /// Gets the list of screen names that a layer is assigned to.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer ID
    ///
    /// # Returns
    ///
    /// Returns a vector of screen names.
    ///
    /// # Errors
    ///
    /// Returns an error if the layer is not found or communication fails.
    pub fn get_layer_screens(&mut self, layer_id: u32) -> Result<Vec<String>> {
        let response = self.send_request("get_layer_screens", json!({ "layer_id": layer_id }))?;
        let screen_names: Vec<String> = serde_json::from_value(response["screen_names"].clone())
            .map_err(|e| IviError::DeserializationError(e.to_string()))?;
        Ok(screen_names)
    }

    /// Adds layers to a screen, setting the render order.
    ///
    /// This replaces all existing layers on the screen with the specified layers.
    /// The order in the array determines the z-order (first element = topmost layer).
    ///
    /// # Arguments
    ///
    /// * `screen_name` - The screen name
    /// * `layer_ids` - Vector of layer IDs in desired render order
    /// * `auto_commit` - If true, automatically commits the changes
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the screen or any layer is not found, or communication fails.
    pub fn add_layers_to_screen(
        &mut self,
        screen_name: &str,
        layer_ids: &[u32],
        auto_commit: bool,
    ) -> Result<()> {
        self.send_request(
            "add_layers_to_screen",
            json!({
                "screen_name": screen_name,
                "layer_ids": layer_ids,
                "auto_commit": auto_commit
            }),
        )
        .map(|_| ())
    }

    /// Removes a layer from a screen.
    ///
    /// # Arguments
    ///
    /// * `screen_name` - The screen name
    /// * `layer_id` - The layer ID to remove
    /// * `auto_commit` - If true, automatically commits the changes
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the screen or layer is not found, or communication fails.
    pub fn remove_layer_from_screen(
        &mut self,
        screen_name: &str,
        layer_id: u32,
        auto_commit: bool,
    ) -> Result<()> {
        self.send_request(
            "remove_layer_from_screen",
            json!({
                "screen_name": screen_name,
                "layer_id": layer_id,
                "auto_commit": auto_commit
            }),
        )
        .map(|_| ())
    }

    /// Sets the complete list of surfaces on a layer, replacing any existing surfaces.
    ///
    /// The z-order is determined by the position in the array:
    /// - First surface ID = bottommost (rendered first, behind others)
    /// - Last surface ID = topmost (rendered last, in front of others)
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The ID of the layer
    /// * `surface_ids` - Array of surface IDs in z-order (first=bottom, last=top)
    /// * `auto_commit` - If true, automatically commits the changes
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the layer or any surface is not found, or communication fails.
    pub fn set_surfaces_on_layer(
        &mut self,
        layer_id: u32,
        surface_ids: &[u32],
        auto_commit: bool,
    ) -> Result<()> {
        self.send_request(
            "set_layer_surfaces",
            json!({
                "layer_id": layer_id,
                "surface_ids": surface_ids,
                "auto_commit": auto_commit
            }),
        )
        .map(|_| ())
    }

    /// Adds a single surface to a layer as the topmost surface.
    ///
    /// The surface will be rendered on top of all other surfaces currently on the layer.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The ID of the layer
    /// * `surface_id` - The ID of the surface to add
    /// * `auto_commit` - If true, automatically commits the changes
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the layer or surface is not found, or communication fails.
    pub fn add_surface_to_layer(
        &mut self,
        layer_id: u32,
        surface_id: u32,
        auto_commit: bool,
    ) -> Result<()> {
        self.send_request(
            "add_surface_to_layer",
            json!({
                "layer_id": layer_id,
                "surface_id": surface_id,
                "auto_commit": auto_commit
            }),
        )
        .map(|_| ())
    }

    /// Removes a surface from a layer.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The ID of the layer
    /// * `surface_id` - The ID of the surface to remove
    /// * `auto_commit` - If true, automatically commits the changes
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the layer or surface is not found, or communication fails.
    pub fn remove_surface_from_layer(
        &mut self,
        layer_id: u32,
        surface_id: u32,
        auto_commit: bool,
    ) -> Result<()> {
        self.send_request(
            "remove_surface_from_layer",
            json!({
                "layer_id": layer_id,
                "surface_id": surface_id,
                "auto_commit": auto_commit
            }),
        )
        .map(|_| ())
    }

    /// Gets the list of surface IDs currently assigned to a layer.
    ///
    /// Returns surfaces in z-order:
    /// - First ID = bottommost (rendered first, behind others)
    /// - Last ID = topmost (rendered last, in front of others)
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The ID of the layer
    ///
    /// # Returns
    ///
    /// Returns a vector of surface IDs in z-order.
    ///
    /// # Errors
    ///
    /// Returns an error if the layer is not found or communication fails.
    pub fn get_layer_surfaces(&mut self, layer_id: u32) -> Result<Vec<u32>> {
        let response = self.send_request("get_layer_surfaces", json!({ "layer_id": layer_id }))?;
        let surface_ids: Vec<u32> = serde_json::from_value(response["surface_ids"].clone())
            .map_err(|e| IviError::DeserializationError(e.to_string()))?;
        Ok(surface_ids)
    }

    /// Commits all pending changes to the IVI compositor atomically.
    ///
    /// This method applies all pending surface and layer modifications in a single
    /// atomic operation, ensuring that changes are applied simultaneously without
    /// visual artifacts or tearing.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful commit, or an error if the commit operation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Communication with the controller fails
    /// - The commit operation fails on the server side
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    ///
    /// // Make multiple changes
    /// client.set_surface_destination_rectangle(1000, 100, 200, 1920, 1080, false)?;
    /// client.set_surface_visibility(1000, true, false)?;
    ///
    /// // Commit all changes atomically
    /// client.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn commit(&mut self) -> Result<()> {
        self.send_request("commit", json!({})).map(|_| ())
    }
}

// ============================================================================
// NotificationListener
// ============================================================================

/// Callback type for notification events.
pub type NotificationCallback = Arc<dyn Fn(&Notification) + Send + Sync + 'static>;

/// Listens for event notifications from the IVI controller.
///
/// Opens its own dedicated socket connection so that notifications are not
/// mixed with RPC responses on the shared `IviClient` connection.
///
/// # Example
///
/// ```no_run
/// use ivi_client::{NotificationListener, EventType};
///
/// # fn main() -> ivi_client::Result<()> {
/// let mut listener = NotificationListener::new(None)?;
///
/// listener.on(EventType::SurfaceCreated, |notif| {
///     println!("Surface created: {}", notif.params["surface_id"]);
/// });
///
/// listener.on_all(|notif| {
///     println!("Event: {:?}", notif.event_type);
/// });
///
/// listener.start(&[EventType::SurfaceCreated, EventType::VisibilityChanged])?;
/// // ... callbacks fire in background thread ...
/// listener.stop();
/// # Ok(())
/// # }
/// ```
pub struct NotificationListener {
    transport: Arc<Mutex<Box<dyn IviClientTransport>>>,
    request_id: AtomicU64,
    callbacks: Arc<Mutex<HashMap<EventType, Vec<NotificationCallback>>>>,
    catch_all_callbacks: Arc<Mutex<Vec<NotificationCallback>>>,
    stop_flag: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl NotificationListener {
    /// Create a new listener connected to the IVI controller.
    pub fn new(remote: Option<&str>) -> Result<Self> {
        #[cfg(not(feature = "enable-ipcon"))]
        let transport: Box<dyn IviClientTransport> =
            Box::new(UnixDomainIviClient::connect(remote)?);

        #[cfg(feature = "enable-ipcon")]
        let transport: Box<dyn IviClientTransport> =
            Box::new(IpconIviClient::ipcon_connect(None, remote)?);

        Ok(Self {
            transport: Arc::new(Mutex::new(transport)),
            request_id: AtomicU64::new(1),
            callbacks: Arc::new(Mutex::new(HashMap::new())),
            catch_all_callbacks: Arc::new(Mutex::new(Vec::new())),
            stop_flag: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        })
    }

    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    fn send_rpc(&self, method: &str, params: Value) -> Result<Value> {
        let request_id = self.next_request_id();
        let request = JsonRpcRequest::new(request_id, method, params);
        let request_json = serde_json::to_vec(&request)
            .map_err(|e| IviError::SerializationError(e.to_string()))?;

        let mut transport = self.transport.lock().unwrap();
        transport.send_request(&request_json)?;
        let response_buf = transport.receive_response()?;

        let response: JsonRpcResponse = serde_json::from_slice(&response_buf)
            .map_err(|e| IviError::DeserializationError(e.to_string()))?;

        if let Some(error) = response.error {
            return Err(IviError::RequestFailed {
                code: error.code,
                message: error.message,
            });
        }

        response.result.ok_or_else(|| {
            IviError::DeserializationError("Response missing both result and error".to_string())
        })
    }

    /// Register a callback for a specific event type.
    /// Multiple callbacks per event type are allowed.
    pub fn on<F>(&mut self, event_type: EventType, callback: F)
    where
        F: Fn(&Notification) + Send + Sync + 'static,
    {
        self.callbacks
            .lock()
            .unwrap()
            .entry(event_type)
            .or_default()
            .push(Arc::new(callback));
    }

    /// Register a catch-all callback invoked for every received event.
    pub fn on_all<F>(&mut self, callback: F)
    where
        F: Fn(&Notification) + Send + Sync + 'static,
    {
        self.catch_all_callbacks
            .lock()
            .unwrap()
            .push(Arc::new(callback));
    }

    /// Subscribe to the given event types on the server and start the
    /// background reader thread. Callbacks registered with `on`/`on_all`
    /// will fire from this thread.
    pub fn start(&mut self, event_types: &[EventType]) -> Result<()> {
        self.send_rpc("subscribe", json!({ "event_types": event_types }))?;

        self.stop_flag.store(false, Ordering::Relaxed);

        let transport = Arc::clone(&self.transport);
        let callbacks = Arc::clone(&self.callbacks);
        let catch_all_callbacks = Arc::clone(&self.catch_all_callbacks);
        let stop_flag = Arc::clone(&self.stop_flag);

        self.thread_handle = Some(std::thread::spawn(move || {
            loop {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }

                let frame = {
                    let mut t = transport.lock().unwrap();
                    let _ = t.set_read_timeout(Some(Duration::from_millis(100)));
                    let r = t.receive_response();
                    let _ = t.set_read_timeout(None);
                    r
                };

                match frame {
                    Ok(bytes) => match Notification::try_from_frame(&bytes) {
                        Ok(Some(notif)) => {
                            // Clone Arc refs before dispatching to avoid holding the lock
                            // while callbacks run.
                            let per_type: Vec<NotificationCallback> = callbacks
                                .lock()
                                .unwrap()
                                .get(&notif.event_type)
                                .cloned()
                                .unwrap_or_default();
                            for cb in &per_type {
                                cb(&notif);
                            }
                            let all: Vec<NotificationCallback> =
                                catch_all_callbacks.lock().unwrap().clone();
                            for cb in &all {
                                cb(&notif);
                            }
                        }
                        Ok(None) => {} // stray RPC response, skip
                        Err(_) => {}   // parse error, skip
                    },
                    Err(IviError::IoError(ref e))
                        if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut =>
                    {
                        continue; // timeout — check stop flag and retry
                    }
                    Err(_) => break, // real error — exit thread
                }
            }
        }));

        Ok(())
    }

    /// Signal the background thread to stop and wait for it to finish.
    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for NotificationListener {
    fn drop(&mut self) {
        self.stop();
        let mut t = self.transport.lock().unwrap();
        let _ = t.disconnect();
    }
}
