# IVI Client Library Documentation

## Overview

The IVI Client Library (`ivi-client`) is a reusable Rust library with C FFI bindings that provides programmatic access to the Weston IVI Controller's JSON-RPC API. It serves as the foundation for building applications that need to control IVI surfaces and layers.

## Architecture

### High-Level Design

```
┌────────────────────────────────────────────────────────────────────┐
│                        External Applications                       │
├──────────────────────────┬─────────────────────────────────────────┤
│     Rust Applications    │          C Applications                 │
│     (including CLI)      │                                         │
│           │              │               │                         │
│           ▼              │               ▼                         │
│   ┌───────────────┐      │    ┌──────────────────┐                 │
│   │   Rust API    │      │    │    C FFI API     │                 │
│   │               │      │    │                  │                 │
│   │  IviClient    │      │    │  ivi_client_*    │                 │
│   │  Notification │      │    │  ivi_notif_*     │                 │
│   │  Listener     │      │    │                  │                 │
│   └───────┬───────┘      │    └────────┬─────────┘                 │
│           │              │             │                           │
│           └──────────────┴─────────────┘                           │
│                          │                                         │
│              ┌───────────▼──────────┐                              │
│              │    IVI Client        │                              │
│              │      Library         │                              │
│              │                      │                              │
│              │ - IviClient (RPC)    │                              │
│              │ - Notification       │                              │
│              │   Listener (events)  │                              │
│              │ - Protocol           │                              │
│              │ - Transport          │                              │
│              └───────────┬──────────┘                              │
└──────────────────────────┼─────────────────────────────────────────┘
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

### Component Layers

#### 1. Public API Layer

**Rust API (`client.rs`)**
- `IviClient` — synchronous request/response operations
- `NotificationListener` — callback-based event subscription with background thread
- Uses `Result<T, IviError>` for error handling
- Manages connection lifecycle

**C FFI API (`ffi.rs`)**
- `ivi_client_*` — synchronous request/response operations
- `ivi_notification_listener_*` — event subscription and callback registration
- Uses error codes and error buffers
- Manages memory allocation/deallocation
- Translates between Rust and C types

#### 2. Protocol Layer (`protocol.rs`)

- Implements JSON-RPC 2.0 protocol
- Defines request and response structures
- Handles serialization/deserialization
- Validates protocol compliance

#### 3. Transport Layer

- UNIX domain socket communication
- Newline-delimited JSON messages
- Buffered I/O for efficiency
- Connection management

#### 4. Type System (`types.rs`)

- Strongly-typed data structures
- Serde serialization support
- Display implementations for debugging
- Validation logic

## Integration with Controller Plugin

### Communication Flow

1. **Client Initialization**
   ```
   Application → IviClient::connect() → UnixStream::connect()
   ```

2. **Request Processing**
   ```
   Application → client.list_surfaces()
              → send_request("list_surfaces", {})
              → JsonRpcRequest serialization
              → Socket write (4-byte length prefix + JSON body)
              → Socket read (length-prefixed frame)
              → JsonRpcResponse deserialization
              → Result<Vec<Surface>>
   ```

3. **Notification Flow**
   ```
   Application → NotificationListener::new()  (own socket connection)
              → listener.on(SurfaceCreated, |n| { ... })
              → listener.start(&[SurfaceCreated])
              → subscribe RPC sent to server
              → background thread started
              → server sends {"method":"notification","params":{...}}
              → background thread receives frame
              → Notification::try_from_frame() parses it
              → registered callbacks invoked
   ```

3. **Error Handling**
   ```
   Controller Error → JSON-RPC error response
                   → IviError::RequestFailed
                   → Application error handling
   ```

### Protocol Compatibility

The library implements the same JSON-RPC protocol as documented in `control_interface.md`:

- **Request Format**: `{"id": <number>, "method": "<method>", "params": {...}}`
- **Response Format**: `{"id": <number>, "result": {...}}` or `{"id": <number>, "error": {...}}`
- **Notification Format**: `{"method": "notification", "params": {"event_type": "<type>", ...}}` (no `id` field)
- **Transport**: UNIX domain socket with 4-byte big-endian length-prefixed framing
- **Default Socket**: `/tmp/weston-ivi-controller.sock`

`IviClient` and `NotificationListener` each open their own independent socket connection. This keeps synchronous RPC responses and unsolicited notifications from interleaving on the same socket.

## API Design Principles

### Rust API

1. **Type Safety**
   - Strong typing for all IVI objects
   - Compile-time validation where possible
   - No raw pointers in public API

2. **Error Handling**
   - `Result<T, IviError>` for all fallible operations
   - Descriptive error messages
   - Error context preservation

3. **Resource Management**
   - RAII for connection lifecycle
   - Automatic cleanup on drop
   - No manual resource management required

4. **Ergonomics**
   - Fluent API design
   - Sensible defaults
   - Clear method names

### C FFI API

1. **Safety**
   - Null pointer checks
   - Bounds validation
   - Error buffer overflow protection

2. **Memory Management**
   - Explicit allocation/deallocation
   - Clear ownership semantics
   - Memory leak prevention

3. **Error Reporting**
   - Error codes for programmatic handling
   - Error messages for debugging
   - Consistent error patterns

4. **Compatibility**
   - Standard C types
   - No C++ dependencies
   - Platform-independent

## Data Flow

### Surface Query Example

```
┌─────────────┐
│ Application │
└──────┬──────┘
       │ client.get_surface(1000)
       ▼
