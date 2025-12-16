// RPC protocol definitions

use serde::{Deserialize, Serialize};

/// Event types for client subscriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    // Surface events
    SurfaceCreated,
    SurfaceDestroyed,
    SourceGeometryChanged,
    DestinationGeometryChanged,
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
    GetSurface {
        id: u32,
    },
    SetSurfaceSourceRectangle {
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    SetSurfaceDestinationRectangle {
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    SetSurfaceVisibility {
        id: u32,
        visible: bool,
    },
    SetSurfaceOpacity {
        id: u32,
        opacity: f32,
    },
    SetSurfaceZOrder {
        id: u32,
        z_order: i32,
    },
    SetSurfaceFocus {
        id: u32,
    },
    Commit,

    // Subscription methods
    Subscribe {
        event_types: Vec<EventType>,
    },
    Unsubscribe {
        event_types: Vec<EventType>,
    },
    ListSubscriptions,

    // Layer methods
    ListLayers,
    GetLayer {
        id: u32,
    },
    CreateLayer {
        id: u32,
        width: i32,
        height: i32,
    },
    SetLayerSourceRectangle {
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    SetLayerDestinationRectangle {
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    SetLayerVisibility {
        id: u32,
        visible: bool,
    },
    SetLayerOpacity {
        id: u32,
        opacity: f32,
    },
    // Layer-surface assignment operations
    SetLayerSurfaces {
        layer_id: u32,
        surface_ids: Vec<u32>,
        auto_commit: bool,
    },
    AddSurfaceToLayer {
        layer_id: u32,
        surface_id: u32,
        auto_commit: bool,
    },
    RemoveSurfaceFromLayer {
        layer_id: u32,
        surface_id: u32,
        auto_commit: bool,
    },
    GetLayerSurfaces {
        layer_id: u32,
    },
    // Screen operations
    ListScreens,
    GetScreen {
        name: String,
    },
    GetScreenLayers {
        screen_name: String,
    },
    GetLayerScreens {
        layer_id: u32,
    },
    AddLayersToScreen {
        screen_name: String,
        layer_ids: Vec<u32>,
        auto_commit: bool,
    },
    RemoveLayerFromScreen {
        screen_name: String,
        layer_id: u32,
        auto_commit: bool,
    },
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
                    })? as u32;
                Ok(RpcMethod::GetSurface { id })
            }

            "set_surface_source_rectangle" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })? as u32;

                let x = request
                    .params
                    .get("x")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'x' parameter".to_string())
                    })? as i32;

                let y = request
                    .params
                    .get("y")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'y' parameter".to_string())
                    })? as i32;

                let width = request
                    .params
                    .get("width")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'width' parameter".to_string())
                    })? as i32;
                let height = request
                    .params
                    .get("height")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'height' parameter".to_string(),
                        )
                    })? as i32;

                Ok(RpcMethod::SetSurfaceSourceRectangle {
                    id,
                    x,
                    y,
                    width,
                    height,
                })
            }

            "set_surface_destination_rectangle" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })? as u32;
                let x = request
                    .params
                    .get("x")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'x' parameter".to_string())
                    })? as i32;
                let y = request
                    .params
                    .get("y")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'y' parameter".to_string())
                    })? as i32;
                let width = request
                    .params
                    .get("width")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'width' parameter".to_string())
                    })? as i32;
                let height = request
                    .params
                    .get("height")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'height' parameter".to_string(),
                        )
                    })? as i32;
                Ok(RpcMethod::SetSurfaceDestinationRectangle {
                    id,
                    x,
                    y,
                    width,
                    height,
                })
            }

            "set_surface_visibility" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })? as u32;
                let visible = request
                    .params
                    .get("visible")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'visible' parameter".to_string(),
                        )
                    })?;
                Ok(RpcMethod::SetSurfaceVisibility { id, visible })
            }

            "set_surface_opacity" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })? as u32;
                let opacity = request
                    .params
                    .get("opacity")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'opacity' parameter".to_string(),
                        )
                    })?;
                Ok(RpcMethod::SetSurfaceOpacity {
                    id,
                    opacity: opacity as f32,
                })
            }

            "set_surface_z_order" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })? as u32;
                let z_order = request
                    .params
                    .get("z_order")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'z_order' parameter".to_string(),
                        )
                    })? as i32;
                Ok(RpcMethod::SetSurfaceZOrder { id, z_order })
            }

            "set_surface_focus" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })?;
                Ok(RpcMethod::SetSurfaceFocus { id: id as u32 })
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

            "create_layer" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })? as u32;
                let width = request
                    .params
                    .get("width")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'width' parameter".to_string())
                    })? as i32;
                let height = request
                    .params
                    .get("height")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'height' parameter".to_string(),
                        )
                    })? as i32;
                Ok(RpcMethod::CreateLayer { id, width, height })
            }

            "get_layer" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })? as u32;
                Ok(RpcMethod::GetLayer { id })
            }

            "set_layer_source_rectangle" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })? as u32;
                let x = request
                    .params
                    .get("x")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'x' parameter".to_string())
                    })? as i32;
                let y = request
                    .params
                    .get("y")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'y' parameter".to_string())
                    })? as i32;
                let width = request
                    .params
                    .get("width")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'width' parameter".to_string())
                    })? as i32;
                let height = request
                    .params
                    .get("height")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'height' parameter".to_string(),
                        )
                    })? as i32;
                Ok(RpcMethod::SetLayerSourceRectangle {
                    id,
                    x,
                    y,
                    width,
                    height,
                })
            }

            "set_layer_destination_rectangle" => {
                let id = request
                    .params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'id' parameter".to_string())
                    })? as u32;
                let x = request
                    .params
                    .get("x")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'x' parameter".to_string())
                    })? as i32;
                let y = request
                    .params
                    .get("y")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'y' parameter".to_string())
                    })? as i32;
                let width = request
                    .params
                    .get("width")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'width' parameter".to_string())
                    })? as i32;
                let height = request
                    .params
                    .get("height")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'height' parameter".to_string(),
                        )
                    })? as i32;
                Ok(RpcMethod::SetLayerDestinationRectangle {
                    id,
                    x,
                    y,
                    width,
                    height,
                })
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
                    })? as u32;
                let opacity = request
                    .params
                    .get("opacity")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'opacity' parameter".to_string(),
                        )
                    })? as f32;
                Ok(RpcMethod::SetLayerOpacity { id, opacity })
            }

            // Layer-surface assignment operations
            "set_layer_surfaces" => {
                let layer_id = request
                    .params
                    .get("layer_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'layer_id' parameter".to_string(),
                        )
                    })? as u32;
                let surface_ids = request
                    .params
                    .get("surface_ids")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'surface_ids' parameter".to_string(),
                        )
                    })?
                    .iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u32))
                    .collect();
                let auto_commit = request
                    .params
                    .get("auto_commit")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(RpcMethod::SetLayerSurfaces {
                    layer_id,
                    surface_ids,
                    auto_commit,
                })
            }

            "add_surface_to_layer" => {
                let layer_id = request
                    .params
                    .get("layer_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'layer_id' parameter".to_string(),
                        )
                    })? as u32;
                let surface_id = request
                    .params
                    .get("surface_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'surface_id' parameter".to_string(),
                        )
                    })? as u32;
                let auto_commit = request
                    .params
                    .get("auto_commit")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(RpcMethod::AddSurfaceToLayer {
                    layer_id,
                    surface_id,
                    auto_commit,
                })
            }

            "remove_surface_from_layer" => {
                let layer_id = request
                    .params
                    .get("layer_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'layer_id' parameter".to_string(),
                        )
                    })? as u32;
                let surface_id = request
                    .params
                    .get("surface_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'surface_id' parameter".to_string(),
                        )
                    })? as u32;
                let auto_commit = request
                    .params
                    .get("auto_commit")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(RpcMethod::RemoveSurfaceFromLayer {
                    layer_id,
                    surface_id,
                    auto_commit,
                })
            }

            "get_layer_surfaces" => {
                let layer_id = request
                    .params
                    .get("layer_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'layer_id' parameter".to_string(),
                        )
                    })? as u32;
                Ok(RpcMethod::GetLayerSurfaces { layer_id })
            }

            // Screen operations
            "list_screens" => Ok(RpcMethod::ListScreens),

            "get_screen" => {
                let name = request
                    .params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        RpcError::invalid_params("Missing or invalid 'name' parameter".to_string())
                    })?
                    .to_string();
                Ok(RpcMethod::GetScreen { name })
            }

            "get_screen_layers" => {
                let screen_name = request
                    .params
                    .get("screen_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'screen_name' parameter".to_string(),
                        )
                    })?
                    .to_string();
                Ok(RpcMethod::GetScreenLayers { screen_name })
            }

            "get_layer_screens" => {
                let layer_id = request
                    .params
                    .get("layer_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'layer_id' parameter".to_string(),
                        )
                    })? as u32;
                Ok(RpcMethod::GetLayerScreens { layer_id })
            }

            "add_layers_to_screen" => {
                let screen_name = request
                    .params
                    .get("screen_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'screen_name' parameter".to_string(),
                        )
                    })?
                    .to_string();
                let layer_ids = request
                    .params
                    .get("layer_ids")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'layer_ids' parameter".to_string(),
                        )
                    })?
                    .iter()
                    .map(|v| {
                        v.as_u64().ok_or_else(|| {
                            RpcError::invalid_params("Invalid layer_id in array".to_string())
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .map(|v| v as u32)
                    .collect();
                let auto_commit = request
                    .params
                    .get("auto_commit")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(RpcMethod::AddLayersToScreen {
                    screen_name,
                    layer_ids,
                    auto_commit,
                })
            }

            "remove_layer_from_screen" => {
                let screen_name = request
                    .params
                    .get("screen_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'screen_name' parameter".to_string(),
                        )
                    })?
                    .to_string();
                let layer_id = request
                    .params
                    .get("layer_id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        RpcError::invalid_params(
                            "Missing or invalid 'layer_id' parameter".to_string(),
                        )
                    })? as u32;
                let auto_commit = request
                    .params
                    .get("auto_commit")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                Ok(RpcMethod::RemoveLayerFromScreen {
                    screen_name,
                    layer_id,
                    auto_commit,
                })
            }

            _ => Err(RpcError::method_not_found(request.method.clone())),
        }
    }
}
