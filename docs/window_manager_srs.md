# IVI Window Manager — Software Requirement Specification

**Document Version:** 1.0
**Date:** 2026-03-29
**Status:** Draft

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Overall Description](#2-overall-description)
3. [Functional Requirements](#3-functional-requirements)
4. [Non-Functional Requirements](#4-non-functional-requirements)
5. [Interface Requirements](#5-interface-requirements)
6. [Data Model](#6-data-model)
7. [Traceability Matrix](#7-traceability-matrix)

---

## 1. Introduction

### 1.1 Purpose

This document defines the software requirements for the IVI Window Manager (`ivi-wm`), a new application that provides automated, policy-driven layout management for surfaces displayed on an IVI (In-Vehicle Infotainment) system built on the Weston compositor.

The window manager operates as a client of the existing Weston IVI Controller plugin (`libweston_ivi_controller.so`), consuming its JSON-RPC 2.0 API via the `ivi-client` library to orchestrate surface placement, layer organization, and screen composition across multiple displays.

### 1.2 Problem Statement

The IVI Controller provides low-level, imperative control over individual surfaces, layers, and screens. Without a window manager, every application or system component that wants to display content must independently calculate coordinates, negotiate z-ordering, and manage visibility. This leads to:

- **No layout policy enforcement** — applications can position themselves anywhere, overlap arbitrarily, or obscure important content.
- **No surface lifecycle automation** — when a new surface appears (via `SurfaceCreated` notification), nothing automatically assigns it to a layer, positions it, or makes it visible.
- **No priority management** — a navigation warning or rear-camera feed has no mechanism to preempt entertainment content.
- **No multi-screen coordination** — in systems with multiple displays (center console, instrument cluster, rear-seat), there is no central authority deciding which surfaces go where.

The window manager solves these problems by acting as the single authority for surface placement and visibility decisions.

### 1.3 Scope

The window manager shall be implemented as a new Rust crate (`ivi-wm`) within the existing `js-controller` workspace. It:

- Connects to the IVI controller via the `ivi-client` library
- Subscribes to surface and layer lifecycle events via `NotificationListener`
- Applies layout policies based on a declarative TOML configuration file
- Exposes its own JSON-RPC 2.0 control API for runtime layout changes by external orchestrators
- Manages layout independently across multiple physical screens

### 1.4 Definitions

| Term | Definition |
|------|-----------|
| **Surface** | A Wayland client's renderable buffer, identified by a numeric ID (`u32`). Corresponds to `IviSurface` in the client library. |
| **Layer** | A grouping construct that holds ordered surfaces. Layers have their own geometry, visibility, and opacity. Corresponds to `IviLayer`. |
| **Screen** | A physical display output, identified by name (e.g., `"HDMI-A-1"`). Corresponds to `IviScreen`. |
| **Zone** | A named rectangular region within a screen, defined in configuration, where surfaces may be placed. |
| **Role** | A classification assigned to a surface (e.g., `"navigation"`, `"media"`, `"camera"`, `"popup"`) that determines its layout policy. |
| **Layout Policy** | A rule set that determines how surfaces of a given role are positioned, sized, layered, and prioritized within a zone. |

### 1.5 References

| Document | Location |
|----------|----------|
| IVI Controller RPC Protocol | `docs/control_interface.md` |
| IVI Client Library Guide | `docs/client_library.md` |
| IVI Controller Configuration | `docs/configuration.md` |
| IVI Client API (Rust) | `ivi-client/src/client.rs` |
| IVI Client Data Types | `ivi-client/src/ffi.rs` |

---

## 2. Overall Description

### 2.1 System Architecture

```
+-------------------+      +-------------------+      +-------------------+
| External          |----->| IVI Window        |----->| IVI Controller    |
| Orchestrators     |      | Manager           |      | Plugin            |
| (HMI launcher,    | JSONR| (ivi-wm)          | JSONR| (JSON-RPC server) |
|  system services)  | PC   | THIS SYSTEM       | PC   |                   |
+-------------------+      +---+----------+----+      +--------+----------+
                               |          ^                    |
                          Control API   Notifications          v
                          (exposed)    (consumed)         Weston Compositor
                               |          |                    |
                               v          |                    v
                     +-------------------+|              IVI Shell
                     | Config File       ||                    |
                     | (TOML)            ||                    v
                     +-------------------+|              Wayland Clients
                                          |              (applications)
                              NotificationListener
```

The window manager occupies a critical middle position:
- **Downstream**: It is a client of the IVI controller, using `IviClient` for commands and `NotificationListener` for events.
- **Upstream**: It is a server to higher-level system components (HMI orchestrator, app launcher), exposing a JSON-RPC 2.0 API for runtime layout control.

### 2.2 Product Perspective

The window manager does **not** modify the Weston compositor, the IVI controller plugin, or the `ivi-client` library. It uses exclusively the existing public API surface:

- **Command channel**: `IviClient::new()` connecting to the controller socket (default `/tmp/weston-ivi-controller.sock`)
- **Event channel**: `NotificationListener::new()` on a separate socket connection, subscribing to events via the `subscribe` RPC method
- **Atomic commits**: All layout changes are batched using `auto_commit: false` and applied atomically via `commit()` to prevent visual tearing

### 2.3 User Classes

| User Class | Interaction |
|-----------|------------|
| **End User** | Indirect — sees the result of layout policies; does not interact with the WM directly |
| **System Integrator** | Configures layout policies, zone definitions, role mappings, and priority rules via TOML configuration |
| **HMI Orchestrator / App Launcher** | Sends commands to the WM control API to request layout transitions (e.g., "show navigation fullscreen", "split navigation + media") |

### 2.4 Operating Environment

- Linux-based IVI platform (Yocto-based, consistent with the existing `yocto/` directory)
- Weston compositor with IVI shell enabled (`shell=ivi-shell.so`)
- IVI controller plugin loaded (`modules=weston-ivi-controller.so`)
- One or more physical displays (screens)
- Rust runtime (no garbage collection — suitable for latency-sensitive display operations)

### 2.5 Constraints

- Must not require modifications to `libweston_ivi_controller.so` or `ivi-client`
- Must operate within the IVI compositor model: surfaces belong to layers, layers belong to screens
- Must handle auto-assigned surface IDs (surfaces arriving with IDs in the `0x10000000`+ range per the controller's auto-assignment configuration)
- Surfaces have `src_rect` (crop from buffer) and `dest_rect` (placement on screen) — the WM sets both for positioning and scaling
- The controller's `set_surface_focus()` routes keyboard and pointer input — the WM must manage this explicitly

---

## 3. Functional Requirements

### 3.1 Surface Lifecycle Management

#### FR-3.1.1 Surface Discovery and Registration

The WM shall subscribe to `SurfaceCreated`, `SurfaceDestroyed`, `SurfaceContentReady`, and `SurfaceContentSizeChanged` events via `NotificationListener`.

Upon receiving `SurfaceCreated`, the WM shall call `get_surface(id)` to obtain the surface's properties (`orig_size`, `src_rect`, `dest_rect`, `visibility`, `opacity`, `orientation`, `z_order`).

Layout decisions shall be deferred until `SurfaceContentReady` is received, indicating the surface has committed its first buffer and `orig_size` is valid.

#### FR-3.1.2 Surface Role Assignment

The WM shall support the following role assignment strategies, checked in order:

1. **Static mapping**: A specific surface ID maps to a role (e.g., surface `1000` = `"navigation"`)
2. **Range mapping**: A surface ID range maps to a role (e.g., `2000-2999` = `"media"`)
3. **Dynamic assignment**: An external orchestrator assigns a role at runtime via the WM control API (`assign_role` method)
4. **Default role**: Surfaces that match no mapping receive a configurable default role

For auto-assigned surfaces (IDs in the range starting at `id_start`, default `0x10000000`), dynamic assignment via the control API shall be the primary mechanism.

#### FR-3.1.3 Surface Placement on Creation

When a surface is registered, has content ready, and its role is determined, the WM shall:

1. Select the appropriate layer for the role (from the configured layer hierarchy)
2. Add the surface to the layer via `add_surface_to_layer(layer_id, surface_id, false)`
3. Calculate the destination rectangle based on the active layout policy for the role's zone
4. Call `set_surface_destination_rectangle(id, x, y, width, height, false)`
5. Set `set_surface_source_rectangle(id, x, y, width, height, false)` if scaling/cropping is needed
6. Call `set_surface_visibility(id, true, false)`
7. Call `set_surface_z_order(id, z_order, false)` within the layer
8. Call `commit()` to apply all changes atomically

#### FR-3.1.4 Surface Destruction Handling

On `SurfaceDestroyed`, the WM shall:

- Remove the surface from its internal tracking
- Recalculate layout for the affected zone (e.g., if a tiled surface is removed, remaining surfaces expand to fill the space)
- Update focus if the destroyed surface had focus (assign focus to the next appropriate surface)
- Call `commit()` to apply changes atomically

### 3.2 Layer Management

#### FR-3.2.1 Layer Architecture

The WM shall create and manage a configurable set of layers organized by purpose and z-order priority. The default hierarchy (from bottom to top):

| Layer Purpose | Default ID | Z-Order | Typical Content |
|--------------|-----------|---------|-----------------|
| Background | 100 | 0 | Wallpaper, system background |
| Main Application | 200 | 1 | Navigation, media, climate |
| Secondary Application | 300 | 2 | Split-screen secondary pane |
| Popup / Dialog | 400 | 3 | Notifications, confirmations |
| System Overlay | 500 | 4 | Status bar, system warnings |
| Priority | 600 | 5 | Rear camera, critical alerts |

Layer IDs, names, and the hierarchy shall be fully configurable via the TOML configuration file.

#### FR-3.2.2 Layer Lifecycle

On startup, the WM shall:

1. Call `list_screens()` to discover available displays and their dimensions
2. Create all configured layers via `create_layer(id, width, height, false)` with dimensions matching the target screen
3. Set each layer's source and destination rectangles to cover the full screen
4. Set each layer visible via `set_layer_visibility(id, true, false)`
5. Call `commit()` to apply all changes

#### FR-3.2.3 Layer-to-Screen Assignment

On startup, the WM shall call `add_layers_to_screen(screen_name, layer_ids, false)` for each configured screen, then `commit()`.

In multi-screen setups, different layer sets may be assigned to different screens. For example:
- Center console: all layers (background through priority)
- Instrument cluster: only system overlay + priority layers

### 3.3 Layout Policies

#### FR-3.3.1 Fullscreen Layout

A single surface occupies the entire zone.

- `dest_rect` is set to the zone's full rectangle (e.g., `(0, 0, zone_width, zone_height)`)
- If the surface's `orig_size` aspect ratio differs from the zone, `src_rect` may be adjusted for aspect-ratio-correct scaling, or the surface may be letterboxed/pillarboxed (configurable behavior)

#### FR-3.3.2 Split-Screen Layout

Two surfaces share a zone, divided either horizontally or vertically.

- Configurable split direction: horizontal (left/right) or vertical (top/bottom)
- Configurable split ratio (e.g., 50/50, 60/40, 70/30)
- Primary surface occupies the dominant pane; secondary surface occupies the remainder
- Both surfaces are placed on appropriate layers (main application for primary, secondary application layer for the other)

#### FR-3.3.3 Tiled Layout

N surfaces share a zone in a grid arrangement.

- The WM calculates a grid layout (e.g., 2x2 for 4 surfaces, 2x3 for 5-6 surfaces)
- Each surface's `dest_rect` is calculated as a cell in the grid
- Configurable gap/margin between tiles (in pixels)

#### FR-3.3.4 Picture-in-Picture (PiP) Layout

A secondary surface is displayed as a small overlay within the zone.

- Configurable PiP position: corner (top-left, top-right, bottom-left, bottom-right) or specific coordinates
- Configurable PiP size: percentage of zone dimensions or absolute pixel dimensions
- The PiP surface is placed on the popup/overlay layer for correct z-ordering above the main surface

#### FR-3.3.5 Layout Transitions

When switching between layout policies (e.g., fullscreen to split-screen), the WM shall:

- Calculate new geometries for all affected surfaces
- Support two transition modes:
  - **Instant**: Single atomic `commit()` — all surfaces jump to their new positions
  - **Animated**: Stepped `dest_rect` updates over a configurable duration, with intermediate `commit()` calls at each step
- Animation parameters (duration in ms, step count) shall be configurable per-zone or globally

### 3.4 Zone Definition

#### FR-3.4.1 Zone Configuration

Each screen shall be divided into named zones defined by rectangles. Example for a 1920x1080 center console:

| Zone Name | Rectangle | Purpose |
|-----------|-----------|---------|
| `main` | `(0, 64, 1920, 952)` | Main content area below status bar |
| `status_bar` | `(0, 0, 1920, 64)` | Top status bar |
| `bottom_bar` | `(0, 1016, 1920, 64)` | Bottom control bar |
| `popup` | `(360, 190, 1200, 700)` | Centered popup/dialog area |

#### FR-3.4.2 Per-Screen Zone Definitions

Each screen has independent zone definitions. Example:
- Center console (1920x1080): main, status_bar, bottom_bar, popup zones
- Instrument cluster (1280x480): speedometer, turn_by_turn, warning zones

#### FR-3.4.3 Overlapping Zones

Zones may overlap. A popup zone may overlay the main zone. Z-ordering between overlapping zones is determined by the layer hierarchy (surfaces in the popup zone are placed on a higher layer than surfaces in the main zone).

### 3.5 Focus Management

#### FR-3.5.1 Focus Tracking

The WM shall track the currently focused surface by subscribing to `FocusChanged` notifications. When focus must change, the WM shall call `set_surface_focus(id, false)` followed by `commit()`.

#### FR-3.5.2 Focus Policy

The WM shall support the following focus assignment strategies (configurable):

- **Follow activation**: Focus follows the most recently activated (brought to foreground) surface
- **Follow topmost**: Focus follows the topmost visible surface in the highest-priority interactive layer
- **Explicit only**: Focus changes only via the WM control API (`activate_surface` method)

### 3.6 Priority and Preemption

#### FR-3.6.1 Priority Hierarchy

Each role shall have a configurable priority level (integer, higher = more important). The default hierarchy:

| Priority | Role Category |
|----------|--------------|
| 1 | Background |
| 2 | Entertainment (media, games) |
| 3 | Productivity (navigation, climate) |
| 4 | System UI (status bar, settings) |
| 5 | Notifications / popups |
| 6 | Priority / safety-critical (camera, alerts) |

#### FR-3.6.2 Priority Preemption

When a surface with a higher priority than the currently displayed surface in a zone needs to be shown:

1. The WM shall preempt the current layout, saving the previous layout state
2. Position the priority surface according to its role's configured layout (typically fullscreen)
3. Apply all changes atomically via `commit()`
4. When the priority surface is destroyed or deactivated, restore the previous layout state

#### FR-3.6.3 Popup and Notification Management

- Popup surfaces shall appear on the popup/dialog layer
- Popup positioning shall be configurable: center, top, bottom, or specific coordinates within the popup zone
- Auto-dismiss timeout: configurable per role (e.g., notifications dismiss after 5 seconds)
- Maximum concurrent popups: configurable (excess popups shall be queued)

### 3.7 Multi-Screen Management

#### FR-3.7.1 Independent Per-Screen Layout

Each screen shall have its own set of zones, layout policies, and managed surfaces. Layout changes on one screen shall not affect other screens unless explicitly requested.

#### FR-3.7.2 Per-Screen Layer Assignment

Different screens may receive different subsets of layers. The configuration shall specify which layers are assigned to which screens via `add_layers_to_screen()`.

#### FR-3.7.3 Cross-Screen Surface Movement

The WM control API shall support moving a surface from one screen to another. This involves:

1. Remove the surface from its current layer via `remove_surface_from_layer()`
2. Add it to the appropriate layer on the target screen via `add_surface_to_layer()`
3. Recalculate layout for both the source and target zones
4. Apply all changes atomically via `commit()`

### 3.8 Opacity and Visual Effects

#### FR-3.8.1 Layer Opacity Control

The WM shall support setting layer opacity via `set_layer_opacity()` for effects such as dimming background content when a popup appears. Dimming parameters (target opacity, affected layers) shall be configurable.

#### FR-3.8.2 Surface Opacity Transitions

When animated transitions are enabled, the WM may use stepped `set_surface_opacity()` calls to implement fade-in and fade-out effects for surface appearance and disappearance.

---

## 4. Non-Functional Requirements

### 4.1 Performance

#### NFR-4.1.1 Layout Response Time

From `SurfaceCreated` notification receipt to `commit()` call: **less than 50ms**.

This is achievable because the `ivi-client` library uses synchronous RPC over UNIX domain sockets with 4-byte length-prefixed framing, which has very low overhead.

#### NFR-4.1.2 Priority Preemption Latency

From priority event trigger to surface visible on screen: **less than 100ms**.

The WM shall pre-create all layers at startup so that preemption only requires `add_surface_to_layer` + `set_surface_destination_rectangle` + `set_surface_visibility` + `commit`.

#### NFR-4.1.3 Animation Frame Rate

If animated transitions are enabled, the WM shall target a minimum of **30 FPS** (33ms per frame). Each animation step requires a batch of geometry updates + `commit()`.

### 4.2 Reliability

#### NFR-4.2.1 Controller Connection Recovery

If the connection to the IVI controller socket is lost, the WM shall attempt reconnection with exponential backoff (starting at 100ms, maximum 10s). On reconnection, the WM shall call `list_surfaces()`, `list_layers()`, and `list_screens()` to rebuild its internal state.

#### NFR-4.2.2 Graceful Degradation

If the WM crashes, the IVI controller and Weston continue to run. Surfaces remain in their last committed state. On restart, the WM shall synchronize with the current controller state by querying all surfaces, layers, and screens.

#### NFR-4.2.3 State Consistency Verification

The WM shall periodically (configurable interval, default 5000ms) call `list_surfaces()` and `list_layers()` to verify its internal state matches the controller. Discrepancies shall be logged and corrected.

### 4.3 Configurability

#### NFR-4.3.1 Configuration Format

TOML configuration file. The configuration shall cover:

- Controller socket path
- WM control socket path
- Screen and zone definitions
- Layer hierarchy
- Role-to-surface mappings
- Layout policies and parameters
- Priority rules
- Animation parameters
- State sync interval

#### NFR-4.3.2 Runtime Reconfiguration

The following parameters shall be changeable at runtime via the WM control API without restart:

- Active layout policy for a zone
- Split ratio in split-screen mode
- PiP position and size
- Role assignment for a surface

### 4.4 Logging and Diagnostics

#### NFR-4.4.1 Structured Logging

The WM shall use the `jlogger-tracing` crate (consistent with the controller and client library) for structured logging. It shall log:

- All layout decisions (surface placement, zone assignment)
- Priority preemption events
- Focus changes
- Connection state changes
- Errors and warnings

Logging level shall be controlled via the `RUST_LOG` environment variable (e.g., `RUST_LOG=ivi_wm=debug`).

---

## 5. Interface Requirements

### 5.1 IVI Controller Interface (Downstream — Consumed)

The WM consumes the following APIs from the `ivi-client` library:

#### Surface Operations (via `IviClient`)

| Method | Signature | Purpose |
|--------|-----------|---------|
| `list_surfaces` | `() -> Result<Vec<IviSurface>>` | Initial state synchronization |
| `get_surface` | `(id: u32) -> Result<IviSurface>` | Query individual surface properties |
| `set_surface_source_rectangle` | `(id, x, y, width, height, auto_commit) -> Result<()>` | Set buffer crop region |
| `set_surface_destination_rectangle` | `(id, x, y, width, height, auto_commit) -> Result<()>` | Set screen position and size |
| `set_surface_visibility` | `(id, visible, auto_commit) -> Result<()>` | Show or hide surface |
| `set_surface_opacity` | `(id, opacity, auto_commit) -> Result<()>` | Set transparency (0.0-1.0) |
| `set_surface_z_order` | `(id, z_order, auto_commit) -> Result<()>` | Set stacking order within layer |
| `set_surface_focus` | `(id, auto_commit) -> Result<()>` | Route keyboard/pointer input |

#### Layer Operations

| Method | Signature | Purpose |
|--------|-----------|---------|
| `list_layers` | `() -> Result<Vec<IviLayer>>` | Query all layers |
| `get_layer` | `(id: u32) -> Result<IviLayer>` | Query individual layer |
| `create_layer` | `(id, width, height, auto_commit) -> Result<IviRequestResult>` | Create a new layer |
| `destroy_layer` | `(id, auto_commit) -> Result<()>` | Destroy a layer |
| `set_layer_source_rectangle` | `(id, x, y, width, height, auto_commit) -> Result<()>` | Set layer crop region |
| `set_layer_destination_rectangle` | `(id, x, y, width, height, auto_commit) -> Result<()>` | Set layer position and size |
| `set_layer_visibility` | `(id, visible, auto_commit) -> Result<()>` | Show or hide layer |
| `set_layer_opacity` | `(id, opacity, auto_commit) -> Result<()>` | Set layer transparency |
| `add_surface_to_layer` | `(layer_id, surface_id, auto_commit) -> Result<()>` | Assign surface to layer |
| `remove_surface_from_layer` | `(layer_id, surface_id, auto_commit) -> Result<()>` | Remove surface from layer |
| `get_layer_surfaces` | `(layer_id) -> Result<Vec<u32>>` | List surfaces in a layer |
| `set_surfaces_on_layer` | `(layer_id, surface_ids, auto_commit) -> Result<()>` | Set ordered surface list on layer |

#### Screen Operations

| Method | Signature | Purpose |
|--------|-----------|---------|
| `list_screens` | `() -> Result<Vec<IviScreen>>` | Discover available displays |
| `get_screen` | `(name: &str) -> Result<IviScreen>` | Get screen properties |
| `get_screen_layers` | `(screen_name: &str) -> Result<Vec<u32>>` | List layers on a screen |
| `get_layer_screens` | `(layer_id: u32) -> Result<Vec<String>>` | List screens showing a layer |
| `add_layers_to_screen` | `(screen_name, layer_ids, auto_commit) -> Result<()>` | Assign layers to screen |
| `remove_layer_from_screen` | `(screen_name, layer_id, auto_commit) -> Result<()>` | Remove layer from screen |

#### Global Operations

| Method | Signature | Purpose |
|--------|-----------|---------|
| `commit` | `() -> Result<()>` | Apply all pending changes atomically |

#### Event Subscriptions (via `NotificationListener`)

| Method | Signature | Purpose |
|--------|-----------|---------|
| `new` | `(remote: Option<&str>) -> Result<Self>` | Create listener connection |
| `on` | `(event_type, callback)` | Register per-event callback |
| `on_all` | `(callback)` | Register catch-all callback |
| `start` | `(event_types: &[EventType]) -> Result<()>` | Subscribe and start listening |
| `stop` | `()` | Stop listener thread |

**Subscribed event types:**

| Event Type | Usage |
|-----------|-------|
| `SurfaceCreated` | Detect new surfaces for role assignment and placement |
| `SurfaceContentReady` | Trigger layout when surface has valid content |
| `SurfaceContentSizeChanged` | Detect application resize for layout recalculation |
| `SurfaceDestroyed` | Clean up and recalculate layout |
| `VisibilityChanged` | Track external visibility changes |
| `FocusChanged` | Synchronize focus state |
| `DestinationGeometryChanged` | Detect external geometry changes |
| `LayerCreated` | Track externally created layers |
| `LayerDestroyed` | Handle unexpected layer removal |

### 5.2 Window Manager Control API (Upstream — Exposed)

The WM shall expose a JSON-RPC 2.0 API over a UNIX domain socket (default: `/tmp/ivi-window-manager.sock`) with length-prefixed framing (4-byte big-endian uint32, consistent with the IVI controller protocol).

#### Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `set_layout` | `zone: string, layout: string, params: object` | Switch layout policy for a zone. `layout` is one of: `"fullscreen"`, `"split_h"`, `"split_v"`, `"tiled"`, `"pip"`. `params` contains layout-specific options (e.g., `ratio`, `pip_position`). |
| `assign_role` | `surface_id: u32, role: string` | Assign or change the role of a surface. Triggers re-evaluation of zone and layer assignment. |
| `get_layout_state` | `screen: string` (optional) | Query current layout state for a screen or all screens. Returns zones, assigned surfaces, and active policies. |
| `activate_surface` | `surface_id: u32` | Bring a surface to the foreground in its zone and grant it focus. |
| `deactivate_surface` | `surface_id: u32` | Hide a surface and recalculate zone layout. |
| `set_split_ratio` | `zone: string, ratio: f32` | Change the split ratio in split-screen mode (0.0-1.0, representing primary pane proportion). |
| `set_pip_position` | `zone: string, position: string, width: i32, height: i32` | Change PiP overlay position (`"top_left"`, `"top_right"`, `"bottom_left"`, `"bottom_right"`) and size. |
| `trigger_priority` | `role: string, zone: string` | Manually trigger a priority preemption for surfaces of the given role. |
| `clear_priority` | `role: string` | End a priority preemption and restore previous layout. |
| `list_managed_surfaces` | `screen: string` (optional) | List all surfaces with their assigned roles, zones, and current geometry. |
| `move_surface_to_screen` | `surface_id: u32, target_screen: string, target_zone: string` | Move a surface from its current screen/zone to a different screen/zone. |

### 5.3 Configuration File Interface

The WM reads a TOML configuration file at startup. Default path: `/etc/ivi-wm/config.toml` (overridable via `--config` CLI argument).

#### Example Configuration

```toml
[general]
controller_socket = "/tmp/weston-ivi-controller.sock"
wm_control_socket = "/tmp/ivi-window-manager.sock"
state_sync_interval_ms = 5000
default_role = "generic"

# --- Screen Definitions ---

[screens.center_console]
name = "HDMI-A-1"
layers = ["background", "main_app", "secondary_app", "popup", "system_overlay", "priority"]

[screens.center_console.zones.status_bar]
x = 0
y = 0
width = 1920
height = 64

[screens.center_console.zones.main]
x = 0
y = 64
width = 1920
height = 952
default_layout = "fullscreen"

[screens.center_console.zones.bottom_bar]
x = 0
y = 1016
width = 1920
height = 64

[screens.center_console.zones.popup]
x = 360
y = 190
width = 1200
height = 700

[screens.instrument_cluster]
name = "HDMI-A-2"
layers = ["system_overlay", "priority"]

[screens.instrument_cluster.zones.main]
x = 0
y = 0
width = 1280
height = 480
default_layout = "tiled"

# --- Layer Definitions ---

[layers.background]
id = 100
z_order = 0

[layers.main_app]
id = 200
z_order = 1

[layers.secondary_app]
id = 300
z_order = 2

[layers.popup]
id = 400
z_order = 3

[layers.system_overlay]
id = 500
z_order = 4

[layers.priority]
id = 600
z_order = 5

# --- Role Definitions ---

[roles.navigation]
surface_ids = [1000, 1001]
default_zone = "main"
default_layer = "main_app"
priority = 3

[roles.media]
surface_id_range = [2000, 2999]
default_zone = "main"
default_layer = "main_app"
priority = 2

[roles.status]
surface_ids = [4000]
default_zone = "status_bar"
default_layer = "system_overlay"
priority = 4

[roles.rear_camera]
surface_ids = [3000]
default_zone = "main"
default_layer = "priority"
priority = 6
preempts = true

[roles.notification]
surface_id_range = [5000, 5999]
default_zone = "popup"
default_layer = "popup"
priority = 5
auto_dismiss_ms = 5000
max_concurrent = 3

[roles.generic]
default_zone = "main"
default_layer = "main_app"
priority = 1

# --- Animation Settings ---

[animations]
enabled = true
default_duration_ms = 300
steps = 10

# --- Focus Settings ---

[focus]
policy = "follow_activation"  # "follow_activation", "follow_topmost", "explicit_only"

# --- Popup Dimming ---

[effects.popup_dimming]
enabled = true
target_opacity = 0.5
affected_layers = ["background", "main_app", "secondary_app"]
```

---

## 6. Data Model

The WM shall maintain the following internal data structures:

### ManagedSurface

Wraps `IviSurface` with window management metadata:

| Field | Type | Description |
|-------|------|-------------|
| `surface` | `IviSurface` | Underlying surface state from controller |
| `role` | `String` | Assigned role name |
| `zone` | `String` | Assigned zone name |
| `screen` | `String` | Screen this surface is displayed on |
| `layer_id` | `u32` | Layer this surface is assigned to |
| `priority` | `u32` | Priority level (from role configuration) |
| `is_content_ready` | `bool` | Whether the surface has committed its first buffer |

### Zone

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Zone identifier |
| `screen_name` | `String` | Parent screen name |
| `rect` | `Rectangle` | Zone bounds (x, y, width, height) |
| `current_layout` | `LayoutPolicy` | Active layout policy |
| `assigned_surfaces` | `Vec<u32>` | Ordered list of surface IDs in this zone |
| `saved_state` | `Option<ZoneState>` | Saved layout state for priority preemption restore |

### LayoutPolicy

| Variant | Fields | Description |
|---------|--------|-------------|
| `Fullscreen` | — | Single surface fills entire zone |
| `SplitH` | `ratio: f32` | Horizontal split (left/right) |
| `SplitV` | `ratio: f32` | Vertical split (top/bottom) |
| `Tiled` | `gap: i32` | Grid arrangement with gap between tiles |
| `PictureInPicture` | `pip_surface: u32, position: PipPosition, width: i32, height: i32` | Main surface + overlay |

### LayerConfig

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Layer identifier in configuration |
| `id` | `u32` | IVI layer ID |
| `z_order` | `i32` | Stacking order |

### PriorityEvent

| Field | Type | Description |
|-------|------|-------------|
| `role` | `String` | Role that triggered preemption |
| `zone` | `String` | Zone where preemption is active |
| `saved_layout` | `LayoutPolicy` | Layout policy before preemption |
| `saved_surfaces` | `Vec<u32>` | Surface assignment before preemption |

---

## 7. Traceability Matrix

Mapping of key requirements to the IVI client API methods they depend on:

| Requirement | IVI Client API Methods |
|-------------|----------------------|
| FR-3.1.1 Surface Discovery | `NotificationListener::start([SurfaceCreated, SurfaceDestroyed, SurfaceContentReady])`, `IviClient::get_surface()` |
| FR-3.1.3 Surface Placement | `add_surface_to_layer()`, `set_surface_destination_rectangle()`, `set_surface_source_rectangle()`, `set_surface_visibility()`, `set_surface_z_order()`, `commit()` |
| FR-3.1.4 Destruction Handling | `set_surface_focus()`, `commit()` |
| FR-3.2.2 Layer Lifecycle | `list_screens()`, `create_layer()`, `set_layer_visibility()`, `set_layer_destination_rectangle()`, `set_layer_source_rectangle()`, `commit()` |
| FR-3.2.3 Layer-Screen Assignment | `add_layers_to_screen()`, `commit()` |
| FR-3.3.1-4 Layout Policies | `set_surface_destination_rectangle()`, `set_surface_source_rectangle()`, `commit()` |
| FR-3.3.5 Animated Transitions | `set_surface_destination_rectangle()` (repeated), `set_surface_opacity()` (optional), `commit()` (per frame) |
| FR-3.5.1 Focus Tracking | `NotificationListener::on(FocusChanged)`, `set_surface_focus()`, `commit()` |
| FR-3.6.2 Priority Preemption | `set_surface_visibility()`, `set_layer_opacity()`, `add_surface_to_layer()`, `set_surface_destination_rectangle()`, `commit()` |
| FR-3.7.3 Cross-Screen Movement | `remove_surface_from_layer()`, `add_surface_to_layer()`, `set_surface_destination_rectangle()`, `commit()` |
| FR-3.8.1 Layer Opacity | `set_layer_opacity()`, `commit()` |
| FR-3.8.2 Surface Opacity Transitions | `set_surface_opacity()` (repeated), `commit()` (per frame) |
| NFR-4.2.1 Connection Recovery | `IviClient::new()`, `list_surfaces()`, `list_layers()`, `list_screens()` |
| NFR-4.2.3 State Verification | `list_surfaces()`, `list_layers()` |