┌─────────────┐
│ IviClient   │
└──────┬──────┘
       │ send_request("get_surface", {"id": 1000})
       ▼
┌─────────────┐
│ Protocol    │ Serialize: {"id":1,"method":"get_surface","params":{"id":1000}}
└──────┬──────┘
       │ Write frame (4-byte length + JSON body)
       ▼
┌─────────────┐
│ Transport   │
└──────┬──────┘
       │ UNIX Socket
       ▼
┌─────────────┐
│ Controller  │ Process request
└──────┬──────┘
       │ {"id":1,"result":{"id":1000,"src_rect":{...},"dest_rect":{...},...}}
       ▼
┌─────────────┐
│ Transport   │
└──────┬──────┘
       │ Read length-prefixed frame
       ▼
┌─────────────┐
│ Protocol    │ Deserialize: JsonRpcResponse
└──────┬──────┘
       │ Extract result
       ▼
┌─────────────┐
│ IviClient   │ Parse as Surface
└──────┬──────┘
       │ Ok(Surface { id: 1000, ... })
       ▼
┌─────────────┐
│ Application │
└─────────────┘
```

### Error Flow Example

```
┌─────────────┐
│ Application │
└──────┬──────┘
       │ client.get_surface(99999)
       ▼
┌─────────────┐
│ IviClient   │
└──────┬──────┘
       │ send_request("get_surface", {"id": 99999})
       ▼
┌─────────────┐
│ Controller  │ Surface not found
└──────┬──────┘
       │ {"id":1,"error":{"code":-32000,"message":"Surface not found: 99999"}}
       ▼
┌─────────────┐
│ Protocol    │ Deserialize error
└──────┬──────┘
       │ JsonRpcError { code: -32000, message: "..." }
       ▼
┌─────────────┐
│ IviClient   │ Convert to IviError
└──────┬──────┘
       │ Err(IviError::RequestFailed { code: -32000, message: "..." })
       ▼
