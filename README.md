# Weston IVI Controller

A Rust-based shared library plugin for the Weston compositor that provides
programmatic control over IVI (In-Vehicle Infotainment) surfaces through a
JSON-RPC interface over UNIX domain sockets.

## Overview

The Weston IVI Controller project consists of three main components:

1. **Weston IVI Controller Plugin** - A shared library plugin that integrates
   with the Weston compositor to expose IVI control functionality via JSON-RPC
2. **IVI Client Library** (`ivi-client`) - A reusable Rust library with C FFI
   bindings for programmatic access to the IVI controller
3. **IVI CLI Tool** (`ivi_cli`) - A command-line interface for interactive
   control and testing

The controller plugin enables external applications to control Wayland client
applications in an IVI environment. It provides a safe, modular architecture
with:

- **Memory-safe Rust implementation** with C FFI for Weston integration
- **JSON-RPC protocol** for client-server communication
- **Pluggable transport layer** (UNIX domain sockets included)
- **Comprehensive surface control** (position, size, visibility, opacity,
  orientation, z-order, focus)
- **Real-time state tracking** of all IVI surfaces
- **Client library** for easy integration into Rust and C applications
- **Command-line tool** for interactive testing and debugging

## Features

- ✅ Control surface geometry (position and size)
- ✅ Manage surface visibility and opacity
- ✅ Adjust surface orientation (0°, 90°, 180°, 270°)
- ✅ Control z-order (stacking order)
- ✅ Route input focus (keyboard and pointer)
- ✅ Query surface state and properties
- ✅ Multiple concurrent client connections
- ✅ Input validation and error handling
- ✅ Comprehensive logging with tracing

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   External Applications                     │
│                 (RPC Clients via JSON-RPC)                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  Transport Layer                            │
│             (UNIX Domain Socket / Pluggable)                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                     RPC Module                              │
│        (Request Parser → Router → Response Builder)         │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  Controller Core                            │
│        (State Manager + Safe IVI API Wrapper)               │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                 Weston Compositor                           │
│                 (IVI Layout API)                            │
└─────────────────────────────────────────────────────────────┘
```

## Building

### Prerequisites

- Rust 1.70 or later
- Weston compositor with IVI shell support
- IVI layout header files (`ivi-layout-export.h`)
- Pixman development headers (`libpixman-1-dev`)

### Initial Setup

Before building, initialize the Weston submodule and install dependencies:

```bash
# Clone the repository with submodules
git clone --recurse-submodules <repository-url>

# Or if already cloned, initialize submodules
git submodule update --init

# Install required development packages (Debian/Ubuntu)
sudo apt-get install libpixman-1-dev

# For other distributions:
# Fedora/RHEL: sudo dnf install pixman-devel
# Arch: sudo pacman -S pixman
```

### Quick Build

Build all components (controller plugin, client library, and CLI tool):

```bash
# Build everything in release mode (optimized)
cargo build --release --workspace

# Or for development (faster builds, includes debug symbols)
cargo build --workspace

# Run all tests
cargo test --workspace
```

### Build Specific Components

**Controller Plugin Only:**
```bash
cargo build --release
# Output: target/release/libweston_ivi_controller.so
```

**IVI Client Library Only:**
```bash
cargo build --release -p ivi-client
# Outputs:
# - target/release/libivi_client.so    (shared library for C)
# - target/release/libivi_client.a     (static library for C)
# - target/release/libivi_client.rlib  (Rust library)
# - ivi-client/include/ivi_client.h    (C header - auto-generated)
```

**IVI CLI Tool Only:**
```bash
cargo build --release -p ivi-cli
# Output: target/release/ivi_cli
```

### Build Outputs

After a successful build, you'll find:

```
target/release/
├── libweston_ivi_controller.so  # Controller plugin for Weston
├── libivi_client.so             # Client library (shared)
├── libivi_client.a              # Client library (static)
├── libivi_client.rlib           # Client library (Rust)
└── ivi_cli                      # CLI binary

