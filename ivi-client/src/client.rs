//! IVI Client implementation for connecting to and communicating with the IVI controller.
//!
//! This module provides the main `IviClient` struct that manages the connection to the
//! Weston IVI controller via UNIX domain sockets and handles JSON-RPC communication.

use crate::error::{IviError, Result};
use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::Value;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicU64, Ordering};
use weston_ivi_controller::rpc::framing::{write_frame, FrameReadResult, FrameReader};

/// Default socket path for the IVI controller
pub const DEFAULT_SOCKET_PATH: &str = "/tmp/weston-ivi-controller.sock";

/// Client for communicating with the Weston IVI controller.
///
/// The `IviClient` maintains a connection to the IVI controller over a UNIX domain socket
/// and provides methods for sending JSON-RPC requests and receiving responses.
///
/// # Example
///
/// ```no_run
/// use ivi_client::IviClient;
///
/// # fn main() -> ivi_client::Result<()> {
/// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
/// // Use the client to interact with the IVI controller
/// client.disconnect()?;
/// # Ok(())
/// # }
/// ```
pub struct IviClient {
    /// UNIX domain socket connection to the IVI controller
    socket: UnixStream,

    /// Frame reader for length-prefixed protocol
    frame_reader: FrameReader,

    /// Atomic counter for generating unique request IDs
    request_id: AtomicU64,
}

impl IviClient {
    /// Connects to the IVI controller at the specified socket path.
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Path to the UNIX domain socket (e.g., "/tmp/weston-ivi-controller.sock")
    ///
    /// # Returns
    ///
    /// Returns a connected `IviClient` instance on success, or an error if the connection fails.
    ///
    /// # Errors
    ///
    /// Returns `IviError::ConnectionFailed` if:
    /// - The socket file does not exist
    /// - Permission is denied
    /// - The connection is refused
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn connect(socket_path: &str) -> Result<Self> {
        let socket = UnixStream::connect(socket_path)
            .map_err(|e| IviError::ConnectionFailed(format!("{}: {}", socket_path, e)))?;

        Ok(Self {
            socket,
            frame_reader: FrameReader::new(),
            request_id: AtomicU64::new(1),
        })
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
    /// let client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// client.disconnect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn disconnect(self) -> Result<()> {
        // The socket will be automatically closed when it goes out of scope
        // We explicitly drop it here for clarity
        drop(self.socket);
        Ok(())
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// let surfaces = client.list_surfaces()?;
    /// for surface in surfaces {
    ///     println!("Surface ID: {}", surface.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_surfaces(&mut self) -> Result<Vec<crate::types::Surface>> {
        use serde_json::json;

        let result = self.send_request("list_surfaces", json!({}))?;

        // Extract the "surfaces" array from the result object
        let surfaces: Vec<crate::types::Surface> = serde_json::from_value(
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// let surface = client.get_surface(1000)?;
    /// println!("Surface destination position: {}", surface.dest_position);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_surface(&mut self, id: u32) -> Result<crate::types::Surface> {
        use serde_json::json;

        let result = self.send_request("get_surface", json!({ "id": id }))?;

        // Parse the result as a surface
        let surface: crate::types::Surface = serde_json::from_value(result).map_err(|e| {
            IviError::DeserializationError(format!("Failed to parse surface: {}", e))
        })?;

        Ok(surface)
    }

    /// Sets the position of a surface.
    ///
    /// # Arguments
    ///
    /// * `id` - The surface ID to modify
    /// * `x` - The x-coordinate of the surface position
    /// * `y` - The y-coordinate of the surface position
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// client.set_surface_position(1000, 100, 200)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_position(&mut self, id: u32, x: i32, y: i32) -> Result<()> {
        use serde_json::json;

        self.send_request("set_surface_position", json!({ "id": id, "x": x, "y": y }))?;
        Ok(())
    }

    /// Sets the size of a surface.
    ///
    /// # Arguments
    ///
    /// * `id` - The surface ID to modify
    /// * `width` - The width of the surface in pixels
    /// * `height` - The height of the surface in pixels
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// client.set_surface_size(1000, 1920, 1080)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_size(&mut self, id: u32, width: u32, height: u32) -> Result<()> {
        use serde_json::json;

        self.send_request(
            "set_surface_size",
            json!({ "id": id, "width": width, "height": height }),
        )?;
        Ok(())
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// client.set_surface_visibility(1000, true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_visibility(&mut self, id: u32, visible: bool) -> Result<()> {
        use serde_json::json;

        self.send_request(
            "set_surface_visibility",
            json!({ "id": id, "visible": visible }),
        )?;
        Ok(())
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// client.set_surface_opacity(1000, 0.75)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_opacity(&mut self, id: u32, opacity: f32) -> Result<()> {
        use serde_json::json;

