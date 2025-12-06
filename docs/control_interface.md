# IVI Controller RPC Protocol

This document describes the JSON-RPC 2.0 protocol used to communicate with the Weston IVI Controller module.

## Table of Contents

- [Overview](#overview)
- [Connection](#connection)
- [Message Format](#message-format)
- [Error Codes](#error-codes)
- [RPC Methods](#rpc-methods)
  - [list_surfaces](#list_surfaces)
  - [get_surface](#get_surface)
  - [set_position](#set_position)
  - [set_size](#set_size)
  - [set_visibility](#set_visibility)
  - [set_opacity](#set_opacity)
  - [set_orientation](#set_orientation)
  - [set_z_order](#set_z_order)
  - [set_focus](#set_focus)
  - [commit](#commit)
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

By default, surface modification methods (`set_position`, `set_size`, `set_visibility`, `set_opacity`, `set_orientation`, `set_z_order`, `set_focus`) **queue changes without immediately committing** them to the compositor. This allows you to:

1. **Batch multiple operations** on one or more surfaces
2. **Apply changes atomically** using the `commit` method
3. **Prevent visual artifacts** like tearing or intermediate states

**Example workflow:**
```
1. set_position (queued, not visible)
2. set_size (queued, not visible)
3. set_opacity (queued, not visible)
4. commit → All changes applied atomically
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
        "position": { "x": 0, "y": 0 },
        "size": { "width": 1920, "height": 1080 },
        "visibility": true,
        "opacity": 1.0,
        "orientation": "Normal",
        "z_order": 0
      },
      {
        "id": 1001,
        "position": { "x": 100, "y": 100 },
        "size": { "width": 800, "height": 600 },
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
  - `position` (object): Position with `x` and `y` coordinates
  - `size` (object): Size with `width` and `height`
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

### set_position

Update the position of an IVI surface.

**Request:**
```json
{
  "id": 3,
  "method": "set_position",
  "params": {
    "id": 1000,
    "x": 100,
    "y": 200
  }
}
```

**Response:**
```json
{
  "id": 3,
  "result": {
    "success": true
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID
- `x` (number, required): X coordinate (must be within display bounds)
- `y` (number, required): Y coordinate (must be within display bounds)
- `auto_commit` (boolean, optional): If `true`, commits changes immediately. Default: `false`

**Returns:**
- `success` (boolean): Always `true` on success
- `committed` (boolean): Indicates whether changes were committed

**Errors:**
- `-32000`: Surface not found
- `-32602`: Invalid parameters (coordinates out of bounds)

**Validation:**
- Position coordinates must be within valid display bounds
- Negative coordinates may be rejected depending on compositor configuration

**Behavior:**
- By default (`auto_commit=false`), changes are queued and require a `commit` call
- With `auto_commit=true`, changes are applied immediately

---

### set_size

Update the size of an IVI surface.

**Request:**
```json
{
  "id": 4,
  "method": "set_size",
  "params": {
    "id": 1000,
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
    "success": true
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID
- `width` (number, required): Width in pixels (must be positive)
- `height` (number, required): Height in pixels (must be positive)

**Returns:**
- `success` (boolean): Always `true` on success

**Errors:**
- `-32000`: Surface not found
- `-32602`: Invalid parameters (non-positive dimensions)

**Validation:**
- Width and height must be positive non-zero values

---

### set_visibility

Show or hide an IVI surface.

**Request:**
```json
{
  "id": 5,
  "method": "set_visibility",
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

### set_opacity

Adjust the opacity of an IVI surface.

**Request:**
```json
{
  "id": 6,
  "method": "set_opacity",
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

### set_orientation

Rotate an IVI surface.

**Request:**
```json
{
  "id": 7,
  "method": "set_orientation",
  "params": {
    "id": 1000,
    "orientation": "Rotate90"
  }
}
```

**Response:**
```json
{
  "id": 7,
  "result": {
    "success": true
  }
}
```

**Parameters:**
- `id` (number, required): Surface ID
- `orientation` (string, required): One of:
  - `"Normal"` - 0 degrees
  - `"Rotate90"` - 90 degrees clockwise
  - `"Rotate180"` - 180 degrees
  - `"Rotate270"` - 270 degrees clockwise

**Returns:**
- `success` (boolean): Always `true` on success

**Errors:**
- `-32000`: Surface not found
- `-32602`: Invalid parameters (invalid orientation value)

**Validation:**
- Only the four specified orientation values are accepted

---

### set_z_order

Change the stacking order (z-order) of an IVI surface.

**Request:**
```json
{
  "id": 8,
  "method": "set_z_order",
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

### set_focus

Route keyboard and pointer input focus to an IVI surface.

**Request:**
```json
{
  "id": 9,
  "method": "set_focus",
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
This method is essential for atomic updates. For example, to move and resize a window without showing intermediate states:

```json
// Step 1: Queue position change
{"id": 1, "method": "set_position", "params": {"id": 1000, "x": 100, "y": 200}}

// Step 2: Queue size change
{"id": 2, "method": "set_size", "params": {"id": 1000, "width": 800, "height": 600}}

// Step 3: Queue opacity change
{"id": 3, "method": "set_opacity", "params": {"id": 1000, "opacity": 0.8}}

// Step 4: Commit all changes atomically
{"id": 4, "method": "commit", "params": {}}
```

All three changes (position, size, opacity) will be applied simultaneously, preventing any visual artifacts.

---

## Data Types

### Surface Object

```typescript
{
  id: number,              // Unique surface identifier
  position: {
    x: number,             // X coordinate in pixels
    y: number              // Y coordinate in pixels
  },
  size: {
    width: number,         // Width in pixels
    height: number         // Height in pixels
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
    
    def set_position(self, surface_id, x, y):
        return self._send_request('set_position', {'id': surface_id, 'x': x, 'y': y})
    
    def set_size(self, surface_id, width, height):
        return self._send_request('set_size', {'id': surface_id, 'width': width, 'height': height})
    
    def set_visibility(self, surface_id, visible):
        return self._send_request('set_visibility', {'id': surface_id, 'visible': visible})
    
    def set_opacity(self, surface_id, opacity):
        return self._send_request('set_opacity', {'id': surface_id, 'opacity': opacity})
    
    def set_orientation(self, surface_id, orientation):
        return self._send_request('set_orientation', {'id': surface_id, 'orientation': orientation})
    
    def set_z_order(self, surface_id, z_order):
        return self._send_request('set_z_order', {'id': surface_id, 'z_order': z_order})
    
    def set_focus(self, surface_id):
        return self._send_request('set_focus', {'id': surface_id})
    
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
            controller.set_position(surface_id, 100, 100)
            controller.set_size(surface_id, 800, 600)
            controller.set_opacity(surface_id, 0.8)
            # Commit all changes atomically
            controller.commit()
            print(f"Atomically updated surface {surface_id}: position, size, and opacity")
            
            # Example 2: Single operation with auto-commit
            print("\n=== Auto-commit Example ===")
            result = controller._send_request('set_position', {
                'id': surface_id, 
                'x': 200, 
                'y': 300,
                'auto_commit': True
            })
            print(f"Moved surface {surface_id} with auto-commit: {result}")
            
            # Example 3: Complex atomic update
            print("\n=== Complex Atomic Update ===")
            controller.set_orientation(surface_id, "Rotate90")
            controller.set_z_order(surface_id, 10)
            controller.set_focus(surface_id)
            controller.commit()
            print(f"Atomically updated surface {surface_id}: rotation, z-order, and focus")
    
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

# Move surface
echo "Moving surface to (200, 300)..."
send_rpc "set_position" '{"id":1000,"x":200,"y":300}'

# Hide surface
echo "Hiding surface..."
send_rpc "set_visibility" '{"id":1000,"visible":false}'

# Show surface with opacity
echo "Showing surface with 50% opacity..."
send_rpc "set_visibility" '{"id":1000,"visible":true}'
send_rpc "set_opacity" '{"id":1000,"opacity":0.5}'
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
    
    // Set position
    send_rpc_request(sock, "{\"id\":2,\"method\":\"set_position\",\"params\":{\"id\":1000,\"x\":100,\"y\":200}}\n");
    
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