ivi-client/include/
└── ivi_client.h                 # C header (auto-generated by cbindgen)
```

### Build Configuration

**Automatic Code Generation:**
- The controller plugin uses `bindgen` to generate Rust FFI bindings from Weston's IVI layout headers
- The client library uses `cbindgen` to generate C header files from Rust code
- Both processes happen automatically during the build

**Build Scripts:**
- `build.rs` (root) - Generates FFI bindings for Weston IVI API
- `ivi-client/build.rs` - Generates C header file from Rust API

### Development Builds

```bash
# Fast incremental builds for development
cargo build --workspace

# Check code without building
cargo check --workspace

# Build with verbose output
cargo build --release --workspace --verbose

# Clean build artifacts
cargo clean
```

### Building Documentation

```bash
# Generate and open documentation for all workspace members
cargo doc --workspace --no-deps --open

# Generate documentation for specific component
cargo doc -p ivi-client --no-deps --open
```

### Cross-Compilation

To cross-compile for different architectures:

```bash
# Install cross-compilation target
rustup target add aarch64-unknown-linux-gnu

# Build for ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Note: You may need to install cross-compilation toolchain
# and set appropriate linker in .cargo/config.toml
```

## Installation

### Prerequisites

The Weston IVI Controller requires:
- Weston compiled with IVI shell support
- Access to the IVI layout API via `ivi-layout-export.h`

**Note:** This module uses the IVI layout API directly through the exported
header interface. The older `ivi-controller.so` module is not required (and has
been removed from recent Weston versions).

### Installing the Controller Plugin

1. Build the plugin:
   ```bash
   cargo build --release
   ```

2. Copy the plugin to Weston's plugin directory:
   ```bash
   # Common locations (check your system):
   sudo cp target/release/libweston_ivi_controller.so /usr/lib/weston/
   # or
   sudo cp target/release/libweston_ivi_controller.so /usr/lib/x86_64-linux-gnu/weston/
   ```

3. Configure Weston to load the IVI shell and RPC controller by editing `weston.ini`:
   ```ini
   [core]
   # Load the IVI shell
   shell=ivi-shell.so
   
   # Load the RPC controller module
   modules=weston-ivi-controller.so
   ```

4. Restart Weston for changes to take effect:
   ```bash
   # Stop existing Weston instance
   killall weston
   
   # Start Weston with the new configuration
   weston
   ```

### Installing the CLI Tool

Install the CLI tool system-wide:

```bash
# Install from the workspace
cargo install --path ivi-cli

# The binary will be installed to ~/.cargo/bin/ivi_cli
# Make sure ~/.cargo/bin is in your PATH

# Verify installation
ivi_cli --version
ivi_cli --help
```

Or use it directly from the build directory:

```bash
./target/release/ivi_cli --help
```

### Installing the Client Library

**For Rust Projects:**

Add to your `Cargo.toml`:
```toml
[dependencies]
ivi-client = { path = "/path/to/weston-ivi-controller/ivi-client" }
# Or if published to crates.io:
# ivi-client = "0.1"
```

**For C/C++ Projects:**

1. Install the header file:
   ```bash
   sudo cp ivi-client/include/ivi_client.h /usr/local/include/
   ```

2. Install the library:
   ```bash
   # Install shared library
   sudo cp target/release/libivi_client.so /usr/local/lib/
   
   # Install static library (optional)
   sudo cp target/release/libivi_client.a /usr/local/lib/
   
   # Update library cache
   sudo ldconfig
   ```

3. Compile your C application:
   ```bash
   # With dynamic linking
   gcc your_app.c -o your_app -livi_client -L/usr/local/lib -I/usr/local/include
   
   # With static linking
   gcc your_app.c -o your_app /usr/local/lib/libivi_client.a -lpthread -ldl -lm
   ```

### Verifying Installation

**Controller Plugin:**
```bash
# Check if plugin is loaded
lsof | grep weston_ivi_controller

