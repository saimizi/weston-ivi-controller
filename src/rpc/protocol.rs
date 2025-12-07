// RPC protocol definitions

use crate::controller::state::Orientation;
use serde::{Deserialize, Serialize};

/// Event types for client subscriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    // Surface events
    SurfaceCreated,
    SurfaceDestroyed,
    GeometryChanged,
    VisibilityChanged,
    OpacityChanged,
    OrientationChanged,
    ZOrderChanged,
    FocusChanged,

    // Layer events
    LayerCreated,
    LayerDestroyed,
    LayerVisibilityChanged,
    LayerOpacityChanged,
}

/// RPC request structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RpcRequest {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

impl RpcRequest {
    /// Create a new RPC request
    pub fn new(id: u64, method: String, params: serde_json::Value) -> Self {
        Self { id, method, params }
    }

    /// Parse an RPC request from JSON bytes
    pub fn from_json(data: &[u8]) -> Result<Self, RpcError> {
        serde_json::from_slice(data).map_err(|e| RpcError {
            code: -32700, // Parse error
            message: format!("Failed to parse request: {}", e),
        })
    }

    /// Serialize an RPC request to JSON bytes
    pub fn to_json(&self) -> Result<Vec<u8>, RpcError> {
        serde_json::to_vec(self).map_err(|e| RpcError {
            code: -32603, // Internal error
            message: format!("Failed to serialize request: {}", e),
        })
    }
}

/// RPC response structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RpcResponse {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl RpcResponse {
    /// Create a successful response
    pub fn success(id: u64, result: serde_json::Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: u64, error: RpcError) -> Self {
        Self {
            id,
            result: None,
            error: Some(error),
        }
    }

    /// Parse an RPC response from JSON bytes
    pub fn from_json(data: &[u8]) -> Result<Self, RpcError> {
        serde_json::from_slice(data).map_err(|e| RpcError {
            code: -32700, // Parse error
            message: format!("Failed to parse response: {}", e),
        })
    }

    /// Serialize an RPC response to JSON bytes
    pub fn to_json(&self) -> Result<Vec<u8>, RpcError> {
        serde_json::to_vec(self).map_err(|e| RpcError {
            code: -32603, // Internal error
            message: format!("Failed to serialize response: {}", e),
        })
    }
}

/// RPC notification structure (JSON-RPC 2.0 notification - no id field)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RpcNotification {
    pub method: String,
    pub params: serde_json::Value,
}

impl RpcNotification {
    /// Create a new RPC notification
    pub fn new(method: String, params: serde_json::Value) -> Self {
        Self { method, params }
    }

    /// Serialize to JSON bytes
    pub fn to_json(&self) -> Result<Vec<u8>, RpcError> {
        serde_json::to_vec(self).map_err(|e| RpcError {
            code: -32603, // Internal error
            message: format!("Failed to serialize notification: {}", e),
        })
    }
}

/// RPC error structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RPC error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for RpcError {}

impl RpcError {
    /// Create a new RPC error
    pub fn new(code: i32, message: String) -> Self {
        Self { code, message }
    }

    /// Create an invalid parameters error
    pub fn invalid_params(message: String) -> Self {
        Self {
            code: -32602,
            message,
        }
    }

    /// Create a method not found error
    pub fn method_not_found(method: String) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method),
        }
    }

    /// Create an internal error
    pub fn internal_error(message: String) -> Self {
        Self {
            code: -32603,
            message,
        }
    }

    /// Create a surface not found error
    pub fn surface_not_found(id: u32) -> Self {
        Self {
            code: -32000,
            message: format!("Surface not found: {}", id),
        }
    }

    /// Create a layer not found error
    pub fn layer_not_found(id: u32) -> Self {
        Self {
            code: -32000,
            message: format!("Layer not found: {}", id),
        }
    }
}

/// RPC method enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum RpcMethod {
    // Surface methods
    ListSurfaces,
    GetSurface { id: u32 },
    SetPosition { id: u32, x: i32, y: i32 },
    SetSize { id: u32, width: i32, height: i32 },
    SetVisibility { id: u32, visible: bool },
    SetOpacity { id: u32, opacity: f32 },
    SetOrientation { id: u32, orientation: Orientation },
    SetZOrder { id: u32, z_order: i32 },
    SetFocus { id: u32 },
    Commit,

    // Subscription methods
    Subscribe { event_types: Vec<EventType> },
    Unsubscribe { event_types: Vec<EventType> },
    ListSubscriptions,

    // Layer methods
    ListLayers,
    GetLayer { id: u32 },
    SetLayerVisibility { id: u32, visible: bool },
    SetLayerOpacity { id: u32, opacity: f32 },
}

