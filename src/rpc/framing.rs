//! Transport-independent message framing for JSON-RPC protocol.
//!
//! This module provides length-prefixed message framing that can be used
//! with any transport layer (UNIX sockets, TCP, etc.).
//!
//! ## Wire Format
//!
//! Messages are encoded as:
//! ```text
//! [4-byte length (big-endian u32)][payload bytes]
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use weston_ivi_controller::rpc::framing::{FrameReader, write_frame};
//! use std::io::Cursor;
//!
//! // Writing a frame
//! let mut buffer = Vec::new();
//! write_frame(&mut buffer, b"hello").unwrap();
//!
//! // Reading a frame
//! let mut reader = FrameReader::new();
//! let mut cursor = Cursor::new(&buffer);
//! let message = reader.read_frame(&mut cursor).unwrap();
//! assert_eq!(message, Some(b"hello".to_vec()));
//! ```

use std::io::{self, Read, Write};

/// Maximum message size (64MB) for DOS protection
pub const MAX_MESSAGE_SIZE: u32 = 64 * 1024 * 1024;

/// Result of attempting to read a frame
#[derive(Debug, PartialEq)]
pub enum FrameReadResult {
    /// A complete message was read
    Complete(Vec<u8>),
    /// Need more data (e.g., WouldBlock on non-blocking socket)
    NeedMore,
    /// End of file / connection closed
    Eof,
}

/// State machine for reading length-prefixed messages
#[derive(Debug)]
enum ReadState {
    /// Waiting for 4-byte length header
    WaitingForHeader {
        header_buf: [u8; 4],
        bytes_read: usize,
    },
    /// Waiting for payload bytes
    WaitingForPayload {
        expected_len: u32,
        bytes_read: usize,
        buffer: Vec<u8>,
    },
}

impl Default for ReadState {
    fn default() -> Self {
        ReadState::WaitingForHeader {
            header_buf: [0; 4],
            bytes_read: 0,
        }
    }
}

/// Frame reader for length-prefixed messages.
///
/// This struct maintains state across multiple read operations,
/// allowing it to handle partial reads correctly.
#[derive(Debug)]
pub struct FrameReader {
    state: ReadState,
    buffer: Vec<u8>, // Unprocessed data from previous reads
}

impl FrameReader {
    /// Create a new frame reader
    pub fn new() -> Self {
        Self {
            state: ReadState::default(),
            buffer: Vec::new(),
        }
    }