# Check if socket exists
ls -l /tmp/weston-ivi-controller.sock

# Test with netcat
echo '{"id":1,"method":"list_surfaces","params":{}}' | nc -U /tmp/weston-ivi-controller.sock
```

**CLI Tool:**
```bash
# Check version
ivi_cli --version

# List surfaces (requires running Weston with IVI controller)
ivi_cli surface list
```

**Client Library:**
```bash
# Check library symbols (Linux)
nm -D /usr/local/lib/libivi_client.so | grep ivi_

# Run example programs
cargo run --release -p ivi-client --example rust_example
```

### How It Works

The module (built as `libweston_ivi_controller.so`, referenced as
`weston-ivi-controller.so` in config):
- Loads as a Weston plugin and retrieves the IVI layout API directly from the
  compositor
- Uses the `ivi-layout-export.h` interface to control IVI surfaces
- Does not require the deprecated `ivi-controller.so` module
- Provides RPC access to the IVI layout API via UNIX domain sockets

**Note on naming:** The build produces `libweston_ivi_controller.so` and a
symbolic link `weston-ivi-controller.so` to it. Please use
`weston-ivi-controller.so` in the Weston configuration.

## Usage

### Starting the Controller

The controller is automatically loaded when Weston starts. By default, it
creates a UNIX domain socket at `/tmp/weston-ivi-controller.sock` for RPC
communication.

### Using the IVI Client Library

The IVI client library provides both Rust and C APIs for programmatic access:

**Rust API:**
```rust
use ivi_client::{IviClient, Result};

fn main() -> Result<()> {
    let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    let surfaces = client.list_surfaces()?;
    client.set_surface_visibility(1000, true)?;
    client.commit()?;
    Ok(())
}
```

**C API:**
```c
#include "ivi_client.h"

IviClient* client = ivi_client_connect("/tmp/weston-ivi-controller.sock", 
                                       error_buf, sizeof(error_buf));
ivi_list_surfaces(client, &surfaces, &count, error_buf, sizeof(error_buf));
ivi_client_disconnect(client);
```

See [ivi-client/README.md](ivi-client/README.md) for detailed library
documentation.

### Using the CLI Tool

The IVI CLI tool provides interactive command-line access:

```bash
# List surfaces
ivi_cli surface list

# Get surface properties
ivi_cli surface get-properties 1000

# Set surface visibility
ivi_cli surface set-visibility 1000 true

# Commit changes
ivi_cli commit
```

See [ivi-cli/README.md](ivi-cli/README.md) for detailed CLI documentation.

### Connecting Custom Clients

Clients can also connect directly to the UNIX domain socket and communicate
using JSON-RPC 2.0 protocol. See
[docs/control_interface.md](docs/control_interface.md) for detailed protocol
documentation.

### Example: Python Client

```python
import socket
import json

# Connect to the controller
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect('/tmp/weston-ivi-controller.sock')

# List all surfaces
request = {
    "id": 1,
    "method": "list_surfaces",
    "params": {}
}
sock.sendall(json.dumps(request).encode() + b'\n')
response = json.loads(sock.recv(4096).decode())
print(response)

# Set surface position
request = {
    "id": 2,
    "method": "set_position",
    "params": {
        "id": 1000,
        "x": 100,
        "y": 200
    }
}
sock.sendall(json.dumps(request).encode() + b'\n')
response = json.loads(sock.recv(4096).decode())
print(response)

sock.close()
```

### Example: Bash with netcat

```bash
# List surfaces
echo '{"id":1,"method":"list_surfaces","params":{}}' | nc -U /tmp/weston-ivi-controller.sock

# Get surface info
echo '{"id":2,"method":"get_surface","params":{"id":1000}}' | nc -U /tmp/weston-ivi-controller.sock

