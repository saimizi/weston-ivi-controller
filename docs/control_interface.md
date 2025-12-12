# IVI Controller RPC Protocol

This document describes the JSON-RPC 2.0 protocol used to communicate with the Weston IVI Controller module.

## Table of Contents

- [Overview](#overview)
- [Connection](#connection)
- [Message Format](#message-format)
- [Error Codes](#error-codes)
- [RPC Methods](#rpc-methods)
  - [list_layers](#list_layers)
  - [get_layer](#get_layer)
  - [set_layer_source_rectangle](#set_layer_source_rectangle)
  - [set_layer_destination_rectangle](#set_layer_destination_rectangle)
  - [set_layer_visibility](#set_layer_visibility)
  - [set_layer_opacity](#set_layer_opacity)
  - [list_surfaces](#list_surfaces)
  - [get_surface](#get_surface)
  - [set_surface_source_rectangle](#set_surface_source_rectangle)
  - [set_surface_destination_rectangle](#set_surface_destination_rectangle)
  - [set_surface_visibility](#set_surface_visibility)
  - [set_surface_opacity](#set_surface_opacity)
  - [set_surface_z_order](#set_surface_z_order)
  - [set_surface_focus](#set_surface_focus)
  - [commit](#commit)
- [Event Notifications](#event-notifications)
  - [subscribe](#subscribe)
  - [unsubscribe](#unsubscribe)
  - [list_subscriptions](#list_subscriptions)
  - [Notification Format](#notification-format)
- [Data Types](#data-types)
- [Examples](#examples)

## Overview

The IVI Controller uses a JSON-RPC 2.0 protocol over UNIX domain sockets for communication between external applications and the Weston compositor. The protocol is stateless, with each request receiving a corresponding response.

### System Architecture

The RPC controller provides programmatic access to Weston's IVI shell:

```
External App → [UNIX Socket] → weston_ivi_controller.so → IVI Layout API → IVI Shell → Wayland Clients
```

**Required Components:**
- **Weston IVI Shell** (`ivi-shell.so`): Base IVI shell implementation that exports the IVI layout API
- **RPC Controller** (`weston_ivi_controller.so`): This module - provides JSON-RPC interface to the IVI layout API

**Note:** The older `ivi-controller.so` module is not required. This controller uses the IVI layout API directly via the `ivi-layout-export.h` interface, which is exported by the IVI shell itself.

### Protocol Features

- **JSON-RPC 2.0 compliant**: Standard request/response format
- **Newline-delimited messages**: Each message is terminated with `\n`
- **Synchronous responses**: Each request receives exactly one response
- **Multiple concurrent clients**: The controller supports multiple simultaneous connections
- **Comprehensive error reporting**: Detailed error codes and messages
- **Atomic updates**: Batch multiple operations and commit them atomically to prevent visual tearing

### Change Batching and Atomic Updates

By default, surface and layer modification methods (`set_surface_source_rectangle`, `set_surface_destination_rectangle`, `set_surface_visibility`, `set_surface_opacity`, `set_surface_z_order`, `set_surface_focus`, `set_layer_source_rectangle`, `set_layer_destination_rectangle`, `set_layer_visibility`, `set_layer_opacity`) **queue changes without immediately committing** them to the compositor. This allows you to:

1. **Batch multiple operations** on one or more surfaces/layers
2. **Apply changes atomically** using the `commit` method
3. **Prevent visual artifacts** like tearing or intermediate states

**Example workflow:**
```
1. set_surface_destination_rectangle (queued, not visible)
2. set_surface_opacity (queued, not visible)
3. commit → All changes applied atomically
```

**Auto-commit mode:** For simple use cases or backward compatibility, add `"auto_commit": true` to any modification request to commit immediately after that operation.

## Connection

### Socket Path

Default socket path: `/tmp/weston-ivi-controller.sock`

### Connection Steps

1. Create a UNIX domain socket
2. Connect to the socket path
3. Send JSON-RPC requests (newline-terminated)
4. Receive JSON-RPC responses (newline-terminated)
5. Close the connection when done

### Example Connection (Python)

```python
import socket
import json

sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect('/tmp/weston-ivi-controller.sock')

# Send request
request = {"id": 1, "method": "list_surfaces", "params": {}}
sock.sendall(json.dumps(request).encode() + b'\n')

# Receive response
response = json.loads(sock.recv(4096).decode())

sock.close()
```

## Message Format

### Request Format

All requests follow the JSON-RPC 2.0 specification:

```json
{
  "id": <number>,
  "method": "<method_name>",
  "params": {
    <parameter_name>: <parameter_value>,
    ...
  }
}
```

**Fields:**
- `id` (number, required): Unique identifier for the request. The response will contain the same ID.
- `method` (string, required): Name of the RPC method to invoke.
- `params` (object, required): Method-specific parameters.

### Response Format

Successful responses:

```json
{
  "id": <number>,
  "result": {
    <result_data>
  }
}
```

Error responses:

```json
{
  "id": <number>,
  "error": {
    "code": <error_code>,
    "message": "<error_message>"
  }
}
```

**Fields:**
- `id` (number): Matches the request ID
- `result` (object, optional): Present on success, contains method-specific result data
- `error` (object, optional): Present on error, contains error details

## Error Codes

The controller uses standard JSON-RPC 2.0 error codes plus custom application-specific codes:

| Code | Name | Description |
|------|------|-------------|
| -32700 | Parse error | Invalid JSON received |
| -32600 | Invalid request | Request structure is invalid |
| -32601 | Method not found | The requested method does not exist |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Internal controller error |
| -32000 | Surface not found | The specified surface ID does not exist |

### Error Response Examples

**Surface not found:**
```json
{
  "id": 5,
  "error": {
    "code": -32000,
    "message": "Surface not found: 1234"
  }
}
```

**Invalid parameters:**
```json
{
  "id": 3,
  "error": {
    "code": -32602,
    "message": "Invalid opacity value: 1.5 (must be between 0.0 and 1.0)"
  }
}
```

## RPC Methods

### list_surfaces

Get information about all active IVI surfaces.

**Request:**
```json
{
  "id": 1,
  "method": "list_surfaces",
  "params": {}
}
```

**Response:**
```json
{
  "id": 1,
  "result": {
    "surfaces": [
      {
        "id": 1000,
        "orig_size": { "width": 1920, "height": 1080 },
        "src_position": { "x": 0, "y": 0 },
        "src_size": { "width": 1920, "height": 1080 },
        "dest_position": { "x": 0, "y": 0 },
        "dest_size": { "width": 1920, "height": 1080 },
        "visibility": true,
        "opacity": 1.0,
        "orientation": "Normal",
        "z_order": 0
      },
      {
        "id": 1001,
        "orig_size": { "width": 1600, "height": 1200 },
        "src_position": { "x": 0, "y": 0 },
        "src_size": { "width": 1600, "height": 1200 },
        "dest_position": { "x": 100, "y": 100 },
        "dest_size": { "width": 800, "height": 600 },
        "visibility": false,
        "opacity": 0.8,
        "orientation": "Rotate90",
        "z_order": 1
      }
    ]
  }
}
```

**Parameters:** None

**Returns:**
- `surfaces` (array): Array of surface objects, each containing:
  - `id` (number): Surface ID
  - `orig_size` (object): Original application buffer size with `width` and `height`
  - `src_position` (object): Source rectangle position with `x` and `y` coordinates
  - `src_size` (object): Source rectangle size with `width` and `height`
  - `dest_position` (object): Destination rectangle position with `x` and `y` coordinates
  - `dest_size` (object): Destination rectangle size with `width` and `height`
  - `visibility` (boolean): Whether the surface is visible
  - `opacity` (number): Opacity value (0.0 - 1.0)
  - `orientation` (string): Orientation ("Normal", "Rotate90", "Rotate180", "Rotate270")
  - `z_order` (number): Z-order (stacking position)

---

### get_surface

Get properties of a specific IVI surface.

**Request:**
```json
{
  "id": 2,
  "method": "get_surface",
  "params": {
    "id": 1000
  }
}
```

**Response:**
```json
{
  "id": 2,
  "result": {
    "id": 1000,
    "position": { "x": 0, "y": 0 },
    "size": { "width": 1920, "height": 1080 },
    "visibility": true,
    "opacity": 1.0,
    "orientation": "Normal",
    "z_order": 0
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID to query

**Returns:** Surface object with all properties (same structure as in `list_surfaces`)

**Errors:**
- `-32000`: Surface not found

---

### set_surface_source_rectangle

Set the source rectangle of an IVI surface (which part of the application buffer to display).

**Request:**
```json
{
  "id": 3,
  "method": "set_surface_source_rectangle",
  "params": {
    "id": 1000,
    "x": 0,
    "y": 0,
    "width": 1920,
    "height": 1080
  }
}
```

**Response:**
```json
{
  "id": 3,
  "result": {
    "success": true,
    "committed": false
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID
- `x` (number, required): Source X coordinate in buffer
- `y` (number, required): Source Y coordinate in buffer
- `width` (number, required): Source width in pixels (must be positive)
- `height` (number, required): Source height in pixels (must be positive)
- `auto_commit` (boolean, optional): If `true`, commits changes immediately. Default: `false`

**Returns:**
- `success` (boolean): Always `true` on success
- `committed` (boolean): Indicates whether changes were committed

**Errors:**
- `-32000`: Surface not found
- `-32602`: Invalid parameters (coordinates or dimensions invalid)

**Validation:**
- Width and height must be positive non-zero values
- Coordinates must be within buffer bounds

**Behavior:**
- By default (`auto_commit=false`), changes are queued and require a `commit` call
- With `auto_commit=true`, changes are applied immediately

---

### set_surface_destination_rectangle

Set the destination rectangle of an IVI surface (where and at what size to display on screen).

**Request:**
```json
{
  "id": 4,
  "method": "set_surface_destination_rectangle",
  "params": {
    "id": 1000,
    "x": 100,
    "y": 200,
    "width": 1280,
    "height": 720
  }
}
```

**Response:**
```json
{
  "id": 4,
  "result": {
    "success": true,
    "committed": false
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID
- `x` (number, required): Destination X coordinate on screen
- `y` (number, required): Destination Y coordinate on screen
- `width` (number, required): Destination width in pixels (must be positive)
- `height` (number, required): Destination height in pixels (must be positive)
- `auto_commit` (boolean, optional): If `true`, commits changes immediately. Default: `false`

**Returns:**
- `success` (boolean): Always `true` on success
- `committed` (boolean): Indicates whether changes were committed

**Errors:**
- `-32000`: Surface not found
- `-32602`: Invalid parameters (coordinates out of bounds or non-positive dimensions)

**Validation:**
- Width and height must be positive non-zero values
- Position coordinates must be within valid display bounds

**Behavior:**
- By default (`auto_commit=false`), changes are queued and require a `commit` call
- With `auto_commit=true`, changes are applied immediately

---

### set_surface_visibility

Show or hide an IVI surface.

**Request:**
```json
{
  "id": 5,
  "method": "set_surface_visibility",
  "params": {
    "id": 1000,
    "visible": true
  }
}
```

**Response:**
```json
{
  "id": 5,
  "result": {
    "success": true
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID
- `visible` (boolean, required): `true` to show, `false` to hide

**Returns:**
- `success` (boolean): Always `true` on success

**Errors:**
- `-32000`: Surface not found

---

### set_surface_opacity

Adjust the opacity of an IVI surface.

**Request:**
```json
{
  "id": 6,
  "method": "set_surface_opacity",
  "params": {
    "id": 1000,
    "opacity": 0.75
  }
}
```

**Response:**
```json
{
  "id": 6,
  "result": {
    "success": true
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID
- `opacity` (number, required): Opacity value (0.0 = fully transparent, 1.0 = fully opaque)

**Returns:**
- `success` (boolean): Always `true` on success

**Errors:**
- `-32000`: Surface not found
- `-32602`: Invalid parameters (opacity out of range [0.0, 1.0])

**Validation:**
- Opacity must be in the range [0.0, 1.0]

---

### set_surface_z_order

Change the stacking order (z-order) of an IVI surface.

**Request:**
```json
{
  "id": 8,
  "method": "set_surface_z_order",
  "params": {
    "id": 1000,
    "z_order": 5
  }
}
```

**Response:**
```json
{
  "id": 8,
  "result": {
    "success": true
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID
- `z_order` (number, required): Z-order value (higher values appear on top)

**Returns:**
- `success` (boolean): Always `true` on success

**Errors:**
- `-32000`: Surface not found
- `-32602`: Invalid parameters (z-order out of valid range)

**Validation:**
- Z-order must be within the valid range for the layer (typically 0-1000)
- Higher z-order values appear on top of lower values

---

### set_surface_focus

Route keyboard and pointer input focus to an IVI surface.

**Request:**
```json
{
  "id": 9,
  "method": "set_surface_focus",
  "params": {
    "id": 1000
  }
}
```

**Response:**
```json
{
  "id": 9,
  "result": {
    "success": true
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID to receive focus

**Returns:**
- `success` (boolean): Always `true` on success

**Errors:**
- `-32000`: Surface not found

**Behavior:**
- Sets both keyboard and pointer focus to the specified surface
- Removes focus from the previously focused surface
- Focus change notifications are sent to both old and new focused surfaces
- By default, changes are queued (not committed). Use `commit` method or `auto_commit` parameter.

**Optional Parameters:**
- `auto_commit` (boolean, optional): If `true`, commits changes immediately. Default: `false`

---

### commit

Commit all pending surface changes atomically to the compositor.

**Request:**
```json
{
  "id": 10,
  "method": "commit",
  "params": {}
}
```

**Response:**
```json
{
  "id": 10,
  "result": {
    "success": true
  }
}
```

**Parameters:** None

**Returns:**
- `success` (boolean): Always `true` on success

**Errors:**
- `-32603`: Internal error if commit fails

**Behavior:**
- Commits all pending changes from previous `set_*` operations
- Changes are applied atomically - all at once
- Prevents visual tearing and intermediate states
- After commit, all queued changes become visible

**Use Case:**
This method is essential for atomic updates. For example, to reposition and resize a window without showing intermediate states:

```json
// Step 1: Queue destination rectangle change
{"id": 1, "method": "set_surface_destination_rectangle", "params": {"id": 1000, "x": 100, "y": 200, "width": 800, "height": 600}}

// Step 2: Queue opacity change
{"id": 2, "method": "set_surface_opacity", "params": {"id": 1000, "opacity": 0.8}}

// Step 3: Commit all changes atomically
{"id": 3, "method": "commit", "params": {}}
```

Both changes (destination rectangle, opacity) will be applied simultaneously, preventing any visual artifacts.

---

### list_layers

List all tracked IVI layers and their properties.

Request:
```json
{ "id": 100, "method": "list_layers", "params": {} }
```

Response:
```json
{
  "id": 100,
  "result": {
    "layers": [
      { "id": 5000, "visibility": true, "opacity": 1.0 },
      { "id": 5001, "visibility": false, "opacity": 0.5 }
    ]
  }
}
```

---

### get_layer

Get properties of a specific layer.

Request:
```json
{ "id": 101, "method": "get_layer", "params": { "id": 5000 } }
```

Response:
```json
{
  "id": 101,
  "result": { "id": 5000, "visibility": true, "opacity": 1.0 }
}
```

Errors: `-32602` for invalid id

---

### set_layer_source_rectangle

Set the source rectangle of an IVI layer.

Request:
```json
{
  "id": 100,
  "method": "set_layer_source_rectangle",
  "params": { "id": 5000, "x": 0, "y": 0, "width": 1920, "height": 1080 }
}
```

Response:
```json
{ "id": 100, "result": { "success": true, "committed": false } }
```

Optional param: `auto_commit` (bool)

---

### set_layer_destination_rectangle

Set the destination rectangle of an IVI layer.

Request:
```json
{
  "id": 101,
  "method": "set_layer_destination_rectangle",
  "params": { "id": 5000, "x": 0, "y": 0, "width": 1920, "height": 1080 }
}
```

Response:
```json
{ "id": 101, "result": { "success": true, "committed": false } }
```

Optional param: `auto_commit` (bool)

---

### set_layer_visibility

Show or hide a layer.

Request:
```json
{ "id": 102, "method": "set_layer_visibility", "params": { "id": 5000, "visible": true } }
```

Response:
```json
{ "id": 102, "result": { "success": true, "committed": false } }
```

Optional param: `auto_commit` (bool)

---

### set_layer_opacity

Set layer opacity.

Request:
```json
{ "id": 103, "method": "set_layer_opacity", "params": { "id": 5000, "opacity": 0.8 } }
```

Response:
```json
{ "id": 103, "result": { "success": true, "committed": false } }
```

Errors: `-32602` for invalid opacity

---

## Event Notifications

Clients may subscribe to real-time events. Subscriptions are per-client and selective by event type. Each client has a best-effort FIFO buffer (default 100); oldest notifications are dropped when full.

- Delivery: Newline-delimited JSON-RPC notifications (no `id`)
- Filtering: By event type (no per-surface filtering)
- Multiple clients: Supported

Supported event types:
- SurfaceCreated, SurfaceDestroyed, GeometryChanged, VisibilityChanged, OpacityChanged, OrientationChanged, ZOrderChanged, FocusChanged
- LayerCreated, LayerDestroyed, LayerVisibilityChanged, LayerOpacityChanged

### subscribe

Request:
```json
{
  "id": 200,
  "method": "subscribe",
  "params": { "event_types": ["SurfaceCreated", "GeometryChanged", "FocusChanged"] }
}
```

Response:
```json
{
  "id": 200,
  "result": { "success": true, "subscribed": ["SurfaceCreated", "GeometryChanged", "FocusChanged"] }
}
```

### unsubscribe

Request:
```json
{
  "id": 201,
  "method": "unsubscribe",
  "params": { "event_types": ["GeometryChanged"] }
}
```

Response:
```json
{
  "id": 201,
  "result": { "success": true, "unsubscribed": ["GeometryChanged"] }
}
```

### list_subscriptions

Request:
```json
{ "id": 202, "method": "list_subscriptions", "params": {} }
```

Response:
```json
{ "id": 202, "result": { "subscriptions": ["SurfaceCreated", "FocusChanged"] } }
```

### Notification Format

Notifications are JSON-RPC messages with no `id` and method `"notification"`.

Common shape:
```json
{
  "method": "notification",
  "params": { /* event-specific fields */ }
}
```

Examples:
- SurfaceCreated
```json
{ "method": "notification", "params": { "event_type": "SurfaceCreated", "surface_id": 1000 } }
```

- GeometryChanged
```json
{
  "method": "notification",
  "params": {
    "event_type": "GeometryChanged",
    "surface_id": 1000,
    "old_position": {"x": 0, "y": 0},
    "new_position": {"x": 100, "y": 100},
    "old_size": {"width": 1920, "height": 1080},
    "new_size": {"width": 1280, "height": 720}
  }
}
```

- VisibilityChanged
```json
{ "method": "notification", "params": { "event_type": "VisibilityChanged", "surface_id": 1000, "old_visibility": false, "new_visibility": true } }
```

- OpacityChanged
```json
{ "method": "notification", "params": { "event_type": "OpacityChanged", "surface_id": 1000, "old_opacity": 1.0, "new_opacity": 0.7 } }
```

- OrientationChanged
```json
{ "method": "notification", "params": { "event_type": "OrientationChanged", "surface_id": 1000, "old_orientation": "Normal", "new_orientation": "Rotate90" } }
```

- ZOrderChanged
```json
{ "method": "notification", "params": { "event_type": "ZOrderChanged", "surface_id": 1000, "old_z_order": 0, "new_z_order": 5 } }
```

- FocusChanged
```json
{ "method": "notification", "params": { "event_type": "FocusChanged", "old_focused_surface": 1000, "new_focused_surface": 2000 } }
```

- LayerCreated / LayerDestroyed
```json
{ "method": "notification", "params": { "event_type": "LayerCreated", "layer_id": 5000 } }
```

- LayerVisibilityChanged
```json
{ "method": "notification", "params": { "event_type": "LayerVisibilityChanged", "layer_id": 5000, "old_visibility": false, "new_visibility": true } }
```

- LayerOpacityChanged
```json
{ "method": "notification", "params": { "event_type": "LayerOpacityChanged", "layer_id": 5000, "old_opacity": 0.5, "new_opacity": 1.0 } }
```

---

## Understanding Surface Rectangles

IVI surfaces support sophisticated buffer management through three types of size/position information:

- **Original Size (orig_size)**: The native dimensions of the application's Wayland buffer. This represents the actual pixel dimensions provided by the application.

- **Source Rectangle (src_position, src_size)**: Defines which portion of the application buffer to display. This enables cropping - you can display only part of the buffer. For example, you might show only the top-left quarter of a 1920×1080 buffer.

- **Destination Rectangle (dest_position, dest_size)**: Defines where and at what size to display the selected source content on screen. This enables positioning and scaling independently of the source.

**Example Use Case**: Display the top-left quarter of a 1920×1080 application buffer at 50% scale:
- `orig_size`: 1920×1080 (application buffer size)
- `src_position`: (0, 0), `src_size`: 960×540 (crop to top-left quarter)
- `dest_position`: (100, 100), `dest_size`: 480×270 (display at 50% scale, positioned at screen coordinates 100,100)

## Data Types

### Surface Object

```typescript
{
  id: number,              // Unique surface identifier
  orig_size: {
    width: number,         // Original application buffer width in pixels
    height: number         // Original application buffer height in pixels
  },
  src_position: {
    x: number,             // Source rectangle X coordinate
    y: number              // Source rectangle Y coordinate
  },
  src_size: {
    width: number,         // Source rectangle width in pixels
    height: number         // Source rectangle height in pixels
  },
  dest_position: {
    x: number,             // Destination rectangle X coordinate on screen
    y: number              // Destination rectangle Y coordinate on screen
  },
  dest_size: {
    width: number,         // Destination rectangle width on screen
    height: number         // Destination rectangle height on screen
  },
  visibility: boolean,     // true = visible, false = hidden
  opacity: number,         // 0.0 (transparent) to 1.0 (opaque)
  orientation: string,     // "Normal" | "Rotate90" | "Rotate180" | "Rotate270"
  z_order: number          // Stacking order (higher = on top)
}
```

### Orientation Values

| Value | Degrees | Description |
|-------|---------|-------------|
| `"Normal"` | 0° | No rotation |
| `"Rotate90"` | 90° | Rotated 90° clockwise |
| `"Rotate180"` | 180° | Rotated 180° |
| `"Rotate270"` | 270° | Rotated 270° clockwise (90° counter-clockwise) |

## Examples

### Complete Python Client Example

```python
#!/usr/bin/env python3
import socket
import json

class IVIController:
    def __init__(self, socket_path='/tmp/weston-ivi-controller.sock'):
        self.socket_path = socket_path
        self.sock = None
        self.request_id = 0
    
    def connect(self):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(self.socket_path)
    
    def disconnect(self):
        if self.sock:
            self.sock.close()
            self.sock = None
    
    def _send_request(self, method, params):
        self.request_id += 1
        request = {
            "id": self.request_id,
            "method": method,
            "params": params
        }
        self.sock.sendall(json.dumps(request).encode() + b'\n')
        response = json.loads(self.sock.recv(4096).decode())
        
        if 'error' in response:
            raise Exception(f"RPC Error {response['error']['code']}: {response['error']['message']}")
        
        return response.get('result')
    
    def list_surfaces(self):
        return self._send_request('list_surfaces', {})
    
    def get_surface(self, surface_id):
        return self._send_request('get_surface', {'id': surface_id})

    def set_surface_source_rectangle(self, surface_id, x, y, width, height):
        return self._send_request('set_surface_source_rectangle',
            {'id': surface_id, 'x': x, 'y': y, 'width': width, 'height': height})

    def set_surface_destination_rectangle(self, surface_id, x, y, width, height):
        return self._send_request('set_surface_destination_rectangle',
            {'id': surface_id, 'x': x, 'y': y, 'width': width, 'height': height})

    def set_surface_visibility(self, surface_id, visible):
        return self._send_request('set_surface_visibility', {'id': surface_id, 'visible': visible})

    def set_surface_opacity(self, surface_id, opacity):
        return self._send_request('set_surface_opacity', {'id': surface_id, 'opacity': opacity})

    def set_surface_z_order(self, surface_id, z_order):
        return self._send_request('set_surface_z_order', {'id': surface_id, 'z_order': z_order})

    def set_surface_focus(self, surface_id):
        return self._send_request('set_surface_focus', {'id': surface_id})
    
    def commit(self):
        return self._send_request('commit', {})

# Usage example
if __name__ == '__main__':
    controller = IVIController()
    controller.connect()
    
    try:
        # List all surfaces
        surfaces = controller.list_surfaces()
        print(f"Found {len(surfaces['surfaces'])} surfaces")
        
        # Get first surface
        if surfaces['surfaces']:
            surface_id = surfaces['surfaces'][0]['id']
            
            # Example 1: Atomic updates (recommended for multiple changes)
            print("\n=== Atomic Update Example ===")
            # Queue multiple changes
            controller.set_surface_destination_rectangle(surface_id, 100, 100, 800, 600)
            controller.set_surface_opacity(surface_id, 0.8)
            # Commit all changes atomically
            controller.commit()
            print(f"Atomically updated surface {surface_id}: destination rectangle and opacity")

            # Example 2: Single operation with auto-commit
            print("\n=== Auto-commit Example ===")
            result = controller._send_request('set_surface_destination_rectangle', {
                'id': surface_id,
                'x': 200,
                'y': 300,
                'width': 1024,
                'height': 768,
                'auto_commit': True
            })
            print(f"Moved surface {surface_id} with auto-commit: {result}")

            # Example 3: Complex atomic update
            print("\n=== Complex Atomic Update ===")
            controller.set_surface_z_order(surface_id, 10)
            controller.set_surface_focus(surface_id)
            controller.commit()
            print(f"Atomically updated surface {surface_id}: z-order and focus")
    
    finally:
        controller.disconnect()
```

### Bash Script Example

```bash
#!/bin/bash

SOCKET="/tmp/weston-ivi-controller.sock"

# Function to send RPC request
send_rpc() {
    local method=$1
    local params=$2
    local id=$((RANDOM))
    
    echo "{\"id\":$id,\"method\":\"$method\",\"params\":$params}" | nc -U "$SOCKET"
}

# List all surfaces
echo "Listing surfaces..."
send_rpc "list_surfaces" "{}"

# Get specific surface
echo "Getting surface 1000..."
send_rpc "get_surface" '{"id":1000}'

# Move and resize surface
echo "Moving and resizing surface to (200, 300) with 1024x768..."
send_rpc "set_surface_destination_rectangle" '{"id":1000,"x":200,"y":300,"width":1024,"height":768}'

# Hide surface
echo "Hiding surface..."
send_rpc "set_surface_visibility" '{"id":1000,"visible":false}'

# Show surface with opacity
echo "Showing surface with 50% opacity..."
send_rpc "set_surface_visibility" '{"id":1000,"visible":true}'
send_rpc "set_surface_opacity" '{"id":1000,"opacity":0.5}'
```

### C Example

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#define SOCKET_PATH "/tmp/weston-ivi-controller.sock"
#define BUFFER_SIZE 4096

int send_rpc_request(int sock, const char *request) {
    char buffer[BUFFER_SIZE];
    
    // Send request
    if (send(sock, request, strlen(request), 0) < 0) {
        perror("send");
        return -1;
    }
    
    // Receive response
    ssize_t n = recv(sock, buffer, BUFFER_SIZE - 1, 0);
    if (n < 0) {
        perror("recv");
        return -1;
    }
    
    buffer[n] = '\0';
    printf("Response: %s\n", buffer);
    
    return 0;
}

int main() {
    int sock;
    struct sockaddr_un addr;
    
    // Create socket
    sock = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sock < 0) {
        perror("socket");
        return 1;
    }
    
    // Connect to controller
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, SOCKET_PATH, sizeof(addr.sun_path) - 1);
    
    if (connect(sock, (struct sockaddr*)&addr, sizeof(addr)) < 0) {
        perror("connect");
        close(sock);
        return 1;
    }
    
    // List surfaces
    send_rpc_request(sock, "{\"id\":1,\"method\":\"list_surfaces\",\"params\":{}}\n");
    
    // Set destination rectangle
    send_rpc_request(sock, "{\"id\":2,\"method\":\"set_surface_destination_rectangle\",\"params\":{\"id\":1000,\"x\":100,\"y\":200,\"width\":800,\"height\":600}}\n");
    
    // Close connection
    close(sock);
    
    return 0;
}
```

## Best Practices

### Connection Management

- **Reuse connections**: Keep the socket open for multiple requests rather than reconnecting for each operation
- **Handle disconnections**: Implement reconnection logic for long-running clients
- **Close cleanly**: Always close the socket when done

### Error Handling

- **Check for errors**: Always check the response for an `error` field
- **Log errors**: Log error codes and messages for debugging
- **Retry on transient errors**: Some errors (like internal errors) may be transient

### Request IDs

- **Use unique IDs**: Ensure each request has a unique ID
- **Match responses**: Use the ID to match responses to requests in async scenarios

### Performance

- **Batch operations**: If possible, batch multiple operations to reduce round trips
- **Validate locally**: Validate parameters before sending to reduce error responses
- **Cache surface list**: Cache the surface list and only refresh when needed

### Thread Safety

- **One connection per thread**: Don't share socket connections across threads
- **Synchronize access**: If sharing a connection, synchronize access with locks

## Troubleshooting

### Connection Refused

**Problem:** Cannot connect to the socket

**Solutions:**
- Verify Weston is running with the IVI controller plugin loaded
- Check the socket path is correct
- Ensure you have permissions to access the socket
- Check `RUST_LOG` output for initialization errors

### Parse Errors

**Problem:** Receiving parse error (-32700)

**Solutions:**
- Ensure JSON is valid (use a JSON validator)
- Verify newline termination (`\n`)
- Check for proper UTF-8 encoding

### Surface Not Found

**Problem:** Receiving surface not found error (-32000)

**Solutions:**
- Use `list_surfaces` to get valid surface IDs
- Verify the surface still exists (it may have been destroyed)
- Check for typos in the surface ID

### Invalid Parameters

**Problem:** Receiving invalid params error (-32602)

**Solutions:**
- Check parameter types match the specification
- Verify values are within valid ranges
- Ensure all required parameters are present

## Version History

- **v0.1.0** - Initial release
  - Basic surface control operations
  - UNIX domain socket transport
  - JSON-RPC 2.0 protocol

## See Also

- [README.md](../README.md) - Main project documentation
- [Weston IVI Shell Documentation](https://wayland.pages.freedesktop.org/weston/toc/ivi-shell.html)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
