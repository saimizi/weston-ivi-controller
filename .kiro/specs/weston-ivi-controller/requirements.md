# Requirements Document

## Introduction

This document specifies the requirements for a Weston IVI Shell Controller module written in Rust. The module will be implemented as a shared library plugin for the Weston compositor, enabling external applications to control Wayland client applications through IVI (In-Vehicle Infotainment) interfaces via an RPC mechanism over UNIX domain sockets.

## Glossary

- **IVI Shell**: In-Vehicle Infotainment Shell, a Wayland shell protocol designed for automotive and embedded display systems
- **Weston**: The reference implementation of a Wayland compositor
- **Wayland Client**: An application that connects to a Wayland compositor to display graphical content
- **IVI Surface**: A Wayland surface managed by the IVI shell protocol
- **IVI Layer**: A container for IVI surfaces that defines rendering order and visibility
- **RPC Interface**: Remote Procedure Call interface allowing external processes to invoke operations
- **RPC Module**: An independent module that handles RPC request/response processing with pluggable transport mechanisms
- **Transport Layer**: The communication mechanism used to send and receive RPC messages (e.g., UNIX domain socket, TCP, etc.)
- **UNIX Domain Socket**: An inter-process communication mechanism using filesystem paths
- **Controller Module**: The Rust shared library plugin that implements IVI control functionality
- **External Application**: A process that communicates with the Controller Module via the RPC interface
- **Z-Order**: The stacking order of surfaces determining which appears on top

## Requirements

### Requirement 1

**User Story:** As a system integrator, I want the IVI controller to be implemented as a Rust shared library, so that it can be loaded into Weston as a plugin with memory safety guarantees.

#### Acceptance Criteria

1. THE Controller Module SHALL be implemented in Rust
2. THE Controller Module SHALL compile to a shared library compatible with Weston's plugin loading mechanism
3. WHEN Weston loads the plugin, THE Controller Module SHALL initialize without errors
4. THE Controller Module SHALL expose C-compatible FFI functions for Weston integration
5. WHEN the plugin is unloaded, THE Controller Module SHALL clean up all allocated resources

### Requirement 2

**User Story:** As an application developer, I want to control the position and size of Wayland clients, so that I can arrange the display layout programmatically.

#### Acceptance Criteria

1. WHEN an External Application requests position change for an IVI Surface, THE Controller Module SHALL update the surface position to the specified coordinates
2. WHEN an External Application requests size change for an IVI Surface, THE Controller Module SHALL update the surface dimensions to the specified width and height
3. THE Controller Module SHALL validate that position coordinates are within valid display bounds
4. THE Controller Module SHALL validate that size dimensions are positive non-zero values
5. WHEN position or size updates are applied, THE Controller Module SHALL notify the Wayland Client of the geometry change

### Requirement 3

**User Story:** As an application developer, I want to control the visibility of Wayland clients, so that I can show or hide applications based on system state.

#### Acceptance Criteria

1. WHEN an External Application requests to show an IVI Surface, THE Controller Module SHALL make the surface visible
2. WHEN an External Application requests to hide an IVI Surface, THE Controller Module SHALL make the surface invisible
3. WHEN visibility state changes, THE Controller Module SHALL update the surface rendering state immediately
4. THE Controller Module SHALL maintain visibility state for each IVI Surface independently

### Requirement 4

**User Story:** As an application developer, I want to control the z-order of Wayland clients, so that I can determine which applications appear on top.

#### Acceptance Criteria

1. WHEN an External Application requests z-order change for an IVI Surface, THE Controller Module SHALL update the surface stacking order to the specified position
2. THE Controller Module SHALL ensure z-order values are applied consistently across all surfaces in the same layer
3. WHEN z-order changes, THE Controller Module SHALL trigger compositor re-rendering to reflect the new stacking order
4. THE Controller Module SHALL validate that z-order values are within valid range for the target layer

### Requirement 5

**User Story:** As an application developer, I want to control the orientation of Wayland clients, so that I can rotate displays for different viewing angles.

#### Acceptance Criteria

1. WHEN an External Application requests orientation change for an IVI Surface, THE Controller Module SHALL rotate the surface to the specified angle
2. THE Controller Module SHALL support orientation values of 0, 90, 180, and 270 degrees
3. WHEN orientation is changed, THE Controller Module SHALL update the surface transformation matrix
4. THE Controller Module SHALL reject orientation values that are not multiples of 90 degrees

### Requirement 6

**User Story:** As an application developer, I want to adjust the opacity of Wayland clients, so that I can create visual effects and layered interfaces.

#### Acceptance Criteria