# Set visibility
echo '{"id":3,"method":"set_visibility","params":{"id":1000,"visible":true}}' | nc -U /tmp/weston-ivi-controller.sock
```

## RPC Methods

The controller supports the following RPC methods:

| Method | Description |
|--------|-------------|
| `list_surfaces` | Get information about all active IVI surfaces |
| `get_surface` | Get properties of a specific surface |
| `set_position` | Update surface position (x, y coordinates) |
| `set_size` | Update surface dimensions (width, height) |
| `set_visibility` | Show or hide a surface |
| `set_opacity` | Adjust surface opacity (0.0 - 1.0) |
| `set_orientation` | Rotate surface (0°, 90°, 180°, 270°) |
| `set_z_order` | Change surface stacking order |
| `set_focus` | Route keyboard and pointer focus to surface |
| `commit` | Commit all pending changes atomically |

### Atomic Updates

By default, surface modification methods (`set_position`, `set_size`, etc.)
**queue changes without committing** them to the compositor. This allows you to
batch multiple operations and apply them atomically using the `commit` method,
preventing visual tearing and lag.

**Example - Atomic move and resize:**
```python
# Queue position change (not visible yet)
send_request({"id": 1, "method": "set_position", "params": {"id": 1000, "x": 100, "y": 200}})

# Queue size change (not visible yet)
send_request({"id": 2, "method": "set_size", "params": {"id": 1000, "width": 800, "height": 600}})

# Commit both changes atomically (visible now)
send_request({"id": 3, "method": "commit", "params": {}})
```

**Auto-commit mode:** For backward compatibility or simple use cases, you can
add `"auto_commit": true` to any modification request to commit immediately:
```python
send_request({"id": 1, "method": "set_position", "params": {"id": 1000, "x": 100, "y": 200, "auto_commit": true}})
```

For detailed protocol documentation, see [docs/control_interface.md](docs/control_interface.md).

## Configuration

### Socket Path

The default socket path is `/tmp/weston-ivi-controller.sock`. This can be
configured by passing arguments to the plugin during Weston initialization.

### Logging

The controller uses the `tracing` framework for logging. Set the `RUST_LOG`
environment variable to control log levels:

```bash
# Enable debug logging
export RUST_LOG=weston_ivi_controller=debug

