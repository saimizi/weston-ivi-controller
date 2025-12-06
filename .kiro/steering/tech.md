# Technology Stack

## Language & Build System

- **Language**: Rust (edition 2021, minimum version 1.70)
- **Build System**: Cargo
- **Library Type**: cdylib (C-compatible dynamic library)

## Core Dependencies

- `libc` - C FFI bindings
- `serde` + `serde_json` - JSON serialization
- `jlogger-tracing` + `tracing` + `tracing-subscriber` - Logging framework
- `thiserror` - Error handling
- `mio` - Non-blocking I/O for transport layer
- `lazy_static` - Static initialization

## Build Dependencies

- `bindgen` - Generates Rust FFI bindings from C headers

## Development Dependencies

- `proptest` - Property-based testing

## External Requirements

- Weston compositor with IVI shell support
- IVI layout header files (`ivi-layout-export.h`)

## Common Commands

```bash
# Build the shared library
cargo build --release

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Enable debug logging
export RUST_LOG=weston_ivi_controller=debug

# Enable trace logging for specific modules
export RUST_LOG=weston_ivi_controller::rpc=trace
```

## Build Output

The compiled plugin is located at `target/release/libweston_ivi_controller.so`

## Installation

Copy the plugin to Weston's plugin directory (typically `/usr/lib/weston/`) and configure in `weston.ini`:

```ini
[core]
shell=ivi-shell.so
modules=weston_ivi_controller.so
```

Note: Reference without 'lib' prefix in config (standard Linux convention).
