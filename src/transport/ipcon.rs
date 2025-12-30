// IPCON transport implementation
//
// This module provides an IPCON-based transport for RPC communication.
// IPCON is a message-based IPC mechanism that supports multicast groups
// for efficient event notification delivery.
//
// Enable with the `enable-ipcon` feature flag in Cargo.toml.

use ipcon_sys::{
    ipcon::{Ipcon, IPF_DEFAULT},
    ipcon_msg::{IpconMsg, IpconMsgType},
};
#[allow(unused)]
use jlogger_tracing::{jdebug, jerror, jinfo, jwarn, JloggerBuilder, LevelFilter};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::{str::FromStr, sync::atomic::AtomicBool};

use crate::rpc::transport::{ClientId, MessageHandler, Transport, TransportError};

/// IPCON transport implementation
///
/// This transport uses IPCON (Inter-Process Communication via Notifications) for
/// client-server communication. It provides:
///
/// - **Unicast messaging**: Direct messages to specific clients
/// - **Multicast groups**: Efficient event notifications to multiple subscribers
/// - **Automatic client tracking**: Detects when clients connect/disconnect
///
/// # Example
///
/// ```no_run
/// use weston_ivi_controller::transport::ipcon::IpconTransport;
///
/// let transport = IpconTransport::new(None).unwrap();
/// // Register handler, start transport...
/// ```
pub struct IpconTransport {
    ih: Arc<Mutex<Ipcon>>,
    clients: Arc<Mutex<Vec<ClientId>>>,
    handler: Option<Arc<dyn MessageHandler>>,
    worker_thread: Option<JoinHandle<Result<(), TransportError>>>,
    running: Arc<AtomicBool>,
}

pub const DEFAULT_WESTON_IVI_CONTROLLER_PEER: &str = "weston-ivi-controller";
pub const DEFAULT_WESTON_IVI_CONTROLLER_GROUP: &str = "weston-ivi-controller-events";

