# Design Document

## Overview

The IVI CLI project provides programmatic access to the Weston IVI controller through a reusable client library, with a command-line tool as a reference implementation and usage example.

**IVI Client Library (`ivi-client`)** - A Rust library with C FFI bindings that implements the control interface to `weston-ivi-controller`. The library provides a type-safe, ergonomic interface to the IVI controller's JSON-RPC API, handling all communication, connection management, and data serialization/deserialization.

**IVI CLI Tool (`ivi_cli`)** - A command-line application that demonstrates how to use the IVI client library. It serves both as a practical tool for developers and system administrators, and as a reference example showing how Rust applications can integrate the library.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     External Applications                    │
├──────────────────────┬──────────────────────────────────────┤
│   Rust Applications  │         C Applications               │
│   (including CLI)    │                                      │
│         │            │              │                        │
│         ▼            │              ▼                        │
│   ┌──────────┐      │      ┌──────────────┐               │
│   │ Rust API │      │      │   C FFI API  │               │
│   └────┬─────┘      │      └──────┬───────┘               │
│        │            │              │                        │
│        └────────────┴──────────────┘                        │
│                     │                                        │
│              ┌──────▼──────┐                                │
│              │ IVI Client  │                                │
│              │   Library   │                                │
│              │             │                                │
│              │ Implements  │                                │
│              │  Control    │                                │
│              │ Interface   │                                │
│              └──────┬──────┘                                │
└─────────────────────┼─────────────────────────────────────┘
                      │
         ┌────────────▼────────────┐
         │   UNIX Domain Socket    │
         │ /tmp/weston-ivi-        │
         │    controller.sock      │
         └────────────┬────────────┘
                      │
         ┌────────────▼────────────┐
         │  weston-ivi-controller  │
         │      (JSON-RPC)         │
         └────────────┬────────────┘
                      │
         ┌────────────▼────────────┐
         │   Weston IVI Shell      │
         └─────────────────────────┘
```

### Component Responsibilities

**IVI Client Library:**
- Implement the control interface to `weston-ivi-controller`
- Establish and manage UNIX socket connections
- Serialize Rust data structures to JSON-RPC requests
- Deserialize JSON-RPC responses to Rust data structures
- Provide type-safe Rust API for Rust applications
- Provide C-compatible FFI bindings for C applications
- Handle errors and translate them between Rust and C
- Manage request IDs for JSON-RPC protocol

**CLI Tool (Example Application):**
- Demonstrate how to use the IVI client library from Rust
- Parse command-line arguments using `clap`
- Validate user input before calling library functions
- Call library functions to perform IVI operations
- Format and display results to the user
- Handle errors and display user-friendly messages
- Provide help and usage information
- Serve as a practical tool for developers and administrators

## Components and Interfaces

### IVI Client Library

#### Rust API

The Rust API provides an idiomatic, safe interface for Rust applications.

**Core Types:**

```rust
// Client connection
pub struct IviClient {
    socket: UnixStream,
    request_id: AtomicU64,
}

// Surface data
pub struct Surface {
    pub id: u32,
    pub position: Position,
    pub size: Size,
    pub visibility: bool,
    pub opacity: f32,
    pub orientation: Orientation,
    pub z_order: i32,
}

// Layer data
pub struct Layer {
    pub id: u32,
    pub visibility: bool,
    pub opacity: f32,
}

// Supporting types
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Size {
    pub width: u32,
    pub height: u32,
}

pub enum Orientation {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
}

// Error type
pub enum IviError {
    ConnectionFailed(String),
    RequestFailed { code: i32, message: String },
    SerializationError(String),
    DeserializationError(String),
    IoError(std::io::Error),
}

pub type Result<T> = std::result::Result<T, IviError>;
```

**Client API:**

```rust
impl IviClient {
    // Connection management
    pub fn connect(socket_path: &str) -> Result<Self>;
    pub fn disconnect(self) -> Result<()>;
    