1. WHEN an External Application requests opacity change for an IVI Surface, THE Controller Module SHALL update the surface opacity to the specified value
2. THE Controller Module SHALL accept opacity values in the range 0.0 (fully transparent) to 1.0 (fully opaque)
3. THE Controller Module SHALL reject opacity values outside the valid range
4. WHEN opacity changes, THE Controller Module SHALL update the surface rendering properties immediately

### Requirement 7

**User Story:** As an application developer, I want to route input focus to specific Wayland clients, so that I can control which application receives user input.

#### Acceptance Criteria

1. WHEN an External Application requests input focus for an IVI Surface, THE Controller Module SHALL set keyboard focus to that surface
2. WHEN an External Application requests input focus for an IVI Surface, THE Controller Module SHALL set pointer focus to that surface
3. THE Controller Module SHALL remove focus from the previously focused surface when focus is changed
4. WHEN focus changes, THE Controller Module SHALL notify both the old and new focused surfaces

### Requirement 8

**User Story:** As an application developer, I want to monitor application state changes, so that I can respond to lifecycle events of Wayland clients.

#### Acceptance Criteria

1. WHEN an IVI Surface is created, THE Controller Module SHALL detect the creation event and record the surface information
2. WHEN an IVI Surface is destroyed, THE Controller Module SHALL detect the destruction event and remove the surface information
3. WHEN surface state changes occur, THE Controller Module SHALL update the internal state representation
4. THE Controller Module SHALL make surface state information available to External Applications via the RPC interface
5. THE Controller Module SHALL maintain accurate state for all active IVI Surfaces

### Requirement 9

**User Story:** As an application developer, I want to communicate with the IVI controller via a well-defined RPC interface, so that I can control Wayland clients from external processes.

#### Acceptance Criteria

1. THE RPC Module SHALL define a clear API for processing RPC requests independent of the transport mechanism
2. THE RPC Module SHALL parse incoming RPC requests and route them to the appropriate IVI control operations
3. WHEN an RPC operation completes, THE RPC Module SHALL serialize the response in a structured format
4. THE RPC Module SHALL provide a transport abstraction layer that allows different communication mechanisms to be plugged in
5. THE RPC Module SHALL handle request validation and error responses independently of the transport layer

### Requirement 10

**User Story:** As a system integrator, I want the RPC transport to be pluggable, so that I can use different communication mechanisms without changing the core RPC logic.

#### Acceptance Criteria

1. THE RPC Module SHALL define a transport trait or interface that transport implementations must satisfy
2. THE Controller Module SHALL support registering different transport implementations at initialization
3. WHEN a transport implementation is registered, THE RPC Module SHALL use it for all client communication
4. THE RPC Module SHALL operate correctly regardless of which transport implementation is active
5. THE transport abstraction SHALL support connection management, message sending, and message receiving operations

### Requirement 11

**User Story:** As an application developer, I want a UNIX domain socket transport implementation, so that I can communicate with the IVI controller using local inter-process communication.

#### Acceptance Criteria

1. THE Controller Module SHALL provide a UNIX domain socket transport implementation
2. THE UNIX domain socket transport SHALL create a socket at a configurable filesystem path
3. WHEN an External Application connects to the socket, THE transport SHALL accept the connection
4. THE UNIX domain socket transport SHALL handle multiple concurrent client connections
5. WHEN a client connection is lost, THE transport SHALL clean up associated resources

### Requirement 12

**User Story:** As an application developer, I want to query IVI surface information via RPC, so that I can retrieve current state of Wayland clients.

#### Acceptance Criteria

1. WHEN an External Application requests surface list, THE RPC Module SHALL return information about all active IVI Surfaces
2. WHEN an External Application requests specific surface properties, THE RPC Module SHALL return the current position, size, visibility, z-order, orientation, and opacity
3. THE RPC Module SHALL serialize surface information in a structured format for RPC responses
4. WHEN surface information is requested for a non-existent surface, THE RPC Module SHALL return an error response

### Requirement 13

**User Story:** As a system administrator, I want the RPC interface to handle errors gracefully, so that client applications can recover from failures.

#### Acceptance Criteria

1. WHEN an RPC request contains invalid data, THE RPC Module SHALL return an error response with a descriptive message
2. WHEN an RPC operation fails due to IVI interface errors, THE RPC Module SHALL return an error response indicating the failure reason
3. WHEN a client connection is lost, THE transport layer SHALL clean up associated resources and continue serving other clients
4. THE Controller Module SHALL log error conditions for debugging purposes
5. WHEN a transport implementation fails to initialize, THE Controller Module SHALL report the error during initialization