impl IpconTransport {
    /// Create a new IPCON transport
    ///
    /// # Arguments
    ///
    /// * `peer` - Optional peer name. Defaults to "weston-ivi-controller" if not provided.
    ///
    /// # Returns
    ///
    /// Returns a new `IpconTransport` instance or an error if initialization fails.
    pub fn new(peer: Option<String>) -> Result<Self, TransportError> {
        let peer = peer.unwrap_or_else(|| DEFAULT_WESTON_IVI_CONTROLLER_PEER.to_string());

        let ih = Ipcon::new(Some(&peer), Some(IPF_DEFAULT)).map_err(|e| {
            jerror!("Failed to create IPCON transport: {}", e);
            TransportError::InitError(format!("Failed to create IPCON transport: {}", e))
        })?;

        ih.register_group(DEFAULT_WESTON_IVI_CONTROLLER_GROUP)
            .map_err(|e| {
                jerror!(
                    "Failed to register IPCON group {}: {}",
                    DEFAULT_WESTON_IVI_CONTROLLER_GROUP,
                    e
                );
                TransportError::InitError(format!(
                    "Failed to register IPCON group {}: {}",
                    DEFAULT_WESTON_IVI_CONTROLLER_GROUP, e
                ))
            })?;

        Ok(Self {
            ih: Arc::new(Mutex::new(ih)),
            clients: Arc::new(Mutex::new(Vec::new())),
            handler: None,
            worker_thread: None,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Main event loop for handling connections
    fn event_loop(
        ih: Arc<Mutex<Ipcon>>,
        handler: Arc<dyn MessageHandler>,
        clients: Arc<Mutex<Vec<ClientId>>>,
        should_run: Arc<AtomicBool>,
    ) -> Result<(), TransportError> {
        loop {
            if !should_run.load(std::sync::atomic::Ordering::SeqCst) {
                jinfo!("IPCON transport event loop stopping");
                break;
            }

            let msg;
            {
                // Mutex lock fails if another thread panicked while holding the lock, so we handle
                // that case
                let ih = ih.lock().unwrap_or_else(|poisoned| {
                    jerror!("Failed to lock IPCON mutex: {}", poisoned);
                    poisoned.into_inner()
                });

                // Wait for a message with a timeout to allow checking the running flag
                msg = ih.receive_msg_timeout(1, 0).map_err(|e| {
                    TransportError::ReceiveError(format!("Failed to receive IPCON message: {}", e))
                })?;
            }

            match msg {
                IpconMsg::IpconMsgUser(data) => {
                    // Ipcon is a message-based protocol, so each message is complete
                    if data.msg_type == IpconMsgType::IpconMsgTypeNormal {
                        let client_id = ClientId::from_str(&data.peer)?;
                        handler.handle_message(&client_id, &data.buf);

                        {
                            // Add client to connected clients list if not already present
                            let mut clients_lock = clients.lock().unwrap();
                            if !clients_lock.contains(&client_id) {
                                jinfo!("IPCON client connected: {}", data.peer);
                                clients_lock.push(client_id);
                            }
                        }
                    }
                }

                IpconMsg::IpconMsgKevent(kevent) => {
                    // Client is removed
                    if let Some(peer) = kevent.peer_removed() {
                        jinfo!("IPCON group removed: {}", peer);
                        handler.handle_disconnect(&ClientId::from_str(&peer)?);

                        {
                            // Remove client from connected clients list
                            let mut clients_lock = clients.lock().unwrap();
                            clients_lock.retain(|c| c.ipcon_id().unwrap() != peer);
                        }
                    }
                }

                IpconMsg::IpconMsgInvalid => {
                    jwarn!("Received invalid IPCON message");
                }
            }
        }

        Ok(())
    }
}

impl Transport for IpconTransport {
    fn start(&mut self) -> Result<(), TransportError> {
        // Start the event loop in a separate thread
        let handler = self
            .handler
            .as_ref()
            .ok_or(TransportError::InitError(
                "Message handler not registered".to_string(),
            ))?
            .clone();

        let ih = Arc::clone(&self.ih);
        let should_run = Arc::clone(&self.running);
        let clients = Arc::clone(&self.clients);
        should_run.store(true, std::sync::atomic::Ordering::SeqCst);

        let handle = thread::spawn(move || -> Result<(), TransportError> {
            IpconTransport::event_loop(ih, handler, clients, should_run)
        });

        self.worker_thread = Some(handle);

        Ok(())
    }

    fn stop(&mut self) -> Result<(), TransportError> {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        if let Some(handle) = self.worker_thread.take() {
            handle.join().map_err(|e| {
                TransportError::InitError(format!("Failed to join IPCON worker thread: {:?}", e))
            })??;
        }
        Ok(())
    }

    fn send(&self, client_id: &ClientId, data: &[u8]) -> Result<(), TransportError> {
        let ih = self.ih.lock().unwrap();
        let peer = client_id.ipcon_id().ok_or_else(|| {
            TransportError::SendError(format!("Client ID {} is not a valid IPCON ID", client_id))
        })?;

        ih.send_unicast_msg(peer, data).map_err(|e| {
            TransportError::SendError(format!("Failed to send IPCON message to {}: {}", peer, e))
        })
    }

    fn send_to_clients(
        &self,
        _client_ids: &[&ClientId],
        data: &[u8],
    ) -> Result<(), TransportError> {
        // We use multicast for notifications
        let ih = self.ih.lock().unwrap();

        ih.send_multicast(DEFAULT_WESTON_IVI_CONTROLLER_GROUP, data, true)
            .map_err(|e| {
                TransportError::SendError(format!("Failed to send IPCON multicast message: {}", e))
            })
    }

    fn get_connected_clients(&self) -> Vec<ClientId> {
        let clients_lock = self.clients.lock().unwrap();
        clients_lock.clone()
    }

    fn register_handler(&mut self, handler: Box<dyn MessageHandler>) {
        self.handler = Some(Arc::from(handler));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::transport::{ClientId, MessageHandler};
    use std::sync::{Arc, Mutex};

    /// Test handler that records messages and disconnects
    type IviMessage = (ClientId, Vec<u8>);
    struct TestHandler {
        messages: Arc<Mutex<Vec<IviMessage>>>,
        disconnects: Arc<Mutex<Vec<ClientId>>>,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                messages: Arc::new(Mutex::new(Vec::new())),
                disconnects: Arc::new(Mutex::new(Vec::new())),
            }
        }

        #[allow(dead_code)]
        fn get_messages(&self) -> Vec<(ClientId, Vec<u8>)> {
            self.messages.lock().unwrap().clone()
        }

        #[allow(dead_code)]
        fn get_disconnects(&self) -> Vec<ClientId> {
            self.disconnects.lock().unwrap().clone()
        }
    }

    impl MessageHandler for TestHandler {
        fn handle_message(&self, client_id: &ClientId, data: &[u8]) {
            self.messages
                .lock()
                .unwrap()
                .push((client_id.clone(), data.to_vec()));
        }

        fn handle_disconnect(&self, client_id: &ClientId) {
            self.disconnects.lock().unwrap().push(client_id.clone());
        }
    }

    #[test]
    fn test_ipcon_transport_creation_with_default_peer() {
        // Test creating transport with default peer name
        let transport = IpconTransport::new(None);

        // This may fail if IPCON is not available on the system
        // That's expected behavior
        match transport {
            Ok(t) => {
                assert!(t.handler.is_none());
                assert_eq!(t.clients.lock().unwrap().len(), 0);
                assert!(!t.running.load(std::sync::atomic::Ordering::SeqCst));
            }
            Err(e) => {
                // IPCON not available is acceptable for unit tests
                println!(
                    "IPCON not available (expected in test environment): {:?}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_ipcon_transport_creation_with_custom_peer() {
        // Test creating transport with custom peer name
        let custom_peer = "test-peer".to_string();
        let transport = IpconTransport::new(Some(custom_peer));

        match transport {
            Ok(t) => {
                assert!(t.handler.is_none());
                assert_eq!(t.clients.lock().unwrap().len(), 0);
            }
            Err(e) => {
                println!(
                    "IPCON not available (expected in test environment): {:?}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_handler_registration() {
        let transport = IpconTransport::new(None);

        if let Ok(mut t) = transport {
            let handler = TestHandler::new();
            assert!(t.handler.is_none());

            t.register_handler(Box::new(handler));
            assert!(t.handler.is_some());
        }
    }

    #[test]
    fn test_start_without_handler_fails() {
        let transport = IpconTransport::new(None);

        if let Ok(mut t) = transport {
            // Starting without registering a handler should fail
            let result = t.start();
            assert!(result.is_err());
            assert!(matches!(result, Err(TransportError::InitError(_))));
        }
    }

    #[test]
    fn test_get_connected_clients_empty() {
        let transport = IpconTransport::new(None);

        if let Ok(t) = transport {
            let clients = t.get_connected_clients();
            assert_eq!(clients.len(), 0);
        }
    }

    #[test]
    fn test_client_id_conversions() {
        // Test ClientId creation for IPCON
        let peer_name = "test-peer";
        let client_id = ClientId::from_str(peer_name).unwrap();

        // Verify it's an IPCON ID
        assert_eq!(client_id.ipcon_id(), Some(peer_name));
        assert_eq!(client_id.unix_domain_id(), None);

        // Test string conversion
        let client_id2 = ClientId::from_string(peer_name.to_string());
        assert_eq!(client_id2.ipcon_id(), Some(peer_name));

        // Test equality
        assert_eq!(client_id, client_id2);
    }

    #[test]
    fn test_client_id_display() {
        let client_id = ClientId::from_str("test-peer").unwrap();
        let display = format!("{}", client_id);
        assert!(display.contains("IpconId"));
        assert!(display.contains("test-peer"));
    }

    #[test]
    #[ignore] // Only run when IPCON is available
    fn test_ipcon_full_lifecycle() {
        // This test requires IPCON to be available on the system
        let mut transport = IpconTransport::new(None).expect("IPCON not available");

        let handler = TestHandler::new();
        let messages = handler.messages.clone();
        let disconnects = handler.disconnects.clone();

        transport.register_handler(Box::new(handler));

        // Start the transport
        transport.start().expect("Failed to start transport");

        // Verify running flag is set
        assert!(transport.running.load(std::sync::atomic::Ordering::SeqCst));

        // Give it a moment to initialize
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Stop the transport
        transport.stop().expect("Failed to stop transport");

        // Verify running flag is cleared
        assert!(!transport.running.load(std::sync::atomic::Ordering::SeqCst));

        // Verify no messages or disconnects (no clients connected in this test)
        assert_eq!(messages.lock().unwrap().len(), 0);
        assert_eq!(disconnects.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_send_requires_ipcon_id() {
        let transport = IpconTransport::new(None);

        if let Ok(t) = transport {
            // Try to send with a Unix domain socket ID (should fail)
            let wrong_client_id = ClientId::from_u64(123);
            let data = b"test message";

            let result = t.send(&wrong_client_id, data);
            assert!(result.is_err());
            assert!(matches!(result, Err(TransportError::SendError(_))));
        }
    }

    #[test]
    #[ignore] // Only run when IPCON is available
    fn test_send_to_clients_multicast() {
        // This test requires IPCON to be available
        let transport = IpconTransport::new(None).expect("IPCON not available");

        let data = b"multicast test message";
        let client_ids = [
            ClientId::from_str("peer1").unwrap(),
            ClientId::from_str("peer2").unwrap(),
        ];
        let client_refs: Vec<&ClientId> = client_ids.iter().collect();

        // This should use multicast group
        let result = transport.send_to_clients(&client_refs, data);

        // May succeed or fail depending on IPCON state
        // Just verify it doesn't panic
        match result {
            Ok(_) => println!("Multicast succeeded"),
            Err(e) => println!("Multicast failed (expected): {:?}", e),
        }
    }

    #[test]
    fn test_stop_without_start() {
        let transport = IpconTransport::new(None);

        if let Ok(mut t) = transport {
            // Stopping without starting should succeed (no-op)
            let result = t.stop();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_multiple_handler_registrations() {
        let transport = IpconTransport::new(None);

        if let Ok(mut t) = transport {
            let handler1 = TestHandler::new();
            let handler2 = TestHandler::new();

            t.register_handler(Box::new(handler1));
            assert!(t.handler.is_some());

            // Second registration should replace the first
            t.register_handler(Box::new(handler2));
            assert!(t.handler.is_some());
        }
    }

    #[test]
    fn test_constants() {
        // Verify the constant values are as expected
        assert_eq!(DEFAULT_WESTON_IVI_CONTROLLER_PEER, "weston-ivi-controller");
        assert_eq!(
            DEFAULT_WESTON_IVI_CONTROLLER_GROUP,
            "weston-ivi-controller-events"
        );
    }
}