    // Surface operations
    pub fn list_surfaces(&mut self) -> Result<Vec<Surface>>;
    pub fn get_surface(&mut self, id: u32) -> Result<Surface>;
    pub fn set_surface_position(&mut self, id: u32, x: i32, y: i32) -> Result<()>;
    pub fn set_surface_size(&mut self, id: u32, width: u32, height: u32) -> Result<()>;
    pub fn set_surface_visibility(&mut self, id: u32, visible: bool) -> Result<()>;
    pub fn set_surface_opacity(&mut self, id: u32, opacity: f32) -> Result<()>;
    pub fn set_surface_orientation(&mut self, id: u32, orientation: Orientation) -> Result<()>;
    pub fn set_surface_z_order(&mut self, id: u32, z_order: i32) -> Result<()>;
    pub fn set_surface_focus(&mut self, id: u32) -> Result<()>;
    
    // Layer operations
    pub fn list_layers(&mut self) -> Result<Vec<Layer>>;
    pub fn get_layer(&mut self, id: u32) -> Result<Layer>;
    pub fn set_layer_visibility(&mut self, id: u32, visible: bool) -> Result<()>;
    pub fn set_layer_opacity(&mut self, id: u32, opacity: f32) -> Result<()>;
    
    // Commit
    pub fn commit(&mut self) -> Result<()>;
    
