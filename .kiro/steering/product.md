# Product Overview

Weston IVI Controller is a Rust-based shared library plugin for the Weston compositor that provides programmatic control over IVI (In-Vehicle Infotainment) surfaces through a JSON-RPC interface over UNIX domain sockets.

## Purpose

Enables external applications to control Wayland client applications in an IVI environment, providing a safe and modular architecture for automotive display management.

## Key Capabilities

- Control surface geometry (position, size)
- Manage visibility and opacity
- Adjust orientation (0°, 90°, 180°, 270°)
- Control z-order (stacking)
- Route input focus (keyboard and pointer)
- Query surface state and properties
- Support multiple concurrent client connections
- Atomic updates to prevent visual tearing

## Architecture

The plugin loads into Weston as a shared library and exposes the IVI layout API through a JSON-RPC interface accessible via UNIX domain sockets. It uses memory-safe Rust with C FFI for Weston integration.
