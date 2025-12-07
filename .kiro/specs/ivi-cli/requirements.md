# Requirements Document

## Introduction

The IVI CLI project consists of two components:

1. **IVI Client Library** - A reusable library that provides a programmatic interface to the Weston IVI controller's JSON-RPC API. The library is implemented in Rust with C FFI bindings, making it usable from both Rust and C applications.

2. **IVI CLI Tool** (`ivi_cli`) - A command-line interface tool built on top of the IVI Client Library that enables developers to interact with and control a Weston IVI (In-Vehicle Infotainment) compositor from the command line.

Together, these components provide both programmatic and interactive access to the IVI controller for testing, debugging, and managing IVI compositor layouts during development and deployment.

## Glossary

- **IVI**: In-Vehicle Infotainment - A system providing entertainment and information in vehicles
- **Weston**: A reference implementation of a Wayland compositor
- **IVI Shell**: A Wayland shell extension designed for automotive display systems
- **Surface**: A visual element representing an application window in the compositor
- **Layer**: A container that groups surfaces and controls their rendering order
- **Screen**: A physical or virtual display output managed by the compositor
- **JSON-RPC**: A remote procedure call protocol encoded in JSON
- **CLI**: Command-Line Interface - A text-based user interface for interacting with software
- **Compositor**: Software that combines visual elements and displays them on screen
- **Z-Order**: The stacking order of visual elements (higher values appear on top)
- **Opacity**: The transparency level of a visual element (0.0 = transparent, 1.0 = opaque)
- **Render Order**: The sequence in which layers are drawn on a screen
- **IVI Client Library**: A library providing programmatic access to the IVI controller JSON-RPC API
- **FFI**: Foreign Function Interface - A mechanism for calling functions written in one language from another
- **C API**: Application Programming Interface designed for use from C programs
- **Rust API**: Application Programming Interface designed for use from Rust programs

## Requirements

### Requirement 1

**User Story:** As a Rust developer, I want a Rust library to interact with the IVI controller, so that I can integrate IVI control into my Rust applications.

#### Acceptance Criteria

1. WHEN a Rust application uses the IVI Client Library THEN the library SHALL provide a safe Rust API for all IVI controller operations
2. WHEN the library connects to the controller THEN the library SHALL establish a connection to the UNIX domain socket
3. WHEN the library sends a request THEN the library SHALL serialize the request to JSON-RPC format and send it over the socket
4. WHEN the library receives a response THEN the library SHALL deserialize the JSON-RPC response and return typed Rust data structures
5. WHEN an error occurs THEN the library SHALL return a Rust Result type with descriptive error information
6. WHEN the library is used in a multi-threaded context THEN the library SHALL provide thread-safe operations

### Requirement 2

**User Story:** As a C developer, I want a C library to interact with the IVI controller, so that I can integrate IVI control into my C applications.

#### Acceptance Criteria

1. WHEN a C application uses the IVI Client Library THEN the library SHALL provide a C-compatible API through FFI bindings
2. WHEN the C API is called THEN the library SHALL translate C types to Rust types and invoke the Rust implementation
3. WHEN the Rust implementation returns a result THEN the library SHALL translate Rust types back to C types
4. WHEN an error occurs in the Rust code THEN the library SHALL return an error code and populate an error message buffer
5. WHEN the C API allocates resources THEN the library SHALL provide corresponding cleanup functions to free those resources
6. WHEN the library is compiled THEN the library SHALL produce both a static library and a shared library for C linking

### Requirement 3

**User Story:** As a library user, I want comprehensive API coverage, so that I can perform all IVI controller operations programmatically.

#### Acceptance Criteria

1. WHEN using the library THEN the library SHALL provide functions for all surface operations including list, get properties, set visibility, set opacity, set position, set size, set orientation, set z-order, and set focus
2. WHEN using the library THEN the library SHALL provide functions for all layer operations including list, get properties, set visibility, and set opacity
3. WHEN using the library THEN the library SHALL provide a commit function to apply pending changes atomically
4. WHEN using the library THEN the library SHALL provide functions to subscribe to and unsubscribe from event notifications
5. WHEN using the library THEN the library SHALL provide a function to receive event notifications from the controller

### Requirement 4

**User Story:** As a library user, I want proper connection management, so that I can reliably communicate with the IVI controller.

#### Acceptance Criteria

1. WHEN creating a client connection THEN the library SHALL accept a socket path parameter with a default value of `/tmp/weston-ivi-controller.sock`
2. WHEN the connection is established THEN the library SHALL maintain the socket connection for multiple requests
3. WHEN the connection fails THEN the library SHALL return an error indicating the connection failure
4. WHEN the client is dropped or closed THEN the library SHALL close the socket connection cleanly
5. WHEN a network error occurs during communication THEN the library SHALL return an error with details about the failure

