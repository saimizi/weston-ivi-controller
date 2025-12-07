# IVI Client Library

A Rust library with C FFI bindings for interacting with the Weston IVI Controller. This library provides both a safe Rust API and C-compatible FFI bindings for controlling IVI (In-Vehicle Infotainment) surfaces and layers through a JSON-RPC interface over UNIX domain sockets.

## Features

- **Dual API**: Native Rust API and C-compatible FFI bindings
- **Type-safe**: Strongly typed data structures for surfaces, layers, and operations
- **Error handling**: Comprehensive error types with detailed messages
- **Connection management**: Persistent socket connections with automatic cleanup
- **JSON-RPC protocol**: Full implementation of the IVI controller protocol
- **Thread-safe**: Safe for use in multi-threaded applications

## Installation

### Rust Projects

Add this to your `Cargo.toml`:

```toml
[dependencies]
ivi-client = { path = "path/to/ivi-client" }
```

### C Projects

The library can be built as a static or shared library:

```bash
cd ivi-client
cargo build --release
```

This produces:
- `target/release/libivi_client.so` - Shared library
- `target/release/libivi_client.a` - Static library
- `include/ivi_client.h` - C header file

Link against the library in your C project:

```bash
gcc -o myapp myapp.c -L./target/release -livi_client -lpthread -ldl -lm
```

## Rust API Usage

### Basic Example

```rust
use ivi_client::{IviClient, Result};

fn main() -> Result<()> {
    // Connect to the IVI controller
    let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;
    
    // List all surfaces
    let surfaces = client.list_surfaces()?;
    println!("Found {} surfaces", surfaces.len());
    
    for surface in surfaces {
        println!("Surface {}: {}x{} at ({}, {})",
            surface.id,
            surface.size.width,
            surface.size.height,
            surface.position.x,
            surface.position.y
        );
    }
    
    // Modify a surface
    if let Some(surface) = surfaces.first() {
        client.set_surface_visibility(surface.id, true)?;
        client.set_surface_opacity(surface.id, 0.8)?;
        client.commit()?;
    }
    
    Ok(())
}
```

### Surface Operations

```rust
use ivi_client::{IviClient, Orientation};

let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;

// Get specific surface
let surface = client.get_surface(1000)?;

// Modify surface properties
client.set_surface_position(1000, 100, 200)?;
client.set_surface_size(1000, 1920, 1080)?;
client.set_surface_visibility(1000, true)?;
client.set_surface_opacity(1000, 1.0)?;
client.set_surface_orientation(1000, Orientation::Rotate90)?;
client.set_surface_z_order(1000, 10)?;
client.set_surface_focus(1000)?;

// Commit changes atomically
client.commit()?;
```

### Layer Operations

```rust
let mut client = IviClient::connect("/tmp/weston-ivi-controller.sock")?;

// List all layers
let layers = client.list_layers()?;

// Get specific layer
let layer = client.get_layer(2000)?;

// Modify layer properties
client.set_layer_visibility(2000, true)?;
client.set_layer_opacity(2000, 0.9)?;
client.commit()?;
```

### Error Handling

```rust
use ivi_client::{IviClient, IviError};

match IviClient::connect("/tmp/weston-ivi-controller.sock") {
    Ok(mut client) => {
        match client.get_surface(1000) {
            Ok(surface) => println!("Surface found: {:?}", surface),
            Err(IviError::RequestFailed { code, message }) => {
                eprintln!("Request failed (code {}): {}", code, message);
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Err(IviError::ConnectionFailed(msg)) => {
        eprintln!("Connection failed: {}", msg);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## C API Usage

### Basic Example

```c
#include "ivi_client.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    char error_buf[256];
    
    // Connect to the IVI controller
    IviClient* client = ivi_client_connect(
        "/tmp/weston-ivi-controller.sock",
        error_buf,
        sizeof(error_buf)
    );
    
    if (client == NULL) {
        fprintf(stderr, "Connection failed: %s\n", error_buf);
        return 1;
    }
    
    // List all surfaces
    IviSurface* surfaces = NULL;
    size_t count = 0;
    IviErrorCode result = ivi_list_surfaces(
        client,
        &surfaces,
        &count,
        error_buf,
        sizeof(error_buf)
    );
    
    if (result != IVI_OK) {
        fprintf(stderr, "Failed to list surfaces: %s\n", error_buf);
        ivi_client_disconnect(client);
        return 1;
    }
    
    printf("Found %zu surfaces\n", count);
    
    for (size_t i = 0; i < count; i++) {
        printf("Surface %u: %ux%u at (%d, %d)\n",
            surfaces[i].id,
            surfaces[i].size.width,
            surfaces[i].size.height,
            surfaces[i].position.x,
            surfaces[i].position.y
        );
    }
    
    // Clean up
    ivi_free_surfaces(surfaces);
    ivi_client_disconnect(client);
    
    return 0;
}
```

### Surface Operations

```c
IviClient* client = ivi_client_connect(
    "/tmp/weston-ivi-controller.sock",
    error_buf,
    sizeof(error_buf)
);