# Enable trace logging for specific modules
export RUST_LOG=weston_ivi_controller::rpc=trace
```

## Development

### Project Structure

```
weston-ivi-controller/
├── src/                    # Controller plugin source
│   ├── lib.rs              # Plugin entry point and FFI exports
│   ├── ffi/                # FFI bindings
│   │   ├── mod.rs
│   │   └── bindings.rs     # Generated IVI bindings
│   ├── controller/         # Core controller logic
│   │   ├── mod.rs
│   │   ├── state.rs        # State management
│   │   ├── ivi_wrapper.rs  # Safe IVI API wrapper
│   │   ├── validation.rs   # Input validation
│   │   ├── events.rs       # Event handling
│   │   └── notifications.rs # Notification system
│   ├── rpc/                # RPC layer
│   │   ├── mod.rs
│   │   ├── protocol.rs     # Protocol definitions
│   │   └── handler.rs      # Request handler
│   ├── transport/          # Transport implementations
│   │   ├── mod.rs
│   │   └── unix_socket.rs  # UNIX socket transport
│   ├── error.rs            # Error types
│   └── logging.rs          # Logging setup
├── ivi-client/             # Client library
│   ├── src/
│   │   ├── lib.rs          # Rust API
│   │   ├── client.rs       # Client implementation
│   │   ├── types.rs        # Data types
│   │   ├── error.rs        # Error types
│   │   ├── protocol.rs     # JSON-RPC protocol
│   │   └── ffi.rs          # C FFI bindings
│   ├── include/
│   │   └── ivi_client.h    # C header file (auto-generated)
│   ├── examples/           # Usage examples
│   ├── tests/              # Tests
│   ├── build.rs            # Build script (cbindgen)
│   ├── Cargo.toml
│   └── README.md
├── ivi-cli/                # CLI tool
│   ├── src/
│   │   ├── main.rs         # CLI entry point
│   │   ├── commands.rs     # Command handlers
│   │   └── output.rs       # Output formatting
│   ├── Cargo.toml
│   └── README.md
├── docs/
│   ├── control_interface.md # RPC protocol documentation
│   └── client_library.md    # Client library documentation
├── build.rs                # Build script (bindgen)
├── Cargo.toml              # Workspace manifest
└── README.md
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_validate_position
```

### Adding New RPC Methods

1. Add the method variant to `RpcMethod` enum in `src/rpc/protocol.rs`
2. Implement parsing logic in `RpcMethod::from_request()`
3. Add handler method in `src/rpc/handler.rs`
4. Update the router in `RpcHandler::handle_request()`
5. Add tests for the new method

### Adding New Transport Mechanisms

1. Implement the `Transport` trait from `src/rpc/transport.rs`
2. Implement the `MessageHandler` trait for receiving messages
3. Register the transport in plugin initialization (`src/lib.rs`)

## Error Handling

The controller provides detailed error responses for:

- **Invalid parameters** (out of bounds, wrong type, etc.)
- **Surface not found** (non-existent surface ID)
- **IVI API errors** (underlying compositor failures)
- **Transport errors** (connection issues, serialization failures)

All errors include descriptive messages and appropriate error codes following
JSON-RPC 2.0 conventions.

## Safety Considerations

- All FFI boundaries are carefully validated
- Raw pointers from C are checked for null
- Panic unwinding across FFI boundaries is prevented
- Lifetimes of C objects are properly managed
- Thread-safe state management with Mutex

## Performance

- Non-blocking I/O for transport layer (using `mio`)
- Efficient HashMap-based surface lookup
- Minimal allocations in hot paths
- Batched IVI API commits

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file
for details.

## Contributing

[Add contribution guidelines here]

## Troubleshooting

### Plugin Fails to Load

**Symptom:** Weston starts but the RPC controller doesn't work, or you see
errors about missing IVI API.

**Solution:** Ensure the IVI shell is properly configured:
```ini
[core]
shell=ivi-shell.so
modules=weston-ivi-controller.so
```

The IVI shell (`ivi-shell.so`) must be loaded for the controller to access the
IVI layout API.

### Socket Not Created

**Symptom:** The socket `/tmp/weston-ivi-controller.sock` doesn't exist.

**Solution:** 
- Check Weston logs for initialization errors
- Verify the plugin loaded successfully: `lsof | grep weston_ivi_controller`
- Enable debug logging: `RUST_LOG=weston_ivi_controller=debug weston`

### IVI Layout API Not Available

**Symptom:** Error messages about null IVI API pointer or "Failed to get IVI
layout interface".

**Solution:** This means the IVI shell is not loaded or not providing the layout
API. Verify:

```bash
# Check if IVI shell is loaded
ps aux | grep weston

# Check weston.ini configuration
cat ~/.config/weston.ini  # or /etc/weston.ini

# Ensure shell=ivi-shell.so is set in [core] section
```

The controller retrieves the IVI layout API directly from Weston using the
`ivi_layout_interface` exported by the IVI shell.

### Permission Denied on Socket

**Symptom:** Clients cannot connect to the socket.

**Solution:**
- Check socket permissions: `ls -l /tmp/weston-ivi-controller.sock`
- Ensure your user has access to the socket
- Run client with appropriate permissions

### Weston Version Compatibility

**Symptom:** Build errors or runtime failures related to IVI API.

**Solution:** 
- Ensure your `ivi-layout-export.h` matches your Weston version
- The controller is designed for Weston versions that expose the IVI layout API
  directly
- Older Weston versions that required `ivi-controller.so` are not supported

## Support

For issues, questions, or contributions, please [add contact information or
issue tracker link].

## Acknowledgments

- Built with Rust for memory safety and performance
- Uses the Weston IVI shell interface
- Inspired by automotive display management requirements
