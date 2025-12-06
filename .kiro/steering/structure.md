# Project Structure

## Directory Layout

```
weston-ivi-controller/
├── src/                    # Source code
│   ├── lib.rs             # Plugin entry point and FFI exports
│   ├── ffi/               # FFI bindings to Weston IVI API
│   ├── controller/        # Core IVI surface management
│   ├── rpc/               # JSON-RPC protocol layer
│   ├── transport/         # Transport implementations (UNIX sockets)
│   ├── error.rs           # Error types and handling
│   └── logging.rs         # Logging initialization
├── ivi-shell/             # IVI layout header files
├── docs/                  # Documentation
├── build.rs               # Build script (bindgen)
├── Cargo.toml             # Package manifest
└── README.md              # Main documentation
```

## Module Organization

### `src/lib.rs`
Plugin lifecycle management (initialization, cleanup), FFI exports for Weston integration.

### `src/ffi/`
- `bindings.rs` - Generated FFI bindings from IVI layout header
- `mod.rs` - FFI module exports

### `src/controller/`
Core controller logic:
- `state.rs` - State management and surface tracking
- `ivi_wrapper.rs` - Safe Rust wrapper around C IVI API
- `validation.rs` - Input validation for parameters
- `events.rs` - IVI surface lifecycle event handling
- `notifications.rs` - Notification system for state changes

### `src/rpc/`
JSON-RPC protocol implementation:
- `protocol.rs` - Request/response structures and method definitions
- `handler.rs` - Request routing and processing
- `transport.rs` - Transport abstraction layer

### `src/transport/`
Transport layer implementations:
- `unix_socket.rs` - UNIX domain socket transport
- `mod.rs` - Transport trait definitions

## Architectural Patterns

### Layered Architecture
```
External Apps → Transport → RPC → Controller → IVI API → Weston
```

### Safety Boundaries
- All FFI operations isolated in dedicated modules
- Unsafe code carefully documented and validated
- Panic catching at FFI boundaries to prevent unwinding into C

### State Management
- Centralized state in `StateManager` with `Arc<Mutex<>>` for thread safety
- Synchronization with IVI compositor state
- Event-driven updates for surface lifecycle

### Error Handling
- Custom error types using `thiserror`
- JSON-RPC 2.0 compliant error codes
- Detailed error messages for debugging

## Code Conventions

- Use `tracing` macros for logging (`tracing::info!`, `tracing::error!`, etc.)
- Document all public APIs with rustdoc comments
- Mark unsafe code with safety documentation
- Use `Arc` for shared ownership, `Mutex` for interior mutability
- Validate all inputs before passing to IVI API
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)