    /// Read a complete frame from the given reader.
    ///
    /// Returns:
    /// - `Ok(Complete(message))` if a complete message was read
    /// - `Ok(NeedMore)` if more data is needed (e.g., WouldBlock)
    /// - `Ok(Eof)` if end of file was reached
    /// - `Err(io::Error)` on I/O error or protocol violation
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Message length is 0
    /// - Message length exceeds MAX_MESSAGE_SIZE
    /// - I/O error occurs
    pub fn read_frame<R: Read>(&mut self, reader: &mut R) -> io::Result<FrameReadResult> {
        loop {
            // First try to process any buffered data
            if !self.buffer.is_empty() {
                if let Some(message) = self.try_extract_message()? {
                    return Ok(FrameReadResult::Complete(message));
                }
            }

            // Need more data - try to read from reader
            let mut temp_buf = [0u8; 4096];
            let n = match reader.read(&mut temp_buf) {
                Ok(0) => {
                    // EOF
                    return Ok(FrameReadResult::Eof);
                }
                Ok(n) => n,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    return Ok(FrameReadResult::NeedMore); // No data available
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                    continue; // Retry
                }
                Err(e) => return Err(e),
            };

            // Append new data to buffer
            self.buffer.extend_from_slice(&temp_buf[..n]);

            // Try to extract a message from buffer
            if let Some(message) = self.try_extract_message()? {
                return Ok(FrameReadResult::Complete(message));
            }
            // If no complete message yet, loop to read more data
        }
    }

    /// Try to extract a complete message from the internal buffer
    fn try_extract_message(&mut self) -> io::Result<Option<Vec<u8>>> {
        loop {
            match &mut self.state {
                ReadState::WaitingForHeader {
                    header_buf,
                    bytes_read,
                } => {
                    let needed = 4 - *bytes_read;
                    let available = self.buffer.len();

                    if available == 0 {
                        return Ok(None); // Need more data
                    }

                    let to_copy = needed.min(available);

                    // Copy bytes to header buffer and remove from internal buffer
                    header_buf[*bytes_read..*bytes_read + to_copy]
                        .copy_from_slice(&self.buffer[..to_copy]);
                    self.buffer.drain(..to_copy);
                    *bytes_read += to_copy;

                    // If we have complete header, parse it
                    if *bytes_read == 4 {
                        let msg_len = u32::from_be_bytes(*header_buf);

                        // Validate message size
                        if msg_len == 0 {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Message length is zero",
                            ));
                        }
                        if msg_len > MAX_MESSAGE_SIZE {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!(
                                    "Message too large: {} bytes (max: {})",
                                    msg_len, MAX_MESSAGE_SIZE
                                ),
                            ));
                        }

                        // Transition to payload state
                        self.state = ReadState::WaitingForPayload {
                            expected_len: msg_len,
                            bytes_read: 0,
                            buffer: Vec::with_capacity(msg_len as usize),
                        };
                    }
                }

                ReadState::WaitingForPayload {
                    expected_len,
                    bytes_read,
                    buffer,
                } => {
                    let needed = (*expected_len as usize) - *bytes_read;
                    let available = self.buffer.len();

                    if available == 0 {
                        return Ok(None); // Need more data
                    }

                    let to_copy = needed.min(available);

                    // Copy bytes to payload buffer and remove from internal buffer
                    buffer.extend_from_slice(&self.buffer[..to_copy]);
                    self.buffer.drain(..to_copy);
                    *bytes_read += to_copy;

                    // If we have complete message, extract it
                    if *bytes_read == *expected_len as usize {
                        let message = std::mem::take(buffer);

                        // Reset to waiting for next header
                        self.state = ReadState::default();

                        return Ok(Some(message));
                    }
                }
            }
        }
    }

    /// Reset the frame reader state.
    ///
    /// This is useful when the underlying connection is reset or
    /// when you want to discard a partially-read message.
    pub fn reset(&mut self) {
        self.state = ReadState::default();
        self.buffer.clear();
    }
}

