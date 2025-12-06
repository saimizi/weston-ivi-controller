// UNIX domain socket transport implementation

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::rpc::transport::{ClientId, MessageHandler, Transport, TransportError};

/// Configuration for UNIX domain socket transport
pub struct UnixSocketConfig {
    pub socket_path: PathBuf,
    pub max_connections: usize,
}

/// Client connection state
struct ClientConnection {
    stream: UnixStream,
    buffer: Vec<u8>,
}

/// Shared state for the transport
struct TransportState {
    clients: HashMap<ClientId, ClientConnection>,
    next_client_id: ClientId,
    handler: Option<Box<dyn MessageHandler>>,
    running: bool,
}

/// UNIX domain socket transport implementation
pub struct UnixSocketTransport {
    config: UnixSocketConfig,
    state: Arc<Mutex<TransportState>>,
    listener_thread: Option<JoinHandle<()>>,
}

impl UnixSocketTransport {
    /// Create a new UNIX socket transport
    pub fn new(config: UnixSocketConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(TransportState {
                clients: HashMap::new(),
                next_client_id: 1,
                handler: None,
                running: false,
            })),
            listener_thread: None,
        }
    }

    /// Accept and handle new connections
    fn accept_connection(
        listener: &UnixListener,
        state: &Arc<Mutex<TransportState>>,
    ) -> io::Result<()> {
        match listener.accept() {
            Ok((stream, _addr)) => {
                // Set non-blocking mode
                stream.set_nonblocking(true)?;

                let mut state_lock = state.lock().unwrap();
                let client_id = state_lock.next_client_id;
                state_lock.next_client_id += 1;

                tracing::info!("New client connected: {}", client_id);

                state_lock.clients.insert(
                    client_id,
                    ClientConnection {
                        stream,
                        buffer: Vec::new(),
                    },
                );

                drop(state_lock);
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(()),
            Err(e) => {
                tracing::error!("Error accepting connection: {}", e);
                Err(e)
            }
        }
    }

    /// Read data from a client (returns messages to process)
    fn read_from_client(connection: &mut ClientConnection) -> io::Result<(bool, Vec<Vec<u8>>)> {
        let mut temp_buf = [0u8; 4096];
        let mut messages = Vec::new();

        loop {
            match connection.stream.read(&mut temp_buf) {
                Ok(0) => {
                    // Connection closed
                    return Ok((false, messages));
                }
                Ok(n) => {
                    connection.buffer.extend_from_slice(&temp_buf[..n]);

                    // Extract complete messages (newline-delimited)
                    while let Some(pos) = connection.buffer.iter().position(|&b| b == b'\n') {
                        let message = connection.buffer.drain(..=pos).collect::<Vec<u8>>();
                        let message = &message[..message.len() - 1]; // Remove newline

                        if !message.is_empty() {
                            messages.push(message.to_vec());
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    return Ok((true, messages));
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                    continue;
                }
                Err(_e) => {
                    return Ok((false, messages));
                }
            }
        }
    }

    /// Main event loop for handling connections
    fn event_loop(listener: UnixListener, state: Arc<Mutex<TransportState>>) {
        listener.set_nonblocking(true).unwrap();

        loop {
            // Check if we should stop
            {
                let state_lock = state.lock().unwrap();
                if !state_lock.running {
                    break;
                }
            }

            // Accept new connections
            let _ = Self::accept_connection(&listener, &state);

            // Process existing connections
            let mut disconnected_clients = Vec::new();
            let mut client_messages: Vec<(ClientId, Vec<Vec<u8>>)> = Vec::new();

            {
                let mut state_lock = state.lock().unwrap();

                for (&client_id, connection) in state_lock.clients.iter_mut() {
                    match Self::read_from_client(connection) {
                        Ok((alive, messages)) => {
                            // Store messages if any
                            if !messages.is_empty() {
                                client_messages.push((client_id, messages));
                            }

                            // Mark for disconnection if not alive
                            if !alive {
                                disconnected_clients.push(client_id);
                            }
                        }
                        Err(_) => {
                            // Connection error
                            disconnected_clients.push(client_id);
                        }
                    }
                }
            }

            // Process messages outside the lock
            {
                let state_lock = state.lock().unwrap();
                if let Some(ref handler) = state_lock.handler {
                    for (client_id, messages) in client_messages {
                        for message in messages {
                            handler.handle_message(client_id, &message);
                        }
                    }
                }
            }

            // Clean up disconnected clients
            if !disconnected_clients.is_empty() {
                let mut state_lock = state.lock().unwrap();

                for client_id in disconnected_clients {
                    tracing::info!("Client {} disconnected, cleaning up", client_id);
                    state_lock.clients.remove(&client_id);

                    if let Some(ref handler) = state_lock.handler {
                        handler.handle_disconnect(client_id);
                    }
                }
            }

            // Small sleep to avoid busy-waiting
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Transport for UnixSocketTransport {
    fn start(&mut self) -> Result<(), TransportError> {
        tracing::info!(
            "Starting UNIX socket transport at {:?}",
            self.config.socket_path
        );

        // Remove existing socket file if it exists
        if self.config.socket_path.exists() {
            tracing::debug!("Removing existing socket file");
            std::fs::remove_file(&self.config.socket_path).map_err(|e| {
                tracing::error!("Failed to remove existing socket: {}", e);
                TransportError::InitError(format!("Failed to remove existing socket: {}", e))
            })?;
        }

        // Create the UNIX domain socket
        let listener = UnixListener::bind(&self.config.socket_path).map_err(|e| {
            tracing::error!("Failed to bind socket: {}", e);
            TransportError::InitError(format!("Failed to bind socket: {}", e))
        })?;

        // Mark as running
        {
            let mut state = self.state.lock().unwrap();
            state.running = true;
        }

        // Start the listener thread
        let state_clone = Arc::clone(&self.state);
        let handle = thread::spawn(move || {
            Self::event_loop(listener, state_clone);
        });

        self.listener_thread = Some(handle);

        tracing::info!("UNIX socket transport started successfully");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), TransportError> {
        tracing::info!("Stopping UNIX socket transport");

        // Signal the thread to stop
        {
            let mut state = self.state.lock().unwrap();
            state.running = false;
        }

        // Wait for the thread to finish
        if let Some(handle) = self.listener_thread.take() {
            handle.join().map_err(|_| {
                tracing::error!("Failed to join listener thread");
                TransportError::ConnectionError("Failed to join listener thread".to_string())
            })?;
        }

        // Clean up the socket file
        if self.config.socket_path.exists() {
            tracing::debug!("Removing socket file");
            std::fs::remove_file(&self.config.socket_path).map_err(|e| {
                tracing::error!("Failed to remove socket: {}", e);
                TransportError::ConnectionError(format!("Failed to remove socket: {}", e))
            })?;
        }

        tracing::info!("UNIX socket transport stopped successfully");

        Ok(())
    }

    fn send(&self, client_id: ClientId, data: &[u8]) -> Result<(), TransportError> {
        let mut state = self.state.lock().unwrap();

        if let Some(connection) = state.clients.get_mut(&client_id) {
            // Send data with newline delimiter
            connection
                .stream
                .write_all(data)
                .map_err(|e| TransportError::SendError(format!("Failed to send data: {}", e)))?;
            connection
                .stream
                .write_all(b"\n")
                .map_err(|e| TransportError::SendError(format!("Failed to send newline: {}", e)))?;
            connection
                .stream
                .flush()
                .map_err(|e| TransportError::SendError(format!("Failed to flush: {}", e)))?;

            Ok(())
        } else {
            Err(TransportError::SendError(format!(
                "Client {} not found",
                client_id
            )))
        }
    }

    fn register_handler(&mut self, handler: Box<dyn MessageHandler>) {
        let mut state = self.state.lock().unwrap();
        state.handler = Some(handler);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::transport::{ClientId, MessageHandler};
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    struct TestHandler {
        messages: Arc<Mutex<Vec<(ClientId, Vec<u8>)>>>,
        disconnects: Arc<Mutex<Vec<ClientId>>>,
    }

    impl MessageHandler for TestHandler {
        fn handle_message(&self, client_id: ClientId, data: &[u8]) {
            self.messages
                .lock()
                .unwrap()
                .push((client_id, data.to_vec()));
        }

        fn handle_disconnect(&self, client_id: ClientId) {
            self.disconnects.lock().unwrap().push(client_id);
        }
    }

    #[test]
    fn test_unix_socket_basic() {
        let socket_path = PathBuf::from("/tmp/test_ivi_socket_basic");

        // Clean up any existing socket
        let _ = std::fs::remove_file(&socket_path);

        let config = UnixSocketConfig {
            socket_path: socket_path.clone(),
            max_connections: 10,
        };

        let messages = Arc::new(Mutex::new(Vec::new()));
        let disconnects = Arc::new(Mutex::new(Vec::new()));

        let handler = TestHandler {
            messages: Arc::clone(&messages),
            disconnects: Arc::clone(&disconnects),
        };

        let mut transport = UnixSocketTransport::new(config);
        transport.register_handler(Box::new(handler));

        // Start the transport
        transport.start().expect("Failed to start transport");

        // Give it time to start
        thread::sleep(Duration::from_millis(100));

        // Connect a client
        let mut client = UnixStream::connect(&socket_path).expect("Failed to connect");

        // Send a message
        client
            .write_all(b"test message\n")
            .expect("Failed to write");
        client.flush().expect("Failed to flush");

        // Give it time to process
        thread::sleep(Duration::from_millis(100));

        // Check that the message was received
        let msgs = messages.lock().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].1, b"test message");

        // Close the client
        drop(client);

        // Give it time to detect disconnection
        thread::sleep(Duration::from_millis(100));

        // Check that disconnection was detected
        let discs = disconnects.lock().unwrap();
        assert_eq!(discs.len(), 1);

        // Stop the transport
        transport.stop().expect("Failed to stop transport");

        // Clean up
        let _ = std::fs::remove_file(&socket_path);
    }

    #[test]
    fn test_unix_socket_multiple_clients() {
        let socket_path = PathBuf::from("/tmp/test_ivi_socket_multi");

        // Clean up any existing socket
        let _ = std::fs::remove_file(&socket_path);

        let config = UnixSocketConfig {
            socket_path: socket_path.clone(),
            max_connections: 10,
        };

        let messages = Arc::new(Mutex::new(Vec::new()));
        let disconnects = Arc::new(Mutex::new(Vec::new()));

        let handler = TestHandler {
            messages: Arc::clone(&messages),
            disconnects: Arc::clone(&disconnects),
        };

        let mut transport = UnixSocketTransport::new(config);
        transport.register_handler(Box::new(handler));

        // Start the transport
        transport.start().expect("Failed to start transport");

        // Give it time to start
        thread::sleep(Duration::from_millis(100));

        // Connect multiple clients
        let mut client1 = UnixStream::connect(&socket_path).expect("Failed to connect client 1");
        let mut client2 = UnixStream::connect(&socket_path).expect("Failed to connect client 2");

        // Send messages from both clients
        client1
            .write_all(b"message from client 1\n")
            .expect("Failed to write");
        client1.flush().expect("Failed to flush");

        client2
            .write_all(b"message from client 2\n")
            .expect("Failed to write");
        client2.flush().expect("Failed to flush");

        // Give it time to process
        thread::sleep(Duration::from_millis(100));

        // Check that both messages were received
        let msgs = messages.lock().unwrap();
        assert_eq!(msgs.len(), 2);

        // Stop the transport
        transport.stop().expect("Failed to stop transport");

        // Clean up
        let _ = std::fs::remove_file(&socket_path);
    }
}
