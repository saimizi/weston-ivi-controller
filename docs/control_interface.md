# IVI Controller RPC Protocol

This document describes the JSON-RPC 2.0 protocol used to communicate with the Weston IVI Controller module.

## Table of Contents

- [Overview](#overview)
- [Connection](#connection)
- [Message Format](#message-format)
- [Error Codes](#error-codes)
- [RPC Methods](#rpc-methods)
  - Surface methods
    - [list_surfaces](#list_surfaces)
    - [get_surface](#get_surface)
    - [set_surface_source_rectangle](#set_surface_source_rectangle)
    - [set_surface_destination_rectangle](#set_surface_destination_rectangle)
    - [set_surface_visibility](#set_surface_visibility)
    - [set_surface_opacity](#set_surface_opacity)
    - [set_surface_z_order](#set_surface_z_order)
    - [set_surface_focus](#set_surface_focus)
    - [commit](#commit)
  - Layer methods
    - [list_layers](#list_layers)
    - [get_layer](#get_layer)
    - [create_layer](#create_layer)
    - [destroy_layer](#destroy_layer)
    - [set_layer_source_rectangle](#set_layer_source_rectangle)
    - [set_layer_destination_rectangle](#set_layer_destination_rectangle)
    - [set_layer_visibility](#set_layer_visibility)
    - [set_layer_opacity](#set_layer_opacity)
    - [set_layer_surfaces](#set_layer_surfaces)
    - [add_surface_to_layer](#add_surface_to_layer)
    - [remove_surface_from_layer](#remove_surface_from_layer)
    - [get_layer_surfaces](#get_layer_surfaces)
  - Screen methods
    - [list_screens](#list_screens)
    - [get_screen](#get_screen)
    - [get_screen_layers](#get_screen_layers)
    - [get_layer_screens](#get_layer_screens)
    - [add_layers_to_screen](#add_layers_to_screen)
    - [remove_layer_from_screen](#remove_layer_from_screen)
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
- **Length-prefixed framing**: Each message is preceded by a 4-byte big-endian unsigned integer giving the byte length of the JSON body
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
3. Send JSON-RPC requests using length-prefixed framing
4. Receive JSON-RPC responses using length-prefixed framing
5. Close the connection when done

### Framing

Each message (request, response, or notification) is framed as:

```
[ 4 bytes: big-endian uint32 length ][ N bytes: JSON body ]
```

### Example Connection (Python)

```python
import socket
import struct
import json

sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect('/tmp/weston-ivi-controller.sock')

# Send request with length-prefixed framing
request = {"id": 1, "method": "list_surfaces", "params": {}}
body = json.dumps(request).encode()
sock.sendall(struct.pack('>I', len(body)) + body)

# Receive length-prefixed response
header = sock.recv(4)
length = struct.unpack('>I', header)[0]
response = json.loads(sock.recv(length).decode())

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
| -32000 | Not found | The specified surface or layer ID does not exist |

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
        "src_rect": { "x": 0, "y": 0, "width": 1920, "height": 1080 },
        "dest_rect": { "x": 0, "y": 0, "width": 1920, "height": 1080 },
        "visibility": true,
        "opacity": 1.0,
        "orientation": "Normal",
        "z_order": 0
      },
      {
        "id": 1001,
        "orig_size": { "width": 1600, "height": 1200 },
        "src_rect": { "x": 0, "y": 0, "width": 1600, "height": 1200 },
        "dest_rect": { "x": 100, "y": 100, "width": 800, "height": 600 },
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
  - `src_rect` (object): Source rectangle with `x`, `y`, `width`, `height`
  - `dest_rect` (object): Destination rectangle with `x`, `y`, `width`, `height`
  - `visibility` (boolean): Whether the surface is visible
  - `opacity` (number): Opacity value (0.0 - 1.0)
  - `orientation` (string): Orientation ("Normal", "Rotate90", "Rotate180", "Rotate270", etc.)
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
    "orig_size": { "width": 1920, "height": 1080 },
    "src_rect": { "x": 0, "y": 0, "width": 1920, "height": 1080 },
    "dest_rect": { "x": 0, "y": 0, "width": 1920, "height": 1080 },
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
- `committed` (boolean): Reflects whether changes were committed (`true` if `auto_commit` was set)

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
- `committed` (boolean): Reflects whether changes were committed

**Errors:**
- `-32000`: Surface not found
- `-32602`: Invalid parameters (coordinates out of bounds or non-positive dimensions)

**Validation:**
- Width and height must be positive non-zero values

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
    "success": true,
    "committed": false
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID
- `visible` (boolean, required): `true` to show, `false` to hide
- `auto_commit` (boolean, optional): If `true`, commits changes immediately. Default: `false`

**Returns:**
- `success` (boolean): Always `true` on success
- `committed` (boolean): Reflects whether changes were committed

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
    "success": true,
    "committed": false
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID
- `opacity` (number, required): Opacity value (0.0 = fully transparent, 1.0 = fully opaque)
- `auto_commit` (boolean, optional): If `true`, commits changes immediately. Default: `false`

**Returns:**
- `success` (boolean): Always `true` on success
- `committed` (boolean): Reflects whether changes were committed

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
    "success": true,
    "committed": false
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID
- `z_order` (number, required): Z-order value (higher values appear on top)
- `auto_commit` (boolean, optional): If `true`, commits changes immediately. Default: `false`

**Returns:**
- `success` (boolean): Always `true` on success
- `committed` (boolean): Reflects whether changes were committed

**Errors:**
- `-32000`: Surface not found
- `-32602`: Invalid parameters (z-order out of valid range)

**Validation:**
- Z-order must be in the range [0, 1000]
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
    "success": true,
    "committed": false
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID to receive focus
- `auto_commit` (boolean, optional): If `true`, commits changes immediately. Default: `false`

**Returns:**
- `success` (boolean): Always `true` on success
- `committed` (boolean): Reflects whether changes were committed

**Errors:**
- `-32000`: Surface not found

**Behavior:**
- Sets both keyboard and pointer focus to the specified surface
- Removes focus from the previously focused surface
- Focus change notifications are sent to both old and new focused surfaces
- By default, changes are queued (not committed). Use `commit` method or `auto_commit` parameter.

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
      {
        "id": 5000,
        "src_rect": { "x": 0, "y": 0, "width": 1920, "height": 1080 },
        "dest_rect": { "x": 0, "y": 0, "width": 1920, "height": 1080 },
        "visibility": true,
        "opacity": 1.0,
        "orientation": "Normal"
      },
      {
        "id": 5001,
        "src_rect": { "x": 0, "y": 0, "width": 1920, "height": 1080 },
        "dest_rect": { "x": 0, "y": 0, "width": 1920, "height": 1080 },
        "visibility": false,
        "opacity": 0.5,
        "orientation": "Normal"
      }
    ]
  }
}
```

**Parameters:** None

**Returns:**
- `layers` (array): Array of layer objects, each containing:
  - `id` (number): Layer ID
  - `src_rect` (object): Source rectangle with `x`, `y`, `width`, `height`
  - `dest_rect` (object): Destination rectangle with `x`, `y`, `width`, `height`
  - `visibility` (boolean): Whether the layer is visible
  - `opacity` (number): Opacity value (0.0 - 1.0)
  - `orientation` (string): Layer orientation

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
  "result": {
    "id": 5000,
    "src_rect": { "x": 0, "y": 0, "width": 1920, "height": 1080 },
    "dest_rect": { "x": 0, "y": 0, "width": 1920, "height": 1080 },
    "visibility": true,
    "opacity": 1.0,
    "orientation": "Normal"
  }
}
```

**Parameters:**
- `id` (number, required): Layer ID

Errors: `-32602` for invalid or missing `id`

---

### create_layer

Create a new IVI layer with the specified ID and dimensions.

Request:
```json
{
  "id": 110,
  "method": "create_layer",
  "params": { "id": 5000, "width": 1920, "height": 1080 }
}
```

Response:
```json
{
  "id": 110,
  "result": { "id": 5000, "committed": false }
}
```

**Parameters:**
- `id` (number, required): Layer ID to create
- `width` (number, required): Layer width in pixels (must be positive)
- `height` (number, required): Layer height in pixels (must be positive)
- `auto_commit` (boolean, optional): If `true`, commits changes immediately. Default: `false`

**Returns:**
- `id` (number): ID of the created layer
- `committed` (boolean): Reflects whether changes were committed

Errors: `-32602` for invalid dimensions, `-32603` if creation fails

---

### destroy_layer

Destroy an existing IVI layer.

Request:
```json
{
  "id": 111,
  "method": "destroy_layer",
  "params": { "id": 5000 }
}
```

Response:
```json
{
  "id": 111,
  "result": { "success": true, "committed": false }
}
```

**Parameters:**
- `id` (number, required): Layer ID to destroy
- `auto_commit` (boolean, optional): If `true`, commits changes immediately. Default: `false`

**Returns:**
- `success` (boolean): Always `true` on success
- `committed` (boolean): Reflects whether changes were committed

Errors: `-32000` if layer not found

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

**Parameters:**
- `id` (number, required): Layer ID
- `x`, `y` (number, required): Source position
- `width`, `height` (number, required): Source dimensions (must be positive)
- `auto_commit` (boolean, optional): If `true`, commits immediately. Default: `false`

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

Optional param: `auto_commit` (bool)

---

### set_layer_surfaces

Replace all surfaces on a layer with the specified set (in render order, first = bottommost).

Request:
```json
{
  "id": 120,
  "method": "set_layer_surfaces",
  "params": { "layer_id": 5000, "surface_ids": [1000, 1001, 1002] }
}
```

Response:
```json
{
  "id": 120,
  "result": { "layer_id": 5000, "surface_ids": [1000, 1001, 1002], "committed": false }
}
```

**Parameters:**
- `layer_id` (number, required): Layer ID
- `surface_ids` (array, required): Ordered array of surface IDs (first = bottommost, last = topmost)
- `auto_commit` (boolean, optional): Default: `false`

Errors: `-32603` if layer or any surface not found

---

### add_surface_to_layer

Add a surface to a layer as the topmost element.

Request:
```json
{
  "id": 121,
  "method": "add_surface_to_layer",
  "params": { "layer_id": 5000, "surface_id": 1002 }
}
```

Response:
```json
{
  "id": 121,
  "result": { "layer_id": 5000, "surface_id": 1002, "committed": false }
}
```

**Parameters:**
- `layer_id` (number, required): Layer ID
- `surface_id` (number, required): Surface ID to add
- `auto_commit` (boolean, optional): Default: `false`

---

### remove_surface_from_layer

Remove a surface from a layer.

Request:
```json
{
  "id": 122,
  "method": "remove_surface_from_layer",
  "params": { "layer_id": 5000, "surface_id": 1001 }
}
```

Response:
```json
{
  "id": 122,
  "result": { "layer_id": 5000, "surface_id": 1001, "committed": false }
}
```

**Parameters:**
- `layer_id` (number, required): Layer ID
- `surface_id` (number, required): Surface ID to remove
- `auto_commit` (boolean, optional): Default: `false`

---

### get_layer_surfaces

Get the list of surfaces assigned to a layer, in render order (first = bottommost, last = topmost).

Request:
```json
{ "id": 123, "method": "get_layer_surfaces", "params": { "layer_id": 5000 } }
```

Response:
```json
{
  "id": 123,
  "result": { "surface_ids": [1000, 1001, 1002] }
}
```

**Parameters:**
- `layer_id` (number, required): Layer ID

Errors: `-32603` if layer not found

---

### list_screens

List all available screens (compositor outputs).

Request:
```json
{ "id": 200, "method": "list_screens", "params": {} }
```

Response:
```json
{
  "id": 200,
  "result": {
    "screens": [
      {
        "name": "HDMI-A-1",
        "width": 1920,
        "height": 1080,
        "x": 0,
        "y": 0,
        "transform": "Normal",
        "enabled": true,
        "scale": 1
      }
    ]
  }
}
```

**Parameters:** None

---

### get_screen

Get properties of a specific screen by name.

Request:
```json
{ "id": 201, "method": "get_screen", "params": { "name": "HDMI-A-1" } }
```

Response:
```json
{
  "id": 201,
  "result": {
    "name": "HDMI-A-1",
    "width": 1920,
    "height": 1080,
    "x": 0,
    "y": 0,
    "transform": "Normal",
    "enabled": true,
    "scale": 1
  }
}
```

**Parameters:**
- `name` (string, required): Screen name

Errors: `-32603` if screen not found

---

### get_screen_layers

Get the list of layer IDs currently assigned to a screen.

Request:
```json
{ "id": 202, "method": "get_screen_layers", "params": { "screen_name": "HDMI-A-1" } }
```

Response:
```json
{
  "id": 202,
  "result": { "layer_ids": [5000, 5001] }
}
```

**Parameters:**
- `screen_name` (string, required): Screen name

Errors: `-32603` if screen not found

---

### get_layer_screens

Get the list of screen names that a layer is assigned to.

Request:
```json
{ "id": 203, "method": "get_layer_screens", "params": { "layer_id": 5000 } }
```

Response:
```json
{
  "id": 203,
  "result": { "screen_names": ["HDMI-A-1"] }
}
```

**Parameters:**
- `layer_id` (number, required): Layer ID

Errors: `-32000` if layer not found

---

### add_layers_to_screen

Set the render order of layers on a screen. This replaces the current layer assignment.

Request:
```json
{
  "id": 204,
  "method": "add_layers_to_screen",
  "params": { "screen_name": "HDMI-A-1", "layer_ids": [5000, 5001] }
}
```

Response:
```json
{
  "id": 204,
  "result": { "screen_name": "HDMI-A-1", "layer_ids": [5000, 5001], "committed": false }
}
```

**Parameters:**
- `screen_name` (string, required): Screen name
- `layer_ids` (array, required): Ordered list of layer IDs
- `auto_commit` (boolean, optional): Default: `false`

Errors: `-32603` if screen or any layer not found

---

### remove_layer_from_screen

Remove a specific layer from a screen.

Request:
```json
{
  "id": 205,
  "method": "remove_layer_from_screen",
  "params": { "screen_name": "HDMI-A-1", "layer_id": 5001 }
}
```

Response:
```json
{
  "id": 205,
  "result": { "screen_name": "HDMI-A-1", "layer_id": 5001, "committed": false }
}
```

**Parameters:**
- `screen_name` (string, required): Screen name
- `layer_id` (number, required): Layer ID to remove
- `auto_commit` (boolean, optional): Default: `false`

Errors: `-32000` if layer not found, `-32603` if screen not found

---

## Event Notifications

Clients may subscribe to real-time events. Subscriptions are per-client and selective by event type. Each client has a best-effort FIFO buffer (default 100); oldest notifications are dropped when full.

- Delivery: Length-prefixed JSON-RPC notifications (no `id`) sent on the subscribed connection
- Filtering: By event type (no per-surface filtering)
- Multiple clients: Supported

Supported event types:
- `SurfaceCreated`, `SurfaceDestroyed`, `SourceGeometryChanged`, `DestinationGeometryChanged`, `VisibilityChanged`, `OpacityChanged`, `OrientationChanged`, `ZOrderChanged`, `FocusChanged`
- `LayerCreated`, `LayerDestroyed`, `LayerVisibilityChanged`, `LayerOpacityChanged`

### subscribe

Request:
```json
{
  "id": 300,
  "method": "subscribe",
  "params": { "event_types": ["SurfaceCreated", "SourceGeometryChanged", "FocusChanged"] }
}
```

Response:
```json
{
  "id": 300,
  "result": { "success": true, "subscribed": ["SurfaceCreated", "SourceGeometryChanged", "FocusChanged"] }
}
```

### unsubscribe

Request:
```json
{
  "id": 301,
  "method": "unsubscribe",
  "params": { "event_types": ["SourceGeometryChanged"] }
}
```

Response:
```json
{
  "id": 301,
  "result": { "success": true, "unsubscribed": ["SourceGeometryChanged"] }
}
```

### list_subscriptions

Request:
```json
{ "id": 302, "method": "list_subscriptions", "params": {} }
```

Response:
```json
{ "id": 302, "result": { "subscriptions": ["SurfaceCreated", "FocusChanged"] } }
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

- SurfaceDestroyed
```json
{ "method": "notification", "params": { "event_type": "SurfaceDestroyed", "surface_id": 1000 } }
```

- SourceGeometryChanged
```json
{
  "method": "notification",
  "params": {
    "event_type": "SourceGeometryChanged",
    "surface_id": 1000,
    "old_rect": {"x": 0, "y": 0, "width": 1920, "height": 1080},
    "new_rect": {"x": 0, "y": 0, "width": 960, "height": 540}
  }
}
```

- DestinationGeometryChanged
```json
{
  "method": "notification",
  "params": {
    "event_type": "DestinationGeometryChanged",
    "surface_id": 1000,
    "old_rect": {"x": 0, "y": 0, "width": 1920, "height": 1080},
    "new_rect": {"x": 100, "y": 100, "width": 1280, "height": 720}
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

- **Source Rectangle (src_rect)**: Defines which portion of the application buffer to display. This enables cropping - you can display only part of the buffer. For example, you might show only the top-left quarter of a 1920×1080 buffer.

- **Destination Rectangle (dest_rect)**: Defines where and at what size to display the selected source content on screen. This enables positioning and scaling independently of the source.

**Example Use Case**: Display the top-left quarter of a 1920×1080 application buffer at 50% scale:
- `orig_size`: 1920×1080 (application buffer size)
- `src_rect`: `{"x": 0, "y": 0, "width": 960, "height": 540}` (crop to top-left quarter)
- `dest_rect`: `{"x": 100, "y": 100, "width": 480, "height": 270}` (display at 50% scale, at screen coordinates 100,100)

## Data Types

### Surface Object

```typescript
{
  id: number,              // Unique surface identifier
  orig_size: {
    width: number,         // Original application buffer width in pixels
    height: number         // Original application buffer height in pixels
  },
  src_rect: {
    x: number,             // Source rectangle X coordinate
    y: number,             // Source rectangle Y coordinate
    width: number,         // Source rectangle width in pixels
    height: number         // Source rectangle height in pixels
  },
  dest_rect: {
    x: number,             // Destination rectangle X coordinate on screen
    y: number,             // Destination rectangle Y coordinate on screen
    width: number,         // Destination rectangle width on screen
    height: number         // Destination rectangle height on screen
  },
  visibility: boolean,     // true = visible, false = hidden
  opacity: number,         // 0.0 (transparent) to 1.0 (opaque)
  orientation: string,     // See Orientation Values below
  z_order: number          // Stacking order (higher = on top)
}
```

### Layer Object

```typescript
{
  id: number,              // Unique layer identifier
  src_rect: {
    x: number,
    y: number,
    width: number,
    height: number
  },
  dest_rect: {
    x: number,
    y: number,
    width: number,
    height: number
  },
  visibility: boolean,
  opacity: number,
  orientation: string
}
```

### Screen Object

```typescript
{
  name: string,            // Screen name (e.g. "HDMI-A-1")
  width: number,           // Screen width in pixels
  height: number,          // Screen height in pixels
  x: number,               // Global X coordinate
  y: number,               // Global Y coordinate
  transform: string,       // Screen transform
  enabled: boolean,        // Whether the screen is active
  scale: number            // Scale factor
}
```

### Orientation Values

| Value | Degrees | Description |
|-------|---------|-------------|
| `"Normal"` | 0° | No rotation |
| `"Rotate90"` | 90° | Rotated 90° clockwise |
| `"Rotate180"` | 180° | Rotated 180° |
| `"Rotate270"` | 270° | Rotated 270° clockwise |
| `"Flipped"` | — | Horizontally flipped |
| `"Flipped90"` | — | Flipped + 90° rotation |
| `"Flipped180"` | — | Flipped + 180° rotation |
| `"Flipped270"` | — | Flipped + 270° rotation |

## Examples

### Complete Python Client Example

```python
#!/usr/bin/env python3
import socket
import struct
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

    def _recv_exact(self, n):
        """Read exactly n bytes from the socket."""
        data = b''
        while len(data) < n:
            chunk = self.sock.recv(n - len(data))
            if not chunk:
                raise Exception("Connection closed")
            data += chunk
        return data

    def _send_request(self, method, params):
        self.request_id += 1
        request = {
            "id": self.request_id,
            "method": method,
            "params": params
        }
        # Length-prefixed framing: 4-byte big-endian length + JSON body
        body = json.dumps(request).encode()
        self.sock.sendall(struct.pack('>I', len(body)) + body)

        # Read length-prefixed response
        header = self._recv_exact(4)
        length = struct.unpack('>I', header)[0]
        response = json.loads(self._recv_exact(length).decode())

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
            surface = surfaces['surfaces'][0]

            # Access rect fields
            print(f"Source rect: {surface['src_rect']}")
            print(f"Dest rect: {surface['dest_rect']}")

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

### Python Notification Listener Example

```python
#!/usr/bin/env python3
import socket
import struct
import json
import threading

class IVINotificationListener:
    """
    Opens its own dedicated connection for receiving notifications so that
    unsolicited messages do not interfere with RPC responses.
    """
    def __init__(self, socket_path='/tmp/weston-ivi-controller.sock'):
        self.socket_path = socket_path
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(socket_path)
        self.request_id = 0
        self.callbacks = {}      # event_type -> list of callables
        self.catch_all = []
        self._thread = None
        self._stop = False

    def _recv_exact(self, n):
        data = b''
        while len(data) < n:
            chunk = self.sock.recv(n - len(data))
            if not chunk:
                raise Exception("Connection closed")
            data += chunk
        return data

    def _send_rpc(self, method, params):
        self.request_id += 1
        body = json.dumps({"id": self.request_id, "method": method, "params": params}).encode()
        self.sock.sendall(struct.pack('>I', len(body)) + body)
        header = self._recv_exact(4)
        length = struct.unpack('>I', header)[0]
        return json.loads(self._recv_exact(length).decode())

    def on(self, event_type, callback):
        """Register a callback for a specific event type."""
        self.callbacks.setdefault(event_type, []).append(callback)

    def on_all(self, callback):
        """Register a catch-all callback for every event type."""
        self.catch_all.append(callback)

    def start(self, event_types):
        """Subscribe and start background reader thread."""
        self._send_rpc('subscribe', {'event_types': event_types})
        self._stop = False
        self._thread = threading.Thread(target=self._reader, daemon=True)
        self._thread.start()

    def _reader(self):
        self.sock.settimeout(0.1)
        while not self._stop:
            try:
                header = self._recv_exact(4)
                length = struct.unpack('>I', header)[0]
                msg = json.loads(self._recv_exact(length).decode())
                if 'id' in msg:
                    continue  # stray RPC response, skip
                params = msg.get('params', {})
                event_type = params.get('event_type')
                for cb in self.callbacks.get(event_type, []):
                    cb(params)
                for cb in self.catch_all:
                    cb(params)
            except socket.timeout:
                continue
            except Exception:
                break

    def stop(self):
        self._stop = True
        if self._thread:
            self._thread.join()
        self.sock.close()

# Usage
listener = IVINotificationListener()

listener.on('SurfaceCreated', lambda p: print(f"Surface created: {p['surface_id']}"))
listener.on('VisibilityChanged', lambda p:
    print(f"Surface {p['surface_id']} visibility: {p['old_visibility']} -> {p['new_visibility']}"))
listener.on_all(lambda p: print(f"Event: {p['event_type']}"))

listener.start(['SurfaceCreated', 'SurfaceDestroyed', 'VisibilityChanged'])

# ... listener fires callbacks in background thread ...

listener.stop()
```

### Bash Script Example

> **Note:** The protocol uses 4-byte big-endian length-prefixed framing, which plain shell tools like `nc` do not support natively. Use the `ivi-cli` tool for shell scripting, or write a small Python/C helper.

```bash
#!/bin/bash

# Use ivi-cli for shell scripting
ivi-cli list-surfaces
ivi-cli get-surface --id 1000
ivi-cli set-surface-destination-rectangle --id 1000 --x 200 --y 300 --width 1024 --height 768
ivi-cli set-surface-visibility --id 1000 --visible false
ivi-cli commit
```

### C Example (using ivi_client.h)

The recommended way to use the protocol from C is via the `ivi_client.h` library, which handles framing automatically:

```c
#include <stdio.h>
#include "ivi_client.h"

int main(void) {
    char err[256];

    // --- Synchronous RPC ---
    IviClient *client = ivi_client_connect(NULL, err, sizeof(err));
    if (!client) { fprintf(stderr, "connect: %s\n", err); return 1; }

    IviSurface *surfaces = NULL;
    size_t count = 0;
    if (ivi_list_surfaces(client, &surfaces, &count, err, sizeof(err)) == OK) {
        printf("Found %zu surfaces\n", count);
        ivi_free_surfaces(surfaces, count);
    }

    ivi_set_surface_destination_rectangle(client, 1000, 100, 200, 800, 600, err, sizeof(err));
    ivi_commit(client, err, sizeof(err));
    ivi_client_disconnect(client);

    // --- Event Notifications ---
    void on_surface_created(const IviNotification *n, void *ud) {
        printf("Surface created: %u\n", n->object_id);
    }
    void on_visibility(const IviNotification *n, void *ud) {
        printf("Surface %u visibility: %d -> %d\n", n->object_id,
               n->visibility.old_visibility, n->visibility.new_visibility);
    }

    NotificationListener *listener =
        ivi_notification_listener_new(NULL, err, sizeof(err));
    if (!listener) { fprintf(stderr, "listener: %s\n", err); return 1; }

    ivi_notification_listener_on(listener, SURFACE_CREATED,    on_surface_created, NULL);
    ivi_notification_listener_on(listener, VISIBILITY_CHANGED, on_visibility,      NULL);

    IviEventType types[] = { SURFACE_CREATED, VISIBILITY_CHANGED };
    ivi_notification_listener_start(listener, types, 2, err, sizeof(err));

    // ... callbacks fire in background thread ...

    ivi_notification_listener_stop(listener);
    ivi_notification_listener_free(listener);
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