        self.send_request(
            "set_surface_opacity",
            json!({ "id": id, "opacity": opacity }),
        )?;
        Ok(())
    }

    /// Sets the orientation of a surface.
    ///
    /// # Arguments
    ///
    /// * `id` - The surface ID to modify
    /// * `orientation` - The orientation to apply (Normal, Rotate90, Rotate180, or Rotate270)
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
    /// use ivi_client::{IviClient, Orientation};
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// client.set_surface_orientation(1000, Orientation::Rotate90)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_orientation(
        &mut self,
        id: u32,
        orientation: crate::types::Orientation,
    ) -> Result<()> {
        use serde_json::json;

        self.send_request(
            "set_surface_orientation",
            json!({ "id": id, "orientation": orientation }),
        )?;
        Ok(())
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// client.set_surface_z_order(1000, 10)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_z_order(&mut self, id: u32, z_order: i32) -> Result<()> {
        use serde_json::json;

        self.send_request(
            "set_surface_z_order",
            json!({ "id": id, "z_order": z_order }),
        )?;
        Ok(())
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// client.set_surface_focus(1000)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_surface_focus(&mut self, id: u32) -> Result<()> {
        use serde_json::json;

        self.send_request("set_surface_focus", json!({ "id": id }))?;
        Ok(())
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// let layers = client.list_layers()?;
    /// for layer in layers {
    ///     println!("Layer ID: {}", layer.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_layers(&mut self) -> Result<Vec<crate::types::Layer>> {
        use serde_json::json;

        let result = self.send_request("list_layers", json!({}))?;

        // Extract the "layers" array from the result object
        let layers: Vec<crate::types::Layer> = serde_json::from_value(
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// let layer = client.get_layer(2000)?;
    /// println!("Layer visibility: {}", layer.visibility);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_layer(&mut self, id: u32) -> Result<crate::types::Layer> {
        use serde_json::json;

        let result = self.send_request("get_layer", json!({ "id": id }))?;

        // Parse the result as a layer
        let layer: crate::types::Layer = serde_json::from_value(result)
            .map_err(|e| IviError::DeserializationError(format!("Failed to parse layer: {}", e)))?;

        Ok(layer)
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// client.set_layer_visibility(2000, true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_layer_visibility(&mut self, id: u32, visible: bool) -> Result<()> {
        use serde_json::json;

        self.send_request(
            "set_layer_visibility",
            json!({ "id": id, "visible": visible }),
        )?;
        Ok(())
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    /// client.set_layer_opacity(2000, 0.75)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_layer_opacity(&mut self, id: u32, opacity: f32) -> Result<()> {
        use serde_json::json;

        self.send_request("set_layer_opacity", json!({ "id": id, "opacity": opacity }))?;
        Ok(())
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
    /// let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    ///
    /// // Make multiple changes
    /// client.set_surface_position(1000, 100, 200)?;
    /// client.set_surface_size(1000, 1920, 1080)?;
    /// client.set_surface_visibility(1000, true)?;
    ///
    /// // Commit all changes atomically
    /// client.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn commit(&mut self) -> Result<()> {
        use serde_json::json;

        self.send_request("commit", json!({}))?;
        Ok(())
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

        // Serialize request to JSON (as bytes for length-prefix protocol)
        let request_json = serde_json::to_vec(&request)
            .map_err(|e| IviError::SerializationError(e.to_string()))?;

        // Send request using shared framing module
        write_frame(&mut self.socket, &request_json).map_err(|e| IviError::IoError(e))?;

        // Read response using shared framing module
        let response_buf = loop {
            match self.frame_reader.read_frame(&mut self.socket)? {
                FrameReadResult::Complete(msg) => break msg,
                FrameReadResult::NeedMore => {
                    // Partial read, continue reading
                    std::thread::yield_now();
                    continue;
                }
                FrameReadResult::Eof => {
                    return Err(IviError::IoError(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Connection closed while reading response",
                    )));
                }
            }
        };

        // Deserialize response
        let response: JsonRpcResponse = serde_json::from_slice(&response_buf)
            .map_err(|e| IviError::DeserializationError(e.to_string()))?;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_socket_path() {
        assert_eq!(DEFAULT_SOCKET_PATH, "/tmp/weston-ivi-controller.sock");
    }

    #[test]
    fn test_request_id_generation() {
        let client = IviClient {
            socket: UnixStream::pair().unwrap().0,
            frame_reader: FrameReader::new(),
            request_id: AtomicU64::new(1),
        };

        assert_eq!(client.next_request_id(), 1);
        assert_eq!(client.next_request_id(), 2);
        assert_eq!(client.next_request_id(), 3);
    }

    #[test]
    fn test_request_id_uniqueness() {
        let client = IviClient {
            socket: UnixStream::pair().unwrap().0,
            frame_reader: FrameReader::new(),
            request_id: AtomicU64::new(100),
        };

        let mut ids = std::collections::HashSet::new();
        for _ in 0..1000 {
            let id = client.next_request_id();
            assert!(ids.insert(id), "Duplicate request ID generated: {}", id);
        }
    }

    #[test]
    fn test_connect_invalid_path() {
        let result = IviClient::connect("/nonexistent/path/to/socket.sock");
        assert!(result.is_err());

        if let Err(IviError::ConnectionFailed(msg)) = result {
            assert!(msg.contains("/nonexistent/path/to/socket.sock"));
        } else {
            panic!("Expected ConnectionFailed error");
        }
    }
}