    // Internal helpers
    fn send_request(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value>;
    fn next_request_id(&self) -> u64;
}
```

#### C FFI API

The C API provides a compatible interface for C applications.

**C Header (`ivi_client.h`):**

```c
#ifndef IVI_CLIENT_H
#define IVI_CLIENT_H

#include <stdint.h>
#include <stdbool.h>

// Opaque client handle
typedef struct IviClient IviClient;

// Error codes
typedef enum {
    IVI_OK = 0,
    IVI_ERR_CONNECTION_FAILED = -1,
    IVI_ERR_REQUEST_FAILED = -2,
    IVI_ERR_SERIALIZATION = -3,
    IVI_ERR_DESERIALIZATION = -4,
    IVI_ERR_IO = -5,
    IVI_ERR_INVALID_PARAM = -6,
} IviErrorCode;

// Orientation enum
typedef enum {
    IVI_ORIENTATION_NORMAL = 0,
    IVI_ORIENTATION_ROTATE90 = 1,
    IVI_ORIENTATION_ROTATE180 = 2,
    IVI_ORIENTATION_ROTATE270 = 3,
} IviOrientation;

// Position struct
typedef struct {
    int32_t x;
    int32_t y;
} IviPosition;

// Size struct
typedef struct {
    uint32_t width;
    uint32_t height;
} IviSize;

// Surface struct
typedef struct {
    uint32_t id;
    IviPosition position;
    IviSize size;
    bool visibility;
    float opacity;
    IviOrientation orientation;
    int32_t z_order;
} IviSurface;

// Layer struct
typedef struct {
    uint32_t id;
    bool visibility;
    float opacity;
} IviLayer;

// Connection management
IviClient* ivi_client_connect(const char* socket_path, char* error_buf, size_t error_buf_len);
void ivi_client_disconnect(IviClient* client);

// Surface operations
IviErrorCode ivi_list_surfaces(IviClient* client, IviSurface** surfaces, size_t* count, char* error_buf, size_t error_buf_len);
IviErrorCode ivi_get_surface(IviClient* client, uint32_t id, IviSurface* surface, char* error_buf, size_t error_buf_len);
IviErrorCode ivi_set_surface_position(IviClient* client, uint32_t id, int32_t x, int32_t y, char* error_buf, size_t error_buf_len);
IviErrorCode ivi_set_surface_size(IviClient* client, uint32_t id, uint32_t width, uint32_t height, char* error_buf, size_t error_buf_len);
IviErrorCode ivi_set_surface_visibility(IviClient* client, uint32_t id, bool visible, char* error_buf, size_t error_buf_len);
IviErrorCode ivi_set_surface_opacity(IviClient* client, uint32_t id, float opacity, char* error_buf, size_t error_buf_len);
IviErrorCode ivi_set_surface_orientation(IviClient* client, uint32_t id, IviOrientation orientation, char* error_buf, size_t error_buf_len);
IviErrorCode ivi_set_surface_z_order(IviClient* client, uint32_t id, int32_t z_order, char* error_buf, size_t error_buf_len);
IviErrorCode ivi_set_surface_focus(IviClient* client, uint32_t id, char* error_buf, size_t error_buf_len);

// Layer operations
IviErrorCode ivi_list_layers(IviClient* client, IviLayer** layers, size_t* count, char* error_buf, size_t error_buf_len);
IviErrorCode ivi_get_layer(IviClient* client, uint32_t id, IviLayer* layer, char* error_buf, size_t error_buf_len);
IviErrorCode ivi_set_layer_visibility(IviClient* client, uint32_t id, bool visible, char* error_buf, size_t error_buf_len);
IviErrorCode ivi_set_layer_opacity(IviClient* client, uint32_t id, float opacity, char* error_buf, size_t error_buf_len);

// Commit
IviErrorCode ivi_commit(IviClient* client, char* error_buf, size_t error_buf_len);

// Memory management
void ivi_free_surfaces(IviSurface* surfaces);
void ivi_free_layers(IviLayer* layers);

#endif // IVI_CLIENT_H
```

### CLI Tool

#### Command Structure

The CLI uses a hierarchical command structure:

```
ivi_cli [global-options] <resource> <command> [command-options] [arguments]
```

**Resources:**
- `surface` - Surface management commands
- `layer` - Layer management commands
- `commit` - Commit pending changes

**Global Options:**
- `--socket <path>` - Custom socket path (default: `/tmp/weston-ivi-controller.sock`)
- `--help` - Display help information
- `--version` - Display version information

#### Command Parsing

The CLI uses the `clap` crate for argument parsing with subcommands:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ivi_cli")]
#[command(about = "IVI Controller Command-Line Interface")]
struct Cli {
    #[arg(long, default_value = "/tmp/weston-ivi-controller.sock")]
    socket: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Surface {
        #[command(subcommand)]
        command: SurfaceCommands,
    },
    Layer {
        #[command(subcommand)]
        command: LayerCommands,
    },
    Commit {
        #[arg(long)]
        socket: Option<String>,
    },
}

#[derive(Subcommand)]
enum SurfaceCommands {
    List,
    GetProperties { id: u32 },
    SetVisibility { id: u32, visible: bool },
    SetOpacity { id: u32, opacity: f32 },
    SetDestRect { id: u32, x: i32, y: i32, width: u32, height: u32 },
}

#[derive(Subcommand)]
enum LayerCommands {
    List,
    GetProperties { id: u32 },
    SetVisibility { id: u32, visible: bool },
    SetOpacity { id: u32, opacity: f32 },
}
```

#### Output Formatting

The CLI formats output for readability:

**List Output:**
```
Surface IDs: 1000, 1001, 1002
```

**Properties Output:**
```
Surface 1000:
  Position: (100, 200)
  Size: 1920x1080
  Visibility: true
  Opacity: 1.0
  Orientation: Normal
  Z-Order: 0
```

**Success Output:**
```
✓ Surface 1000 visibility set to true
```

**Error Output:**
```
✗ Error: Surface not found: 1234 (code: -32000)
```

## Data Models

### JSON-RPC Protocol

The library implements the JSON-RPC 2.0 protocol for communication with the controller.

**Request Format:**
```json
{
  "id": <number>,
  "method": "<method_name>",
  "params": { ... }
}
```

**Response Format (Success):**
```json
{
  "id": <number>,
  "result": { ... }
}
```

**Response Format (Error):**
```json
{
  "id": <number>,
  "error": {
    "code": <error_code>,
    "message": "<error_message>"
  }
}
```

### Internal Data Flow

1. **CLI receives command** → Parse arguments with clap
2. **CLI validates input** → Check ranges, types, required fields
3. **CLI calls library function** → Pass validated parameters
4. **Library creates JSON-RPC request** → Serialize with serde_json
5. **Library sends request** → Write to UNIX socket with newline
6. **Library receives response** → Read from socket until newline
7. **Library parses response** → Deserialize with serde_json
8. **Library returns result** → Rust Result type or C error code
9. **CLI formats output** → Display to user with formatting
10. **CLI exits** → Return appropriate exit code


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Request-Response Round Trip

*For any* valid IVI controller request, serializing the request to JSON-RPC format, sending it, receiving the response, and deserializing it should produce a valid result or error without data corruption.

**Validates: Requirements 2.3, 2.4**

### Property 2: Error Propagation Consistency

*For any* error condition in the library, the error SHALL be properly represented in the Result type with a descriptive message that includes the error code and context.

**Validates: Requirements 2.5**

### Property 3: FFI Type Translation Consistency

*For any* C API function call with valid parameters, the result SHALL be equivalent to calling the corresponding Rust API function directly with the same parameters.

**Validates: Requirements 3.2, 3.3**

### Property 4: FFI Error Translation

*For any* error condition that occurs in the Rust implementation when called through the C API, the C API SHALL return a non-zero error code and populate the error message buffer with a descriptive message.

**Validates: Requirements 3.4**

### Property 5: Connection Reusability

*For any* sequence of valid requests, sending them all through the same client connection SHALL succeed without requiring reconnection.

**Validates: Requirements 5.2**

## Error Handling

### Library Error Handling

The library uses Rust's `Result` type for error handling:

```rust
pub enum IviError {
    ConnectionFailed(String),
    RequestFailed { code: i32, message: String },
    SerializationError(String),
    DeserializationError(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for IviError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IviError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            IviError::RequestFailed { code, message } => {
                write!(f, "Request failed (code {}): {}", code, message)
            }
            IviError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            IviError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            IviError::IoError(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for IviError {}

impl From<std::io::Error> for IviError {
    fn from(err: std::io::Error) -> Self {
        IviError::IoError(err)
    }
}

impl From<serde_json::Error> for IviError {
    fn from(err: serde_json::Error) -> Self {
        IviError::SerializationError(err.to_string())
    }
}
```

### C API Error Handling

The C API uses error codes and error message buffers:

```c
// Example usage
char error_buf[256];
IviClient* client = ivi_client_connect("/tmp/weston-ivi-controller.sock", error_buf, sizeof(error_buf));
if (client == NULL) {
    fprintf(stderr, "Connection failed: %s\n", error_buf);
    return 1;
}

IviSurface surface;
IviErrorCode result = ivi_get_surface(client, 1000, &surface, error_buf, sizeof(error_buf));
if (result != IVI_OK) {
    fprintf(stderr, "Failed to get surface: %s\n", error_buf);
    ivi_client_disconnect(client);
    return 1;
}
```

### CLI Error Handling

The CLI displays user-friendly error messages and exits with appropriate codes:

```rust
fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Surface { command } => handle_surface_command(&cli.socket, command),
        Commands::Layer { command } => handle_layer_command(&cli.socket, command),
        Commands::Commit { .. } => handle_commit(&cli.socket),
    };
    
    match result {
        Ok(output) => {
            println!("{}", output);
            std::process::exit(0);
        }
        Err(err) => {
            eprintln!("✗ Error: {}", err);
            std::process::exit(1);
        }
    }
}
```

**Exit Codes:**
- `0` - Success
- `1` - General error (connection failed, invalid parameters, request failed)

## Testing Strategy

### Unit Testing

Unit tests verify specific functionality of individual components:

**Library Unit Tests:**
- Connection establishment with valid and invalid socket paths
- JSON-RPC request serialization for each method
- JSON-RPC response deserialization for success and error cases
- Error type conversions (io::Error → IviError, serde_json::Error → IviError)
- Request ID generation and uniqueness

**CLI Unit Tests:**
- Argument parsing for all commands
- Input validation (opacity ranges, positive dimensions)
- Output formatting for different data types
- Error message formatting

**Example Unit Test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_opacity_validation() {
        assert!(validate_opacity(0.0).is_ok());
        assert!(validate_opacity(0.5).is_ok());
        assert!(validate_opacity(1.0).is_ok());
        assert!(validate_opacity(-0.1).is_err());
        assert!(validate_opacity(1.1).is_err());
    }
    