┌─────────────┐
│ Application │ Handle error
└─────────────┘
```

## Event Notifications

The library supports receiving unsolicited event notifications from the controller via `NotificationListener`. It opens a dedicated socket connection separate from `IviClient` and dispatches callbacks from a background thread.

### Event Types

| Rust `EventType`             | C `IviEventType`               | Triggered when                              |
|------------------------------|--------------------------------|---------------------------------------------|
| `SurfaceCreated`             | `SURFACE_CREATED`              | A new surface is created                    |
| `SurfaceContentReady`        | `SURFACE_CONTENT_READY`        | Surface commits its first buffer            |
| `SurfaceContentSizeChanged`  | `SURFACE_CONTENT_SIZE_CHANGED` | Surface buffer dimensions change            |
| `SurfaceDestroyed`           | `SURFACE_DESTROYED`            | A surface is destroyed                      |
| `SourceGeometryChanged`      | `SOURCE_GEOMETRY_CHANGED`      | Surface source rectangle changes            |
| `DestinationGeometryChanged` | `DESTINATION_GEOMETRY_CHANGED` | Surface destination rectangle changes       |
| `VisibilityChanged`          | `VISIBILITY_CHANGED`           | Surface visibility changes                  |
| `OpacityChanged`             | `OPACITY_CHANGED`              | Surface opacity changes                     |
| `OrientationChanged`         | `ORIENTATION_CHANGED`          | Surface orientation changes                 |
| `ZOrderChanged`              | `Z_ORDER_CHANGED`              | Surface z-order changes                     |
| `FocusChanged`               | `FOCUS_CHANGED`                | Input focus moves to a different surface    |
| `LayerCreated`               | `LAYER_CREATED`                | A new layer is created                      |
| `LayerDestroyed`             | `LAYER_DESTROYED`              | A layer is destroyed                        |
| `LayerVisibilityChanged`     | `LAYER_VISIBILITY_CHANGED`     | Layer visibility changes                    |
| `LayerOpacityChanged`        | `LAYER_OPACITY_CHANGED`        | Layer opacity changes                       |

### `IviNotification` Structure (C)

```c
typedef struct IviNotification {
    IviEventType         event_type;
    IviObjectType        object_type;   // SURFACE or LAYER
    uint32_t             object_id;     // surface/layer ID; for FocusChanged: new focused surface
    uint32_t             object_old_id; // for FocusChanged: previous focused surface (else 0)
    IviVisibilityChange  visibility;    // { old_visibility, new_visibility }
    IviOpacityChange     opacity;       // { old_opacity, new_opacity }
    IviGeometryChange    src_geometry;  // { old_rect, new_rect }
    IviGeometryChange    dest_geometry; // { old_rect, new_rect }
    IviZOrderChange      z_order;       // { old_z_order, new_z_order }
    IviOrientationChange orientation;   // { old_orientation, new_orientation }
    IviContentReadyInfo  content_ready; // { width, height } — SurfaceContentReady only
    IviContentSizeChange content_size;  // { old_width, old_height, new_width, new_height } — SurfaceContentSizeChanged only
} IviNotification;
```

Only the fields relevant to `event_type` are populated; all others are zero-initialised. The C user should check `event_type` before reading property sub-structs.

### Rust API Example

```rust
use ivi_client::{EventType, NotificationListener};

let mut listener = NotificationListener::new(None)?;

// Register a per-type callback
listener.on(EventType::SurfaceCreated, |notif| {
    let id = notif.params["surface_id"].as_u64().unwrap_or(0);
    println!("Surface created: {}", id);
});

// Register a catch-all callback
listener.on_all(|notif| {
    println!("Event: {:?}", notif.event_type);
});

// Subscribe and start background thread
listener.start(&[
    EventType::SurfaceCreated,
    EventType::SurfaceDestroyed,
    EventType::VisibilityChanged,
])?;

// ... callbacks fire in the background thread ...

listener.stop(); // or just drop `listener`
```

### C API Example

```c
#include "ivi_client.h"
#include <stdio.h>

void on_surface_created(const IviNotification* notif, void* user_data) {
    printf("Surface created: %u\n", notif->object_id);
}

void on_visibility_changed(const IviNotification* notif, void* user_data) {
    printf("Surface %u visibility: %d -> %d\n",
           notif->object_id,
           notif->visibility.old_visibility,
           notif->visibility.new_visibility);
}