### Requirement 5

**User Story:** As a library user, I want type-safe data structures, so that I can work with IVI data in a structured and safe manner.

#### Acceptance Criteria

1. WHEN the library represents surface data THEN the library SHALL provide a structured type containing all surface properties
2. WHEN the library represents layer data THEN the library SHALL provide a structured type containing all layer properties
3. WHEN the library represents position data THEN the library SHALL provide a structured type with x and y coordinates
4. WHEN the library represents size data THEN the library SHALL provide a structured type with width and height dimensions
5. WHEN the library represents orientation THEN the library SHALL provide an enumeration type with values for Normal, Rotate90, Rotate180, and Rotate270

### Requirement 6

**User Story:** As a library user, I want comprehensive documentation, so that I can understand how to use the library effectively.

#### Acceptance Criteria

1. WHEN the library is published THEN the library SHALL include rustdoc documentation for all public Rust APIs
2. WHEN the library is published THEN the library SHALL include header file comments for all public C APIs
3. WHEN the library is published THEN the library SHALL include usage examples for common operations in both Rust and C
4. WHEN the library is published THEN the library SHALL include a README with installation and usage instructions

### Requirement 7

**User Story:** As a CLI user, I want to connect to the IVI controller via UNIX socket, so that I can send commands to manage the compositor.

#### Acceptance Criteria

1. WHEN the CLI tool is invoked THEN the CLI SHALL use the IVI Client Library to establish a connection to the UNIX domain socket at `/tmp/weston-ivi-controller.sock`
2. WHEN the socket connection fails THEN the CLI SHALL display a clear error message indicating the connection failure and exit with a non-zero status code
3. WHEN a command completes THEN the CLI SHALL close the socket connection cleanly
4. WHEN the socket path is non-standard THEN the CLI SHALL accept a custom socket path via the `--socket` command-line option
5. WHEN network communication fails THEN the CLI SHALL display the JSON-RPC error code and message to the user

### Requirement 8

**User Story:** As a CLI user, I want to list all available surfaces, so that I can see which application windows are currently active in the compositor.

#### Acceptance Criteria

1. WHEN the user executes `ivi_cli surface list` THEN the CLI SHALL use the library to send a `list_surfaces` request to the controller
2. WHEN the controller responds with surface data THEN the CLI SHALL display the surface IDs in a human-readable format
3. WHEN no surfaces exist THEN the CLI SHALL display a message indicating no surfaces are available
4. WHEN the request fails THEN the CLI SHALL display the error message and exit with a non-zero status code

### Requirement 9

**User Story:** As a CLI user, I want to view detailed properties of a specific surface, so that I can inspect its current configuration.

#### Acceptance Criteria

1. WHEN the user executes `ivi_cli surface get-properties <surface_id>` THEN the CLI SHALL use the library to send a `get_surface` request with the specified surface ID
2. WHEN the controller responds with surface properties THEN the CLI SHALL display all properties including position, size, visibility, opacity, orientation, and z-order in a formatted output
3. WHEN the surface ID does not exist THEN the CLI SHALL display an error message indicating the surface was not found
4. WHEN the surface ID parameter is missing THEN the CLI SHALL display usage information and exit with a non-zero status code

### Requirement 10

**User Story:** As a CLI user, I want to control surface visibility, so that I can show or hide application windows.

#### Acceptance Criteria

1. WHEN the user executes `ivi_cli surface set-visibility <surface_id> <true|false>` THEN the CLI SHALL use the library to send a `set_visibility` request with the specified parameters
2. WHEN the visibility parameter is `true` THEN the CLI SHALL request the surface to be made visible
3. WHEN the visibility parameter is `false` THEN the CLI SHALL request the surface to be hidden
4. WHEN the operation succeeds THEN the CLI SHALL display a success message
5. WHEN the visibility parameter is neither `true` nor `false` THEN the CLI SHALL display an error message and exit with a non-zero status code

### Requirement 11

**User Story:** As a CLI user, I want to adjust surface opacity, so that I can control the transparency of application windows.

#### Acceptance Criteria

1. WHEN the user executes `ivi_cli surface set-opacity <surface_id> <opacity>` THEN the CLI SHALL use the library to send a `set_opacity` request with the specified parameters
2. WHEN the opacity value is between 0.0 and 1.0 THEN the CLI SHALL accept the value as valid
3. WHEN the opacity value is outside the range 0.0 to 1.0 THEN the CLI SHALL display an error message and exit with a non-zero status code
4. WHEN the operation succeeds THEN the CLI SHALL display a success message

### Requirement 12

**User Story:** As a CLI user, I want to set the destination rectangle of a surface, so that I can control where and how large the surface appears on screen.

#### Acceptance Criteria

