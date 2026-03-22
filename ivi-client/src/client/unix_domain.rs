use super::IviClientTransport;
use crate::error::{IviError, Result};
use std::os::unix::net::UnixStream;
use std::time::Duration;
use weston_ivi_controller::rpc::framing::{write_frame, FrameReadResult, FrameReader};

/// Default socket path for the IVI controller
pub const DEFAULT_SOCKET_PATH: &str = "/tmp/weston-ivi-controller.sock";

/// Client for communicating with the Weston IVI controller.
///
/// The `IviClient` maintains a connection to the IVI controller over a UNIX domain socket
/// and provides methods for sending JSON-RPC requests and receiving responses.
///
/// # Example
///
/// ```no_run
/// use ivi_client::IviClient;
///
/// # fn main() -> ivi_client::Result<()> {
/// let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
/// // Use the client to interact with the IVI controller
/// client.disconnect()?;
/// # Ok(())
/// # }
/// ```
pub struct UnixDomainIviClient {
    /// UNIX domain socket connection to the IVI controller
    socket: Option<UnixStream>,

    /// Frame reader for length-prefixed protocol
    frame_reader: FrameReader,
}

impl UnixDomainIviClient {
    /// Connects to the IVI controller at the specified socket path.
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Path to the UNIX domain socket (e.g., "/tmp/weston-ivi-controller.sock")
    ///
    /// # Returns
    ///
    /// Returns a connected `IviClient` instance on success, or an error if the connection fails.
    ///
    /// # Errors
    ///
    /// Returns `IviError::ConnectionFailed` if:
    /// - The socket file does not exist
    /// - Permission is denied
    /// - The connection is refused
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ivi_client::IviClient;
    ///
    /// # fn main() -> ivi_client::Result<()> {
    /// let client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn connect(socket_path: Option<&str>) -> Result<Self> {
        let socket_path = socket_path.unwrap_or(DEFAULT_SOCKET_PATH);
        let socket = UnixStream::connect(socket_path)
            .map_err(|e| IviError::ConnectionFailed(format!("{}: {}", socket_path, e)))?;

        Ok(Self {
            socket: Some(socket),
            frame_reader: FrameReader::new(),
        })
    }
}

impl IviClientTransport for UnixDomainIviClient {
    fn send_request(&mut self, request: &[u8]) -> Result<()> {
        let socket = self.socket.as_mut().ok_or_else(|| {
            IviError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Socket is not connected",
            ))
        })?;
        write_frame(socket, request).map_err(IviError::IoError)
    }

    fn receive_response(&mut self) -> Result<Vec<u8>> {
        let socket = self.socket.as_mut().ok_or_else(|| {
            IviError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Socket is not connected",
            ))
        })?;
        let response_buf = loop {
            match self.frame_reader.read_frame(socket)? {
                FrameReadResult::Complete(msg) => break msg,
                FrameReadResult::NeedMore => {
                    // Partial read, continue reading
                    std::thread::yield_now();
                    continue;
                }
                FrameReadResult::Eof => {
                    return Err(IviError::IoError(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Connection closed while reading response",
                    )));
                }
            }
        };
        Ok(response_buf)
    }

    fn disconnect(&mut self) -> Result<()> {
        // The socket will be automatically closed when it goes out of scope
        // We explicitly drop it here for clarity
        if let Some(socket) = self.socket.take() {
            drop(socket);
        }
        Ok(())
    }

    fn set_read_timeout(&mut self, timeout: Option<Duration>) -> Result<()> {
        let socket = self.socket.as_mut().ok_or_else(|| {
            IviError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Socket is not connected",
            ))
        })?;
        socket.set_read_timeout(timeout).map_err(IviError::IoError)
    }
}
