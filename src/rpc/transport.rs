// Transport abstraction layer

use thiserror::Error;

/// Client identifier
pub type ClientId = u64;

/// Transport error types
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Send error: {0}")]
    SendError(String),

    #[error("Receive error: {0}")]
    ReceiveError(String),

    #[error("Initialization error: {0}")]
    InitError(String),
}

/// Transport trait for pluggable communication mechanisms
pub trait Transport: Send + Sync {
    /// Start the transport
    fn start(&mut self) -> Result<(), TransportError>;

    /// Stop the transport
    fn stop(&mut self) -> Result<(), TransportError>;

    /// Send data to a client
    fn send(&self, client_id: ClientId, data: &[u8]) -> Result<(), TransportError>;

    /// Send data to multiple clients (for notifications)
    fn send_to_clients(&self, client_ids: &[ClientId], data: &[u8]) -> Result<(), TransportError>;

    /// Get list of all connected client IDs
    fn get_connected_clients(&self) -> Vec<ClientId>;

    /// Register a message handler
    fn register_handler(&mut self, handler: Box<dyn MessageHandler>);
}

/// Message handler trait for processing incoming messages
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message
    fn handle_message(&self, client_id: ClientId, data: &[u8]);

    /// Handle client disconnection
    fn handle_disconnect(&self, client_id: ClientId);
}