1. WHEN the user executes `ivi_cli surface set-dest-rect <surface_id> <x> <y> <width> <height>` THEN the CLI SHALL use the library to send `set_position` and `set_size` requests with the specified parameters
2. WHEN all dimension parameters are valid integers THEN the CLI SHALL accept the values
3. WHEN width or height is not a positive integer THEN the CLI SHALL display an error message and exit with a non-zero status code
4. WHEN the operations succeed THEN the CLI SHALL display a success message

### Requirement 13

**User Story:** As a CLI user, I want to list all available layers, so that I can see which layers exist in the compositor.

#### Acceptance Criteria

1. WHEN the user executes `ivi_cli layer list` THEN the CLI SHALL use the library to send a `list_layers` request to the controller
2. WHEN the controller responds with layer data THEN the CLI SHALL display the layer IDs in a human-readable format
3. WHEN no layers exist THEN the CLI SHALL display a message indicating no layers are available

### Requirement 14

**User Story:** As a CLI user, I want to view detailed properties of a specific layer, so that I can inspect its current configuration.

#### Acceptance Criteria

1. WHEN the user executes `ivi_cli layer get-properties <layer_id>` THEN the CLI SHALL use the library to send a `get_layer` request with the specified layer ID
2. WHEN the controller responds with layer properties THEN the CLI SHALL display all properties including visibility and opacity in a formatted output
3. WHEN the layer ID does not exist THEN the CLI SHALL display an error message indicating the layer was not found

### Requirement 15

**User Story:** As a CLI user, I want to control layer visibility, so that I can show or hide entire groups of surfaces.

#### Acceptance Criteria

1. WHEN the user executes `ivi_cli layer set-visibility <layer_id> <true|false>` THEN the CLI SHALL use the library to send a `set_layer_visibility` request with the specified parameters
2. WHEN the visibility parameter is `true` THEN the CLI SHALL request the layer to be made visible
3. WHEN the visibility parameter is `false` THEN the CLI SHALL request the layer to be hidden
4. WHEN the operation succeeds THEN the CLI SHALL display a success message

### Requirement 16

**User Story:** As a CLI user, I want to adjust layer opacity, so that I can control the transparency of entire groups of surfaces.

#### Acceptance Criteria

1. WHEN the user executes `ivi_cli layer set-opacity <layer_id> <opacity>` THEN the CLI SHALL use the library to send a `set_layer_opacity` request with the specified parameters
2. WHEN the opacity value is between 0.0 and 1.0 THEN the CLI SHALL accept the value as valid
3. WHEN the opacity value is outside the range 0.0 to 1.0 THEN the CLI SHALL display an error message and exit with a non-zero status code

### Requirement 17

**User Story:** As a CLI user, I want to commit pending changes atomically, so that I can ensure multiple operations are applied simultaneously without visual artifacts.

#### Acceptance Criteria

1. WHEN the user executes any surface or layer modification command THEN the CLI SHALL use the library to send the request without the `auto_commit` parameter
2. WHEN the user provides the `--commit` flag with any modification command THEN the CLI SHALL use the library to send an additional `commit` request after the modification request
3. WHEN the commit operation succeeds THEN the CLI SHALL display a message indicating changes were committed
4. WHEN the user executes `ivi_cli commit` THEN the CLI SHALL use the library to send a `commit` request to apply all pending changes

### Requirement 18

**User Story:** As a CLI user, I want clear and helpful error messages, so that I can quickly understand and fix issues with my commands.

#### Acceptance Criteria

1. WHEN a command is invoked with incorrect arguments THEN the CLI SHALL display usage information for that specific command
2. WHEN a JSON-RPC error is received THEN the CLI SHALL display the error code and message in a human-readable format
3. WHEN a command fails THEN the CLI SHALL exit with a non-zero status code
4. WHEN a command succeeds THEN the CLI SHALL exit with status code 0

### Requirement 19

**User Story:** As a CLI user, I want to see help information for commands, so that I can learn how to use the CLI tool.

#### Acceptance Criteria

1. WHEN the user executes `ivi_cli --help` THEN the CLI SHALL display general usage information and a list of available resources
2. WHEN the user executes `ivi_cli <resource> --help` THEN the CLI SHALL display usage information for all commands available for that resource
3. WHEN the user executes `ivi_cli --version` THEN the CLI SHALL display the version number of the CLI tool

### Requirement 20

**User Story:** As a CLI user, I want formatted and readable output, so that I can easily understand the information displayed by the CLI.

#### Acceptance Criteria

1. WHEN displaying surface or layer properties THEN the CLI SHALL format the output with clear labels and proper indentation
2. WHEN displaying lists of IDs THEN the CLI SHALL format them in a comma-separated or line-separated format
3. WHEN displaying success messages THEN the CLI SHALL use clear and concise language
4. WHEN displaying numeric values THEN the CLI SHALL format them with appropriate precision