// Get specific surface
IviSurface surface;
IviErrorCode result = ivi_get_surface(
    client,
    1000,
    &surface,
    error_buf,
    sizeof(error_buf)
);

if (result == IVI_OK) {
    printf("Surface visibility: %s\n", surface.visibility ? "true" : "false");
    printf("Surface opacity: %.2f\n", surface.opacity);
}

// Modify surface properties
ivi_set_surface_position(client, 1000, 100, 200, error_buf, sizeof(error_buf));
ivi_set_surface_size(client, 1000, 1920, 1080, error_buf, sizeof(error_buf));
ivi_set_surface_visibility(client, 1000, true, error_buf, sizeof(error_buf));
ivi_set_surface_opacity(client, 1000, 1.0, error_buf, sizeof(error_buf));
ivi_set_surface_orientation(client, 1000, IVI_ORIENTATION_ROTATE90, error_buf, sizeof(error_buf));

// Commit changes
ivi_commit(client, error_buf, sizeof(error_buf));
```

### Memory Management

The C API requires explicit memory management:

```c
// Arrays allocated by the library must be freed
IviSurface* surfaces = NULL;
size_t count = 0;
ivi_list_surfaces(client, &surfaces, &count, error_buf, sizeof(error_buf));

// Use the surfaces...

// Free when done
ivi_free_surfaces(surfaces);

// Client handles must be disconnected
ivi_client_disconnect(client);
```

### Error Handling

```c
char error_buf[256];
IviErrorCode result = ivi_get_surface(
    client,
    1000,
    &surface,
    error_buf,
    sizeof(error_buf)
);

switch (result) {
    case IVI_OK:
        printf("Success\n");
        break;
    case IVI_ERR_CONNECTION_FAILED:
        fprintf(stderr, "Connection failed: %s\n", error_buf);
        break;
    case IVI_ERR_REQUEST_FAILED:
        fprintf(stderr, "Request failed: %s\n", error_buf);
        break;
    case IVI_ERR_INVALID_PARAM:
        fprintf(stderr, "Invalid parameter\n");
        break;
    default:
        fprintf(stderr, "Error: %s\n", error_buf);
        break;
}
```

## Building and Linking

### Building the Library

```bash
# Build release version
cargo build --release

# Build with debug symbols
cargo build

# Run tests
cargo test

# Generate documentation
cargo doc --open
```

### Linking in C Projects

#### Dynamic Linking

```bash
gcc -o myapp myapp.c -L./target/release -livi_client -lpthread -ldl -lm
export LD_LIBRARY_PATH=./target/release:$LD_LIBRARY_PATH
./myapp
```

#### Static Linking

```bash
gcc -o myapp myapp.c ./target/release/libivi_client.a -lpthread -ldl -lm
./myapp
```

### CMake Integration

```cmake
find_library(IVI_CLIENT_LIB ivi_client HINTS ${CMAKE_SOURCE_DIR}/target/release)
include_directories(${CMAKE_SOURCE_DIR}/include)
target_link_libraries(myapp ${IVI_CLIENT_LIB} pthread dl m)
```

## API Reference

### Rust Types

- `IviClient` - Client connection handle
- `Surface` - Surface data structure
- `Layer` - Layer data structure
- `Position` - X/Y coordinates
- `Size` - Width/height dimensions
- `Orientation` - Rotation enum (Normal, Rotate90, Rotate180, Rotate270)
- `IviError` - Error type
- `Result<T>` - Result type alias

### C Types

- `IviClient` - Opaque client handle
- `IviSurface` - Surface data structure
- `IviLayer` - Layer data structure
- `IviPosition` - X/Y coordinates
- `IviSize` - Width/height dimensions
- `IviOrientation` - Rotation enum
- `IviErrorCode` - Error code enum

See the generated documentation for complete API details:

```bash
cargo doc --open
```

## Requirements

- Rust 1.70 or later
- Weston compositor with IVI shell support
- IVI controller plugin running and listening on UNIX socket

## Examples

See the `examples/` directory for complete working examples:

- `rust_example.rs` - Rust API usage example
- `c_example.c` - C API usage example

Build and run examples:

```bash
# Rust example
cargo run --example rust_example

# C example
gcc -o c_example examples/c_example.c -L./target/release -livi_client -lpthread -ldl -lm
./c_example
```

## License

MIT

## Contributing

Contributions are welcome! Please ensure all tests pass before submitting:

```bash
cargo test
cargo clippy
cargo fmt
```