    #[test]
    fn test_request_serialization() {
        let request = create_set_opacity_request(1, 1000, 0.75);
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"method\":\"set_opacity\""));
        assert!(json.contains("\"id\":1000"));
        assert!(json.contains("\"opacity\":0.75"));
    }
}
```

### Property-Based Testing

Property-based tests verify universal properties across many randomly generated inputs using the `proptest` crate.

**Testing Framework:** `proptest` (Rust property-based testing library)

**Configuration:** Each property test runs a minimum of 100 iterations to ensure thorough coverage.

**Property Test Examples:**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_request_response_round_trip(
        method in prop::sample::select(vec!["list_surfaces", "list_layers", "commit"]),
        request_id in 1u64..1000000u64
    ) {
        // **Feature: ivi-cli, Property 1: Request-Response Round Trip**
        let request = JsonRpcRequest {
            id: request_id,
            method: method.to_string(),
            params: serde_json::json!({}),
        };
        
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();
        
        prop_assert_eq!(request.id, deserialized.id);
        prop_assert_eq!(request.method, deserialized.method);
    }
    
    #[test]
    fn prop_error_propagation_consistency(
        error_code in -32700i32..-32000i32,
        error_message in ".*"
    ) {
        // **Feature: ivi-cli, Property 2: Error Propagation Consistency**
        let error = IviError::RequestFailed {
            code: error_code,
            message: error_message.clone(),
        };
        
        let error_string = error.to_string();
        prop_assert!(error_string.contains(&error_code.to_string()));
        prop_assert!(error_string.contains(&error_message));
    }
    
    #[test]
    fn prop_connection_reusability(
        num_requests in 1usize..20usize
    ) {
        // **Feature: ivi-cli, Property 5: Connection Reusability**
        // This test would require a mock server
        // Verify that multiple requests can be sent on the same connection
        // without errors or reconnection
    }
}
```

