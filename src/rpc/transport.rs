// Transport abstraction layer

use thiserror::Error;

/// Client identifier for different transport types
///
/// This enum represents client IDs across different transport mechanisms.
/// The variant used depends on which transport feature is enabled.
///
/// # Variants
///
/// - `UnixDomainId`: Numeric ID for Unix domain socket clients (default)
/// - `IpconId`: String-based peer name for IPCON clients (requires `enable-ipcon` feature)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClientId {
    /// Unix domain socket client ID (numeric)
    UnixDomainId(u64),

    /// IPCON peer name (string-based, requires `enable-ipcon` feature)
    #[cfg(feature = "enable-ipcon")]
    IpconId(String),
}

impl ClientId {
    /// Extract the Unix domain socket ID if this is a `UnixDomainId` variant
    ///
    /// Returns `None` for other variants.
    pub fn unix_domain_id(&self) -> Option<u64> {
        match self {
            ClientId::UnixDomainId(id) => Some(*id),

            #[cfg(feature = "enable-ipcon")]
            _ => None,
        }
    }

    /// Extract the IPCON peer name if this is an `IpconId` variant
    ///
    /// Returns `None` for other variants.
    /// Only available when the `enable-ipcon` feature is enabled.
    #[cfg(feature = "enable-ipcon")]
    pub fn ipcon_id(&self) -> Option<&str> {
        match self {
            ClientId::IpconId(id) => Some(id),
            _ => None,
        }
    }

    /// Create a `ClientId` from a numeric ID (Unix domain socket)
    pub fn from_u64(id: u64) -> Self {
        ClientId::UnixDomainId(id)
    }

    /// Create a `ClientId` from a String (IPCON peer name)
    ///
    /// Only available when the `enable-ipcon` feature is enabled.
    #[cfg(feature = "enable-ipcon")]
    pub fn from_string(id: String) -> Self {
        ClientId::IpconId(id)
    }

    /// Create a `ClientId` from a string slice (IPCON peer name)
    ///
    /// Only available when the `enable-ipcon` feature is enabled.
    #[cfg(feature = "enable-ipcon")]
    pub fn from_str(id: &str) -> Self {
        ClientId::IpconId(id.to_string())
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientId::UnixDomainId(id) => write!(f, "UnixDomainId({})", id),
            #[cfg(feature = "enable-ipcon")]
            ClientId::IpconId(id) => write!(f, "IpconId({})", id),
        }
    }
}

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
///
/// This trait defines the interface for different IPC transport implementations.
/// Currently supported transports:
///
/// - **Unix domain sockets** (default): Local socket-based communication
/// - **IPCON** (optional): Message-based IPC with multicast support
///
/// Implementations must be thread-safe (`Send + Sync`).
pub trait Transport: Send + Sync {
    /// Start the transport and begin accepting connections
    ///
    /// This should initialize the transport's event loop and start listening
    /// for incoming connections or messages.
    fn start(&mut self) -> Result<(), TransportError>;

    /// Stop the transport and close all connections
    ///
    /// This should cleanly shut down the transport, close all client connections,
    /// and join any background threads.
    fn stop(&mut self) -> Result<(), TransportError>;

    /// Send data to a specific client
    ///
    /// # Arguments
    ///
    /// * `client_id` - The target client identifier
    /// * `data` - The raw bytes to send (should be a complete framed message)
    fn send(&self, client_id: &ClientId, data: &[u8]) -> Result<(), TransportError>;

    /// Send data to multiple clients (typically for event notifications)
    ///
    /// # Arguments
    ///
    /// * `client_ids` - List of target client identifiers
    /// * `data` - The raw bytes to send (same data to all clients)
    ///
    /// Implementations may use multicast or unicast depending on the transport.
    fn send_to_clients(&self, client_ids: &[&ClientId], data: &[u8]) -> Result<(), TransportError>;

    /// Get a list of all currently connected client IDs
    fn get_connected_clients(&self) -> Vec<ClientId>;

    /// Register a message handler for processing incoming messages
    ///
    /// This should be called before `start()`. The handler will be invoked
    /// for each incoming message and client disconnect event.
    fn register_handler(&mut self, handler: Box<dyn MessageHandler>);
}

/// Message handler trait for processing incoming messages
///
/// Implementors of this trait process incoming RPC messages and handle
/// client lifecycle events (connection/disconnection).
///
/// The handler is invoked by the transport layer and should not block
/// for extended periods.
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message from a client
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client that sent the message
    /// * `data` - The raw message bytes (already framed/complete)
    fn handle_message(&self, client_id: &ClientId, data: &[u8]);

    /// Handle client disconnection event
    ///
    /// Called when a client disconnects. Implementors should clean up
    /// any client-specific state (e.g., subscriptions).
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client that disconnected
    fn handle_disconnect(&self, client_id: &ClientId);
}
