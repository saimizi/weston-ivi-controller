//! Integration tests for the IVI client library
//!
//! These tests verify the complete functionality of the client library.
#[allow(unused_imports)]
use ivi_client::{IviClient, IviError};

#[cfg(not(feature = "enable-ipcon"))]
#[test]
fn test_connection_to_nonexistent_socket() {
    let result = IviClient::new(Some("/tmp/nonexistent-socket-12345.sock"));
    assert!(result.is_err());

    match result {
        Err(IviError::ConnectionFailed(msg)) => {
            assert!(msg.contains("/tmp/nonexistent-socket-12345.sock"));
        }
        _ => panic!("Expected ConnectionFailed error"),
    }
}

// Note: Full end-to-end tests with a real IVI controller would require
// a running Weston instance with the IVI controller plugin loaded.
// Those tests would be added in a separate test suite that can be run
// in a proper test environment.