### Integration Testing

Integration tests verify the complete system behavior:

**CLI Integration Tests:**
- End-to-end command execution with mock IVI controller
- Error handling for various failure scenarios
- Output formatting verification
- Exit code verification

**Library Integration Tests:**
- Communication with real or mock IVI controller
- Multiple request sequences
- Error recovery
- Resource cleanup

**Example Integration Test:**
```rust
#[test]
fn test_cli_surface_list() {
    // Start mock IVI controller
    let mock_server = start_mock_server();
    
    // Run CLI command
    let output = Command::new("ivi_cli")
        .args(&["--socket", mock_server.socket_path(), "surface", "list"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Surface IDs:"));
    
    mock_server.stop();
}
```

### C API Testing

C API tests verify the FFI bindings work correctly:

**Test Approach:**
- Write C test programs that use the C API
- Compile and link against the library
- Run tests and verify output
- Compare behavior with Rust API tests

**Example C Test:**
```c
#include "ivi_client.h"
#include <assert.h>
#include <stdio.h>

void test_connection() {
    char error_buf[256];
    IviClient* client = ivi_client_connect("/tmp/test.sock", error_buf, sizeof(error_buf));
    assert(client != NULL);
    ivi_client_disconnect(client);
}

void test_list_surfaces() {
    char error_buf[256];
    IviClient* client = ivi_client_connect("/tmp/test.sock", error_buf, sizeof(error_buf));
    assert(client != NULL);
    
    IviSurface* surfaces = NULL;
    size_t count = 0;
    IviErrorCode result = ivi_list_surfaces(client, &surfaces, &count, error_buf, sizeof(error_buf));
    
    assert(result == IVI_OK);
    assert(count >= 0);
    
    if (surfaces) {
        ivi_free_surfaces(surfaces);
    }
    
    ivi_client_disconnect(client);
}

int main() {
    test_connection();
    test_list_surfaces();
    printf("All C API tests passed\n");
    return 0;
}
```