void on_focus_changed(const IviNotification* notif, void* user_data) {
    printf("Focus: %u -> %u\n", notif->object_old_id, notif->object_id);
}

int main(void) {
    char err[256];

    NotificationListener* listener =
        ivi_notification_listener_new(NULL, err, sizeof(err));
    if (!listener) { fprintf(stderr, "%s\n", err); return 1; }

    ivi_notification_listener_on(listener, SURFACE_CREATED,    on_surface_created,    NULL);
    ivi_notification_listener_on(listener, VISIBILITY_CHANGED, on_visibility_changed, NULL);
    ivi_notification_listener_on(listener, FOCUS_CHANGED,      on_focus_changed,      NULL);

    IviEventType types[] = { SURFACE_CREATED, VISIBILITY_CHANGED, FOCUS_CHANGED };
    if (ivi_notification_listener_start(listener, types, 3, err, sizeof(err)) != OK) {
        fprintf(stderr, "start failed: %s\n", err);
        ivi_notification_listener_free(listener);
        return 1;
    }

    // ... listener fires callbacks in background thread ...

    ivi_notification_listener_stop(listener);
    ivi_notification_listener_free(listener);
    return 0;
}
```

### Notification Listener Lifecycle (C)

| Step | Function                           | Description                                  |
|------|------------------------------------|----------------------------------------------|
| 1    | `ivi_notification_listener_new()`  | Open dedicated socket connection             |
| 2    | `ivi_notification_listener_on()`   | Register per-type callback (repeat as needed)|
| 2    | `ivi_notification_listener_on_all()`| Register catch-all callback (optional)      |
| 3    | `ivi_notification_listener_start()`| Subscribe + start background reader thread  |
| 4    | `ivi_notification_listener_stop()` | Stop background thread (optional before free)|
| 5    | `ivi_notification_listener_free()` | Stop thread, disconnect, free memory         |

## Thread Safety

### Rust API

The `IviClient` struct is **not** thread-safe by default:
- Uses `&mut self` for operations
- Single connection per client instance
- Not `Send` or `Sync`

For multi-threaded applications:
- Create separate client instances per thread
- Use `Arc<Mutex<IviClient>>` for shared access
- Consider connection pooling for high concurrency

`NotificationListener` is thread-aware by design:
- Callbacks are registered before calling `start()` (single-threaded setup)
- The background reader thread is started by `start()`
- Callbacks fire from the background thread — ensure they are thread-safe
- `stop()` blocks until the background thread exits cleanly

### C FFI API

The C API is thread-safe with proper usage:
- Each `IviClient*` handle is independent
- Each `NotificationListener*` handle is independent
- No shared state between handles
- Caller responsible for synchronization if sharing handles
- Callbacks registered with `ivi_notification_listener_on*` fire from a background thread

## Memory Management

### Rust API

Memory is managed automatically:
- RAII for connection lifecycle
- Automatic cleanup on drop
- No manual memory management

### C FFI API

Memory management is explicit:

**Allocation:**
- `ivi_client_connect()` allocates client handle
- `ivi_list_surfaces()` allocates surface array
- `ivi_list_layers()` allocates layer array
- `ivi_notification_listener_new()` allocates listener handle (includes its own socket connection)

**Deallocation:**
- `ivi_client_disconnect()` frees client handle
- `ivi_free_surfaces()` frees surface array
- `ivi_free_layers()` frees layer array
- `ivi_notification_listener_free()` stops the background thread, closes the socket, and frees the listener handle

**Rules:**
1. Always free allocated resources
2. Don't use handles after freeing
3. Don't free the same resource twice
4. Error buffers are caller-allocated

## Error Handling Patterns

### Rust Error Handling

```rust
use ivi_client::{IviClient, IviError};

// Pattern 1: Early return with ?
fn example1() -> Result<(), IviError> {
    let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))?;
    let surfaces = client.list_surfaces()?;
    Ok(())
}

