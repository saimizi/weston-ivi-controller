//! Error types for the Weston IVI Controller
//!
//! This module defines all error types used throughout the controller,
//! providing a unified error handling approach.

use crate::controller::ValidationError;
use crate::rpc::RpcError;
use thiserror::Error;

/// Main error type for the controller
#[derive(Debug, Error)]
pub enum ControllerError {
    /// Invalid parameter error
    #[error("Invalid parameter '{param}': {reason}")]
    InvalidParameter { param: String, reason: String },

    /// Out of bounds error
    #[error("Parameter '{param}' out of bounds: value={value}, bounds={bounds}")]
    OutOfBounds {
        param: String,
        value: String,
        bounds: String,
    },

    /// Surface not found error
    #[error("Surface not found: id={id}")]
    SurfaceNotFound { id: u32 },

    /// IVI API error
    #[error("IVI API error during '{operation}': code={code}")]
    IviApiError { operation: String, code: i32 },

    /// Transport error
    #[error("Transport error: {message}")]
    TransportError { message: String },

    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// State error
    #[error("State error: {message}")]
    StateError { message: String },

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),

    /// RPC error
    #[error("RPC error: {0}")]
    RpcError(#[from] RpcError),

    /// Initialization error
    #[error("Initialization error: {message}")]
    InitializationError { message: String },

    /// Plugin error
    #[error("Plugin error: {message}")]
    PluginError { message: String },
}

impl From<String> for ControllerError {
    fn from(message: String) -> Self {
        Self::PluginError { message }
    }
}

impl From<&str> for ControllerError {
    fn from(message: &str) -> Self {
        Self::PluginError {
            message: message.to_string(),
        }
    }
}

impl From<crate::rpc::TransportError> for ControllerError {
    fn from(error: crate::rpc::TransportError) -> Self {
        Self::TransportError {
            message: error.to_string(),
        }
    }
}

impl ControllerError {
    /// Create an invalid parameter error
    pub fn invalid_parameter(param: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidParameter {
            param: param.into(),
            reason: reason.into(),
        }
    }

    /// Create an out of bounds error
    pub fn out_of_bounds(
        param: impl Into<String>,
        value: impl Into<String>,
        bounds: impl Into<String>,
    ) -> Self {
        Self::OutOfBounds {
            param: param.into(),
            value: value.into(),
            bounds: bounds.into(),
        }
    }

    /// Create a surface not found error
    pub fn surface_not_found(id: u32) -> Self {
        Self::SurfaceNotFound { id }
    }

    /// Create an IVI API error
    pub fn ivi_api_error(operation: impl Into<String>, code: i32) -> Self {
        Self::IviApiError {
            operation: operation.into(),
            code,
        }
    }

    /// Create a transport error
    pub fn transport_error(message: impl Into<String>) -> Self {
        Self::TransportError {
            message: message.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    /// Create a state error
    pub fn state_error(message: impl Into<String>) -> Self {
        Self::StateError {
            message: message.into(),
        }
    }

    /// Create an initialization error
    pub fn initialization_error(message: impl Into<String>) -> Self {
        Self::InitializationError {
            message: message.into(),
        }
    }

    /// Create a plugin error
    pub fn plugin_error(message: impl Into<String>) -> Self {
        Self::PluginError {
            message: message.into(),
        }
    }

    /// Convert to an RPC error for sending to clients
    pub fn to_rpc_error(&self) -> RpcError {
        match self {
            Self::InvalidParameter { param, reason } => {
                RpcError::invalid_params(format!("Invalid parameter '{}': {}", param, reason))
            }
            Self::OutOfBounds {
                param,
                value,
                bounds,
            } => RpcError::invalid_params(format!(
                "Parameter '{}' out of bounds: value={}, bounds={}",
                param, value, bounds
            )),
            Self::SurfaceNotFound { id } => RpcError::surface_not_found(*id),
            Self::IviApiError { operation, code } => RpcError::internal_error(format!(
                "IVI API error during '{}': code={}",
                operation, code
            )),
            Self::TransportError { message } => {
                RpcError::internal_error(format!("Transport error: {}", message))
            }
            Self::SerializationError { message } => {
                RpcError::internal_error(format!("Serialization error: {}", message))
            }
            Self::StateError { message } => {
                RpcError::internal_error(format!("State error: {}", message))
            }
            Self::ValidationError(e) => RpcError::invalid_params(e.to_string()),
            Self::RpcError(e) => e.clone(),
            Self::InitializationError { message } => {
                RpcError::internal_error(format!("Initialization error: {}", message))
            }
            Self::PluginError { message } => {
                RpcError::internal_error(format!("Plugin error: {}", message))
            }
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> i32 {
        match self {
            Self::InvalidParameter { .. } => -32602,
            Self::OutOfBounds { .. } => -32602,
            Self::SurfaceNotFound { .. } => -32000,
            Self::IviApiError { .. } => -32001,
            Self::TransportError { .. } => -32002,
            Self::SerializationError { .. } => -32700,
            Self::StateError { .. } => -32003,
            Self::ValidationError(_) => -32602,
            Self::RpcError(e) => e.code,
            Self::InitializationError { .. } => -32004,
            Self::PluginError { .. } => -32005,
        }
    }
}

/// Result type alias for controller operations
pub type ControllerResult<T> = std::result::Result<T, ControllerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_parameter_error() {
        let err = ControllerError::invalid_parameter("x", "must be positive");
        assert!(matches!(err, ControllerError::InvalidParameter { .. }));
        assert_eq!(err.to_string(), "Invalid parameter 'x': must be positive");
    }

    #[test]
    fn test_out_of_bounds_error() {
        let err = ControllerError::out_of_bounds("z_order", "100", "[0, 10]");
        assert!(matches!(err, ControllerError::OutOfBounds { .. }));
        assert!(err.to_string().contains("out of bounds"));
    }

    #[test]
    fn test_surface_not_found_error() {
        let err = ControllerError::surface_not_found(42);
        assert!(matches!(err, ControllerError::SurfaceNotFound { id: 42 }));
        assert_eq!(err.to_string(), "Surface not found: id=42");
    }

    #[test]
    fn test_ivi_api_error() {
        let err = ControllerError::ivi_api_error("set_position", -1);
        assert!(matches!(err, ControllerError::IviApiError { .. }));
        assert!(err.to_string().contains("IVI API error"));
    }

    #[test]
    fn test_to_rpc_error() {
        let err = ControllerError::surface_not_found(42);
        let rpc_err = err.to_rpc_error();
        assert_eq!(rpc_err.code, -32000);
        assert!(rpc_err.message.contains("42"));
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(
            ControllerError::invalid_parameter("x", "test").error_code(),
            -32602
        );
        assert_eq!(ControllerError::surface_not_found(1).error_code(), -32000);
        assert_eq!(
            ControllerError::ivi_api_error("test", 0).error_code(),
            -32001
        );
    }
}