### Test Coverage Goals

- **Unit Tests:** Cover all public API functions and error paths
- **Property Tests:** Cover all identified correctness properties (minimum 5 properties)
- **Integration Tests:** Cover all CLI commands and common usage patterns
- **C API Tests:** Cover all C FFI functions and memory management

### Testing Requirements

- All property-based tests MUST run at least 100 iterations
- Each property test MUST be tagged with a comment referencing the design document property
- Tag format: `// **Feature: ivi-cli, Property N: <property description>**`
- Tests MUST be written after implementation to verify correctness
- Tests MUST NOT use mocks for core logic (only for external dependencies like the IVI controller)

## Implementation Notes

### Project Structure

The IVI client library and CLI tool will be integrated into the existing `weston-ivi-controller` project:

```
weston-ivi-controller/
├── Cargo.toml                 # Workspace manifest (updated)
├── README.md                  # Project documentation (updated)
├── src/                       # Existing controller plugin code
│   ├── lib.rs
│   ├── controller/
│   ├── rpc/
│   ├── transport/
│   └── ...
├── ivi-client/                # New: Client library crate
│   ├── Cargo.toml
│   ├── README.md
│   ├── src/
│   │   ├── lib.rs            # Rust API
│   │   ├── client.rs         # Client implementation
│   │   ├── types.rs          # Data types
│   │   ├── error.rs          # Error types
│   │   ├── protocol.rs       # JSON-RPC protocol
│   │   └── ffi.rs            # C FFI bindings
│   ├── include/
│   │   └── ivi_client.h      # C header file
│   ├── examples/
│   │   ├── rust_example.rs   # Rust usage example
│   │   └── c_example.c       # C usage example
│   └── tests/
│       └── c_api_tests.c     # C API tests
├── ivi-cli/                   # New: CLI binary crate
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── main.rs           # CLI entry point
│       ├── commands.rs       # Command handlers
│       └── output.rs         # Output formatting
├── docs/
│   ├── control_interface.md  # Existing RPC protocol docs
│   └── client_library.md     # New: Client library docs
└── target/
    ├── release/
    │   ├── libweston_ivi_controller.so  # Existing plugin
    │   ├── libivi_client.so             # New: Client library
    │   ├── libivi_client.a              # New: Static library
    │   └── ivi_cli                      # New: CLI binary
    └── ...
```

### Dependencies

**Existing Controller Plugin:**
- All existing dependencies remain unchanged

**Library (`ivi-client`):**
- `serde` + `serde_json` - JSON serialization (already in workspace)
- `thiserror` - Error handling (already in workspace)
- `proptest` (dev) - Property-based testing

**CLI (`ivi-cli`):**
- `ivi-client` - The library (workspace member)
- `clap` - Command-line argument parsing

### Build Configuration

**Cargo.toml (workspace root - updated):**
```toml
[workspace]
members = [".", "ivi-client", "ivi-cli"]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
libc = "0.2"
mio = "0.8"
jlogger-tracing = "0.1"
tracing = "0.1"
tracing-subscriber = "0.3"
lazy_static = "1.4"
```

**Cargo.toml (ivi-client):**
```toml
[package]
name = "ivi-client"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib", "staticlib", "cdylib"]

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
proptest = "1.0"

[build-dependencies]
cbindgen = "0.24"
```