impl RpcMethod {
    /// Parse an RPC method from a request
    pub fn from_request(request: &RpcRequest) -> Result<Self, RpcError> {
        match request.method.as_str() {
            "list_surfaces" => Ok(RpcMethod::ListSurfaces),

            "get_surface" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })?;
                Ok(RpcMethod::GetSurface { id: id as u32 })
            }

            "set_position" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })?;
                let x = request
                    .params
                    .get("x")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'x' parameter".to_string())
                    })?;
                let y = request
                    .params
                    .get("y")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'y' parameter".to_string())
                    })?;
                Ok(RpcMethod::SetPosition {
                    id: id as u32,
                    x: x as i32,
                    y: y as i32,
                })
            }

            "set_size" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })?;
                let width = request
                    .params
                    .get("width")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'width' parameter".to_string())
                    })?;
                let height = request
                    .params
                    .get("height")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'height' parameter".to_string(),
                        )
                    })?;
                Ok(RpcMethod::SetSize {
                    id: id as u32,
                    width: width as i32,
                    height: height as i32,
                })
            }

            "set_visibility" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })?;
                let visible = request
                    .params
                    .get("visible")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'visible' parameter".to_string(),
                        )
                    })?;
                Ok(RpcMethod::SetVisibility {
                    id: id as u32,
                    visible,
                })
            }

            "set_opacity" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })?;
                let opacity = request
                    .params
                    .get("opacity")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'opacity' parameter".to_string(),
                        )
                    })?;
                Ok(RpcMethod::SetOpacity {
                    id: id as u32,
                    opacity: opacity as f32,
                })
            }

            "set_orientation" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })?;
                let orientation: Orientation = serde_json::from_value(
                    request
                        .params
                        .get("orientation")
                        .ok_or_else(|| {
                            RpcError::invalid_params("Missing 'orientation' parameter".to_string())
                        })?
                        .clone(),
                )
                .map_err(|_| {
                    RpcError::invalid_params("Invalid 'orientation' parameter".to_string())
                })?;
                Ok(RpcMethod::SetOrientation {
                    id: id as u32,
                    orientation,
                })
            }

            "set_z_order" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })?;
                let z_order = request
                    .params
                    .get("z_order")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'z_order' parameter".to_string(),
                        )
                    })?;
                Ok(RpcMethod::SetZOrder {
                    id: id as u32,
                    z_order: z_order as i32,
                })
            }

            "set_focus" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })?;
                Ok(RpcMethod::SetFocus { id: id as u32 })
            }

            "commit" => Ok(RpcMethod::Commit),

            // Subscription methods
            "subscribe" => {
                let event_types: Vec<EventType> = serde_json::from_value(
                    request
                        .params
                        .get("event_types")
                        .ok_or_else(|| {
                            RpcError::invalid_params("Missing 'event_types' parameter".to_string())
                        })?
                        .clone(),
                )
                .map_err(|_| {
                    RpcError::invalid_params("Invalid 'event_types' parameter".to_string())
                })?;
                Ok(RpcMethod::Subscribe { event_types })
            }

            "unsubscribe" => {
                let event_types: Vec<EventType> = serde_json::from_value(
                    request
                        .params
                        .get("event_types")
                        .ok_or_else(|| {
                            RpcError::invalid_params("Missing 'event_types' parameter".to_string())
                        })?
                        .clone(),
                )
                .map_err(|_| {
                    RpcError::invalid_params("Invalid 'event_types' parameter".to_string())
                })?;
                Ok(RpcMethod::Unsubscribe { event_types })
            }

            "list_subscriptions" => Ok(RpcMethod::ListSubscriptions),

            // Layer methods
            "list_layers" => Ok(RpcMethod::ListLayers),

            "get_layer" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })?;
                Ok(RpcMethod::GetLayer { id: id as u32 })
            }

            "set_layer_visibility" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })?;
                let visible = request
                    .params
                    .get("visible")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'visible' parameter".to_string(),
                        )
                    })?;
                Ok(RpcMethod::SetLayerVisibility {
                    id: id as u32,
                    visible,
                })
            }

            "set_layer_opacity" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })?;
                let opacity = request
                    .params
                    .get("opacity")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'opacity' parameter".to_string(),
                        )
                    })?;
                Ok(RpcMethod::SetLayerOpacity {
                    id: id as u32,
                    opacity: opacity as f32,
                })
            }

            _ => Err(RpcError::method_not_found(request.method.clone())),
        }
    }
}