impl Default for FrameReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Write a length-prefixed frame to the given writer.
///
/// # Arguments
///
/// * `writer` - The writer to send the frame to
/// * `data` - The payload data to send
///
/// # Errors
///
/// Returns error if:
/// - Data length exceeds MAX_MESSAGE_SIZE
/// - I/O error occurs
///
/// # Example
///
/// ```rust,ignore
/// use weston_ivi_controller::rpc::framing::write_frame;
/// use std::io::Cursor;
///
/// let mut buffer = Vec::new();
/// write_frame(&mut buffer, b"hello world").unwrap();
/// assert_eq!(buffer.len(), 4 + 11); // 4-byte header + 11-byte payload
/// ```
pub fn write_frame<W: Write>(writer: &mut W, data: &[u8]) -> io::Result<()> {
    // Validate message size
    let msg_len = data.len();
    if msg_len > MAX_MESSAGE_SIZE as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Message too large: {} bytes (max: {})",
                msg_len, MAX_MESSAGE_SIZE
            ),
        ));
    }

    // Prepare length prefix (4 bytes, big-endian)
    let len_bytes = (msg_len as u32).to_be_bytes();

    // Send length prefix
    writer.write_all(&len_bytes)?;

    // Send payload
    writer.write_all(data)?;

    // Flush
    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_read_frame() {
        let mut buffer = Vec::new();
        let data = b"test message";

        // Write frame
        write_frame(&mut buffer, data).unwrap();

        // Verify buffer format
        assert_eq!(buffer.len(), 4 + data.len());
        assert_eq!(&buffer[0..4], &(data.len() as u32).to_be_bytes());
        assert_eq!(&buffer[4..], data);

        // Read frame
        let mut reader = FrameReader::new();
        let mut cursor = Cursor::new(&buffer);
        let message = reader.read_frame(&mut cursor).unwrap();

        assert_eq!(message, FrameReadResult::Complete(data.to_vec()));
    }

    #[test]
    fn test_partial_header_read() {
        let mut buffer = Vec::new();
        write_frame(&mut buffer, b"test").unwrap();

        let mut reader = FrameReader::new();

        // Read only 2 bytes of header
        let mut cursor = Cursor::new(&buffer[0..2]);
        assert_eq!(
            reader.read_frame(&mut cursor).unwrap(),
            FrameReadResult::Eof
        );

        // Read remaining 2 bytes of header + payload
        let mut cursor = Cursor::new(&buffer[2..]);
        let message = reader.read_frame(&mut cursor).unwrap();
        assert_eq!(message, FrameReadResult::Complete(b"test".to_vec()));
    }

    #[test]
    fn test_partial_payload_read() {
        let data = b"test message with some length";
        let mut buffer = Vec::new();
        write_frame(&mut buffer, data).unwrap();

        let mut reader = FrameReader::new();

        // Read header + partial payload (10 bytes)
        let mut cursor = Cursor::new(&buffer[0..14]);
        assert_eq!(
            reader.read_frame(&mut cursor).unwrap(),
            FrameReadResult::Eof
        );

        // Read remaining payload
        let mut cursor = Cursor::new(&buffer[14..]);
        let message = reader.read_frame(&mut cursor).unwrap();
        assert_eq!(message, FrameReadResult::Complete(data.to_vec()));
    }

    #[test]
    fn test_multiple_messages() {
        let mut buffer = Vec::new();
        write_frame(&mut buffer, b"first").unwrap();
        write_frame(&mut buffer, b"second").unwrap();

        let mut reader = FrameReader::new();
        let mut cursor = Cursor::new(&buffer);

        // Read first message
        let msg1 = reader.read_frame(&mut cursor).unwrap();
        assert_eq!(msg1, FrameReadResult::Complete(b"first".to_vec()));

        // Read second message
        let msg2 = reader.read_frame(&mut cursor).unwrap();
        assert_eq!(msg2, FrameReadResult::Complete(b"second".to_vec()));

        // No more messages
        let msg3 = reader.read_frame(&mut cursor).unwrap();
        assert_eq!(msg3, FrameReadResult::Eof);
    }

    #[test]
    fn test_zero_length_message() {
        let buffer = [0u8, 0, 0, 0]; // Zero-length header
        let mut reader = FrameReader::new();
        let mut cursor = Cursor::new(&buffer);

        let result = reader.read_frame(&mut cursor);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_message_too_large() {
        // Try to write a message that's too large
        let large_data = vec![0u8; (MAX_MESSAGE_SIZE + 1) as usize];
        let mut buffer = Vec::new();

        let result = write_frame(&mut buffer, &large_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_reader_reset() {
        let mut buffer = Vec::new();
        write_frame(&mut buffer, b"test").unwrap();

        let mut reader = FrameReader::new();

        // Read partial header
        let mut cursor = Cursor::new(&buffer[0..2]);
        assert_eq!(
            reader.read_frame(&mut cursor).unwrap(),
            FrameReadResult::Eof
        );

        // Reset reader
        reader.reset();

        // Should be able to read from beginning again
        let mut cursor = Cursor::new(&buffer);
        let message = reader.read_frame(&mut cursor).unwrap();
        assert_eq!(message, FrameReadResult::Complete(b"test".to_vec()));
    }
}