**Cargo.toml (ivi-cli):**
```toml
[package]
name = "ivi-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "ivi_cli"
path = "src/main.rs"

[dependencies]
ivi-client = { path = "../ivi-client" }
clap = { version = "4.0", features = ["derive"] }
```

### C Header Generation

Use `cbindgen` to automatically generate the C header from Rust code:

**cbindgen.toml:**
```toml
language = "C"
include_guard = "IVI_CLIENT_H"
autogen_warning = "/* Warning: This file is auto-generated. Do not edit manually. */"
include_version = true
namespace = "ivi"
```

**build.rs:**
```rust
extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("include/ivi_client.h");
}
```

### Memory Management in C API

The C API must carefully manage memory to prevent leaks:

**Allocation Strategy:**
- Arrays (surfaces, layers) are allocated by Rust and freed by C
- Strings in error buffers are written by Rust into caller-provided buffers
- Client handles are opaque pointers managed by Rust

**Example Implementation:**
```rust
#[no_mangle]
pub extern "C" fn ivi_list_surfaces(
    client: *mut IviClient,
    surfaces: *mut *mut IviSurface,
    count: *mut usize,
    error_buf: *mut c_char,
    error_buf_len: usize,
) -> IviErrorCode {
    if client.is_null() || surfaces.is_null() || count.is_null() {
        return IviErrorCode::InvalidParam;
    }
    
    let client = unsafe { &mut *client };
    
    match client.list_surfaces() {
        Ok(surface_list) => {
            let boxed_slice = surface_list.into_boxed_slice();
            unsafe {
                *count = boxed_slice.len();
                *surfaces = Box::into_raw(boxed_slice) as *mut IviSurface;
            }
            IviErrorCode::Ok
        }
        Err(err) => {
            write_error_to_buffer(&err, error_buf, error_buf_len);
            IviErrorCode::RequestFailed
        }
    }
}

#[no_mangle]
pub extern "C" fn ivi_free_surfaces(surfaces: *mut IviSurface) {
    if !surfaces.is_null() {
        unsafe {
            let _ = Box::from_raw(surfaces);
        }
    }
}
```

## Security Considerations

### Input Validation

- **CLI:** Validate all user inputs before passing to library (ranges, types, required fields)
- **Library:** Validate all parameters before serializing to JSON-RPC
- **C API:** Check for null pointers and invalid parameters

### Socket Security

- UNIX domain sockets inherit file system permissions
- Default socket path `/tmp/weston-ivi-controller.sock` should have appropriate permissions
- Consider allowing socket path configuration for security-sensitive deployments

### Error Information Disclosure

- Error messages should be descriptive but not leak sensitive information
- Avoid including internal paths or system details in error messages
- Log detailed errors internally, show user-friendly messages externally

## Performance Considerations

### Connection Pooling

The library maintains a single connection per client instance. For applications that need multiple concurrent connections, create multiple client instances.

### Request Batching

The IVI controller supports atomic commits. Applications should batch multiple operations and commit them together for better performance and consistency.

### Memory Efficiency

- Use streaming JSON parsing for large responses
- Reuse buffers where possible
- Clean up resources promptly (RAII in Rust, explicit free in C)

## Future Enhancements

### Potential Features

1. **Async API** - Add async/await support for non-blocking operations
2. **Event Subscriptions** - Support for receiving notifications from the controller
3. **Connection Pooling** - Built-in connection pool for multi-threaded applications
4. **Configuration File** - Support for CLI configuration file (socket path, defaults)
5. **Shell Completion** - Generate shell completion scripts for bash/zsh/fish
6. **JSON Output Mode** - CLI option to output results as JSON for scripting
7. **Batch Mode** - CLI option to read multiple commands from a file
8. **Interactive Mode** - REPL-style interactive shell for the CLI

### API Stability

- The Rust API follows semantic versioning
- The C API maintains ABI compatibility within major versions
- Breaking changes will be clearly documented in release notes
