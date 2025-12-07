//! JSON-RPC 2.0 protocol implementation for IVI controller communication.
//!
//! This module defines the request and response structures for the JSON-RPC protocol
//! used to communicate with the Weston IVI controller over UNIX domain sockets.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request structure.
///
/// Represents a request to be sent to the IVI controller.
///
/// # Example
///
/// ```
/// use ivi_client::protocol::JsonRpcRequest;
/// use serde_json::json;
///
/// let request = JsonRpcRequest {
///     id: 1,
///     method: "list_surfaces".to_string(),
///     params: json!({}),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    /// Unique identifier for this request. Used to match responses to requests.
    pub id: u64,

    /// The name of the method to invoke on the server.
    pub method: String,

    /// Parameters to pass to the method. Can be an object or array.
    pub params: Value,
}

impl JsonRpcRequest {
    /// Creates a new JSON-RPC request.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique request identifier
    /// * `method` - Method name to invoke
    /// * `params` - Method parameters as a JSON value
    ///
    /// # Example
    ///
    /// ```
    /// use ivi_client::protocol::JsonRpcRequest;
    /// use serde_json::json;
    ///
    /// let request = JsonRpcRequest::new(1, "get_surface", json!({"id": 1000}));
    /// ```
    pub fn new(id: u64, method: impl Into<String>, params: Value) -> Self {
        Self {
            id,
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 2.0 response structure.
///
/// Represents a response received from the IVI controller.
/// A response contains either a successful result or an error.
///
/// # Example
///
/// ```
/// use ivi_client::protocol::JsonRpcResponse;
/// use serde_json::json;
///
/// let response = JsonRpcResponse {
///     id: 1,
///     result: Some(json!({"surfaces": [1000, 1001]})),
///     error: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    /// Request identifier that this response corresponds to.
    pub id: u64,

    /// The result of a successful method invocation.
    /// Present only if the request succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error information if the request failed.
    /// Present only if the request failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Creates a new successful JSON-RPC response.
    ///
    /// # Arguments
    ///
    /// * `id` - Request identifier
    /// * `result` - Result value
    ///
    /// # Example
    ///
    /// ```
    /// use ivi_client::protocol::JsonRpcResponse;
    /// use serde_json::json;
    ///
    /// let response = JsonRpcResponse::success(1, json!({"status": "ok"}));
    /// ```
    pub fn success(id: u64, result: Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Creates a new error JSON-RPC response.
    ///
    /// # Arguments
    ///
    /// * `id` - Request identifier
    /// * `error` - Error information
    ///
    /// # Example
    ///
    /// ```
    /// use ivi_client::protocol::{JsonRpcResponse, JsonRpcError};
    ///
    /// let error = JsonRpcError::new(-32000, "Surface not found");
    /// let response = JsonRpcResponse::error(1, error);
    /// ```
    pub fn error(id: u64, error: JsonRpcError) -> Self {
        Self {
            id,
            result: None,
            error: Some(error),
        }
    }

    /// Checks if this response represents a successful result.
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Checks if this response represents an error.
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

/// JSON-RPC 2.0 error structure.
///
/// Represents an error returned by the IVI controller.
///
/// # Example
///
/// ```
/// use ivi_client::protocol::JsonRpcError;
///
/// let error = JsonRpcError::new(-32000, "Surface not found");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    /// Error code. Standard JSON-RPC error codes are in the range -32768 to -32000.
    pub code: i32,

    /// Human-readable error message.
    pub message: String,

    /// Optional additional error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Creates a new JSON-RPC error.
    ///
    /// # Arguments
    ///
    /// * `code` - Error code
    /// * `message` - Error message
    ///
    /// # Example
    ///
    /// ```
    /// use ivi_client::protocol::JsonRpcError;
    ///
    /// let error = JsonRpcError::new(-32602, "Invalid params");
    /// ```
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Creates a new JSON-RPC error with additional data.
    ///
    /// # Arguments
    ///
    /// * `code` - Error code
    /// * `message` - Error message
    /// * `data` - Additional error data
    pub fn with_data(code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_creation() {
        let request = JsonRpcRequest::new(1, "list_surfaces", json!({}));
        assert_eq!(request.id, 1);
        assert_eq!(request.method, "list_surfaces");
        assert_eq!(request.params, json!({}));
    }

    #[test]
    fn test_request_serialization() {
        let request = JsonRpcRequest::new(42, "get_surface", json!({"id": 1000}));
        let serialized = serde_json::to_string(&request).unwrap();

        // Verify the JSON contains expected fields
        assert!(serialized.contains("\"id\":42"));
        assert!(serialized.contains("\"method\":\"get_surface\""));
        assert!(serialized.contains("\"params\""));
    }

    #[test]
    fn test_request_deserialization() {
        let json_str = r#"{"id":1,"method":"list_surfaces","params":{}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();

        assert_eq!(request.id, 1);
        assert_eq!(request.method, "list_surfaces");
        assert_eq!(request.params, json!({}));
    }

    #[test]
    fn test_request_round_trip() {
        let original = JsonRpcRequest::new(123, "commit", json!({}));
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_success_response_creation() {
        let response = JsonRpcResponse::success(1, json!({"surfaces": [1000, 1001]}));
        assert_eq!(response.id, 1);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert!(response.is_success());
        assert!(!response.is_error());
    }

    #[test]
    fn test_error_response_creation() {
        let error = JsonRpcError::new(-32000, "Surface not found");
        let response = JsonRpcResponse::error(1, error);

        assert_eq!(response.id, 1);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert!(!response.is_success());
        assert!(response.is_error());
    }

    #[test]
    fn test_response_serialization() {
        let response = JsonRpcResponse::success(1, json!({"status": "ok"}));
        let serialized = serde_json::to_string(&response).unwrap();

        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("\"result\""));
        assert!(!serialized.contains("\"error\""));
    }

    #[test]
    fn test_response_deserialization() {
        let json_str = r#"{"id":1,"result":{"surfaces":[1000,1001]}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json_str).unwrap();

        assert_eq!(response.id, 1);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_error_response_serialization() {
        let error = JsonRpcError::new(-32000, "Surface not found");
        let response = JsonRpcResponse::error(1, error);
        let serialized = serde_json::to_string(&response).unwrap();

        assert!(serialized.contains("\"id\":1"));
        assert!(serialized.contains("\"error\""));
        assert!(!serialized.contains("\"result\""));
    }

    #[test]
    fn test_error_response_deserialization() {
        let json_str = r#"{"id":1,"error":{"code":-32000,"message":"Surface not found"}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json_str).unwrap();

        assert_eq!(response.id, 1);
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32000);
        assert_eq!(error.message, "Surface not found");
    }

    #[test]
    fn test_error_with_data() {
        let error = JsonRpcError::with_data(
            -32602,
            "Invalid params",
            json!({"param": "opacity", "value": 1.5}),
        );

        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "Invalid params");
        assert!(error.data.is_some());
    }

    #[test]
    fn test_response_round_trip() {
        let original = JsonRpcResponse::success(456, json!({"layers": [100, 200, 300]}));
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_error_response_round_trip() {
        let error = JsonRpcError::new(-32601, "Method not found");
        let original = JsonRpcResponse::error(789, error);
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}