// Pattern 2: Match on specific errors
fn example2() {
    match IviClient::new(Some("/tmp/weston-ivi-controller.sock")) {
        Ok(mut client) => {
            // Use client
        }
        Err(IviError::ConnectionFailed(msg)) => {
            eprintln!("Connection failed: {}", msg);
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
        }
    }
}

// Pattern 3: Unwrap with context
fn example3() {
    let mut client = IviClient::new(Some("/tmp/weston-ivi-controller.sock"))
        .expect("Failed to connect to IVI controller");
}
```

### C Error Handling

```c
// Pattern 1: Check return codes
IviErrorCode result = ivi_get_surface(client, 1000, &surface, error_buf, sizeof(error_buf));
if (result != IVI_OK) {
    fprintf(stderr, "Error: %s\n", error_buf);
    return 1;
}

// Pattern 2: Check for NULL
IviClient* client = ivi_client_connect(socket_path, error_buf, sizeof(error_buf));
if (client == NULL) {
    fprintf(stderr, "Connection failed: %s\n", error_buf);
    return 1;
}

// Pattern 3: Switch on error codes
switch (result) {
    case OK:
        // Success
        break;
    case CONNECTION_FAILED:
        // Handle connection error
        break;
    case REQUEST_FAILED:
        // Handle request error
        break;
    default:
        // Handle unknown error
        break;
}
```

## Performance Considerations

### Connection Reuse

Reuse connections for multiple requests:

```rust
// Good: Reuse connection
let mut client = IviClient::new(Some(socket_path))?;
for id in surface_ids {
    client.set_surface_visibility(id, true, false)?;
}
client.commit()?;

// Bad: Create new connection each time
for id in surface_ids {
    let mut client = IviClient::new(Some(socket_path))?;
    client.set_surface_visibility(id, true, true)?;
}
```

### Batch Operations

Use atomic commits for multiple changes:

```rust
// Good: Batch with commit
client.set_surface_destination_rectangle(1000, 100, 200, 800, 600, false)?;
client.set_surface_visibility(1000, true, false)?;
client.commit()?; // Apply all at once

// Bad: Individual commits
client.set_surface_destination_rectangle(1000, 100, 200, 800, 600, true)?;
client.set_surface_visibility(1000, true, true)?;
```

### Memory Efficiency

In C, free resources promptly:

```c
// Good: Free immediately after use
IviSurface* surfaces = NULL;
size_t count = 0;
ivi_list_surfaces(client, &surfaces, &count, error_buf, sizeof(error_buf));
// Use surfaces...
ivi_free_surfaces(surfaces); // Free immediately

// Bad: Accumulate allocations
for (int i = 0; i < 100; i++) {
    IviSurface* surfaces = NULL;
    size_t count = 0;
    ivi_list_surfaces(client, &surfaces, &count, error_buf, sizeof(error_buf));
    // Use surfaces...
    // Forgot to free - memory leak!
}
```

## Testing

### Unit Tests

The library includes comprehensive unit tests:

```bash
cd ivi-client
cargo test
```

Tests cover:
- Type serialization/deserialization
- Error handling and conversion
- Request ID generation
- Protocol compliance

### Integration Tests

Integration tests require a running IVI controller:

```bash
# Start Weston with IVI controller
weston &

# Run integration tests
cargo test --test integration_test
```

### C API Tests

C API tests verify FFI bindings:

```bash
# Build C tests
gcc -o c_api_test tests/c_api_tests.c -L./target/release -livi_client

# Run C tests
LD_LIBRARY_PATH=./target/release ./c_api_test
```

## Building and Linking

### Building the Library

```bash
cd ivi-client
cargo build --release
```

Outputs:
- `target/release/libivi_client.so` - Shared library
- `target/release/libivi_client.a` - Static library
- `include/ivi_client.h` - C header (auto-generated)

### Linking in Rust Projects

Add to `Cargo.toml`:

```toml
[dependencies]
ivi-client = { path = "../ivi-client" }
```

### Linking in C Projects

**Dynamic linking:**
```bash
gcc -o myapp myapp.c -L./target/release -livi_client -lpthread -ldl -lm
export LD_LIBRARY_PATH=./target/release:$LD_LIBRARY_PATH
./myapp
```

**Static linking:**
```bash
gcc -o myapp myapp.c ./target/release/libivi_client.a -lpthread -ldl -lm
./myapp
```

### CMake Integration

```cmake
# Find the library
find_library(IVI_CLIENT_LIB ivi_client HINTS ${CMAKE_SOURCE_DIR}/target/release)

