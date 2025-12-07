//! Error types for the IVI client library

use thiserror::Error;

/// Result type alias for IVI client operations
pub type Result<T> = std::result::Result<T, IviError>;

/// Error types that can occur when using the IVI client library
#[derive(Error, Debug)]
pub enum IviError {
    /// Failed to establish connection to the IVI controller
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// JSON-RPC request failed with an error code and message
    #[error("Request failed (code {code}): {message}")]
    RequestFailed { code: i32, message: String },

    /// Failed to serialize data to JSON
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Failed to deserialize JSON response
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// I/O error occurred during communication
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<serde_json::Error> for IviError {
    fn from(err: serde_json::Error) -> Self {
        IviError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_connection_failed_error() {
        let error = IviError::ConnectionFailed("Socket not found".to_string());
        assert_eq!(error.to_string(), "Connection failed: Socket not found");
    }

    #[test]
    fn test_request_failed_error() {
        let error = IviError::RequestFailed {
            code: -32000,
            message: "Surface not found".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Request failed (code -32000): Surface not found"
        );
    }

    #[test]
    fn test_serialization_error() {
        let error = IviError::SerializationError("Invalid JSON".to_string());
        assert_eq!(error.to_string(), "Serialization error: Invalid JSON");
    }

    #[test]
    fn test_deserialization_error() {
        let error = IviError::DeserializationError("Unexpected field".to_string());
        assert_eq!(error.to_string(), "Deserialization error: Unexpected field");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let ivi_error: IviError = io_error.into();
        assert!(matches!(ivi_error, IviError::IoError(_)));
    }

    #[test]
    fn test_serde_json_error_conversion() {
        let json_str = "{invalid json}";
        let result: std::result::Result<serde_json::Value, _> = serde_json::from_str(json_str);
        let json_error = result.unwrap_err();
        let ivi_error: IviError = json_error.into();
        assert!(matches!(ivi_error, IviError::SerializationError(_)));
    }
}