# Include headers
include_directories(${CMAKE_SOURCE_DIR}/include)

# Link against library
target_link_libraries(myapp ${IVI_CLIENT_LIB} pthread dl m)
```

## Extending the Library

### Adding New Methods

To add support for new controller methods:

1. **Add method to client.rs:**
   ```rust
   pub fn new_method(&mut self, param: Type) -> Result<ReturnType> {
       use serde_json::json;
       let result = self.send_request("new_method", json!({ "param": param }))?;
       let parsed: ReturnType = serde_json::from_value(result)?;
       Ok(parsed)
   }
   ```

2. **Add C FFI binding in ffi.rs:**
   ```rust
   #[no_mangle]
   pub extern "C" fn ivi_new_method(
       client: *mut IviClient,
       param: CType,
       error_buf: *mut c_char,
       error_buf_len: usize,
   ) -> IviErrorCode {
       // Implementation
   }
   ```

3. **Update C header:**
   ```bash
   cargo build  # Regenerates header with cbindgen
   ```

4. **Add tests:**
   ```rust
   #[test]
   fn test_new_method() {
       // Test implementation
   }
   ```

### Adding New Types

To add new data types:

1. **Define in types.rs:**
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct NewType {
       pub field: Type,
   }
   ```

2. **Add C-compatible type in ffi.rs:**
   ```rust
   #[repr(C)]
   pub struct CNewType {
       pub field: CType,
   }
   ```

3. **Add conversion functions:**
   ```rust
   impl From<NewType> for CNewType { ... }
   impl From<CNewType> for NewType { ... }
   ```

## Best Practices

### Rust Applications

1. **Use Result types properly**
   - Don't unwrap in library code
   - Propagate errors with `?`
   - Provide context in error messages

2. **Manage connections efficiently**
   - Reuse connections when possible
   - Close connections explicitly
   - Handle connection failures gracefully

3. **Batch operations**
   - Use atomic commits
   - Minimize round trips
   - Group related changes

### C Applications

1. **Check all return values**
   - Never ignore error codes
   - Always check for NULL
   - Read error messages

2. **Manage memory carefully**
   - Free all allocated resources
   - Don't use freed pointers
   - Provide adequate error buffers

3. **Handle errors gracefully**
   - Log error messages
   - Clean up on failure
   - Provide user feedback

## Troubleshooting

### Connection Issues

**Problem:** `ConnectionFailed` error

**Solutions:**
- Verify controller is running
- Check socket path is correct
- Verify socket permissions
- Check for firewall/SELinux issues

### Serialization Errors

**Problem:** `SerializationError` or `DeserializationError`

**Solutions:**
- Verify data types match protocol
- Check for invalid UTF-8
- Validate JSON structure
- Update library version

### Request Failures

**Problem:** `RequestFailed` with error code

**Solutions:**
- Check error message for details
- Verify surface/layer IDs exist
- Validate parameter ranges
- Check controller logs

## Future Enhancements

Potential improvements for future versions:

1. **Async API** - Add async/await support using tokio
2. **Connection Pooling** - Built-in connection pool for multi-threaded apps
3. **Retry Logic** - Automatic retry with exponential backoff
4. **Connection Monitoring** - Health checks and reconnection
5. **Batch API** - Send multiple requests in one round trip

## References

- [IVI Client Library README](../ivi-client/README.md)
- [Control Interface Documentation](control_interface.md)
- [Project README](../README.md)
- [Rust API Documentation](https://docs.rs/ivi-client) (when published)
