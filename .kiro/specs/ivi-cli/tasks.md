# Implementation Plan

- [x] 1. Set up project structure and dependencies
  - Update workspace Cargo.toml to add `ivi-client` and `ivi-cli` as members
  - Create `ivi-client/` directory with Cargo.toml, src/, include/, examples/, tests/
  - Create `ivi-cli/` directory with Cargo.toml and src/
  - Configure ivi-client Cargo.toml with crate types (rlib, staticlib, cdylib)
  - Add new dependencies to workspace (clap for CLI, proptest for testing)
  - Update root README.md to document the new components
  - _Requirements: 1.1, 2.1, 3.1, 7.1_

- [x] 2. Implement core data types and error handling
  - [x] 2.1 Define Rust data structures for Surface, Layer, Position, Size, Orientation
    - Create types module with all IVI data structures
    - Implement serde serialization/deserialization for JSON-RPC
    - Add Display and Debug implementations
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  
  - [x] 2.2 Implement error types and conversions
    - Create IviError enum with all error variants
    - Implement Display and Error traits
    - Add From implementations for io::Error and serde_json::Error
    - _Requirements: 2.5, 5.3, 5.5_
  
  - [ ]* 2.3 Write property test for error propagation
    - **Property 2: Error Propagation Consistency**
    - **Validates: Requirements 2.5**

- [x] 3. Implement JSON-RPC protocol layer
  - [x] 3.1 Create JSON-RPC request and response structures
    - Define JsonRpcRequest and JsonRpcResponse types
    - Implement serialization/deserialization
    - Add request ID generation logic
    - _Requirements: 2.3, 2.4_
  
  - [ ]* 3.2 Write property test for request-response round trip
    - **Property 1: Request-Response Round Trip**
    - **Validates: Requirements 2.3, 2.4**

- [x] 4. Implement UNIX socket connection management
  - [x] 4.1 Create IviClient struct with connection handling
    - Implement connect() method with socket path parameter
    - Add disconnect() method for clean shutdown
    - Implement request ID counter with AtomicU64
    - _Requirements: 2.2, 5.1, 5.4_
  
  - [x] 4.2 Implement send_request() helper method
    - Serialize request to JSON
    - Send over UNIX socket with newline termination
    - Receive response from socket
    - Deserialize response and handle errors
    - _Requirements: 2.3, 2.4, 5.2_
  
  - [ ]* 4.3 Write property test for connection reusability
    - **Property 5: Connection Reusability**
    - **Validates: Requirements 5.2**

- [x] 5. Implement surface operation methods
  - [x] 5.1 Implement list_surfaces() method
    - Create JSON-RPC request for list_surfaces
    - Parse response into Vec<Surface>
    - _Requirements: 4.1, 8.1_
  
  - [x] 5.2 Implement get_surface() method
    - Create JSON-RPC request for get_surface with ID parameter
    - Parse response into Surface struct
    - _Requirements: 4.1, 9.1_
  
  - [x] 5.3 Implement surface modification methods
    - Implement set_surface_position(id, x, y)
    - Implement set_surface_size(id, width, height)
    - Implement set_surface_visibility(id, visible)
    - Implement set_surface_opacity(id, opacity)
    - Implement set_surface_orientation(id, orientation)
    - Implement set_surface_z_order(id, z_order)
    - Implement set_surface_focus(id)
    - _Requirements: 4.1, 10.1, 11.1, 12.1_
  
  - [ ]* 5.4 Write unit tests for surface operations
    - Test list_surfaces with mock responses
    - Test get_surface with valid and invalid IDs
    - Test surface modification methods
    - _Requirements: 4.1_

- [x] 6. Implement layer operation methods
  - [x] 6.1 Implement list_layers() method
    - Create JSON-RPC request for list_layers
    - Parse response into Vec<Layer>
    - _Requirements: 4.2, 13.1_
  
  - [x] 6.2 Implement get_layer() method
    - Create JSON-RPC request for get_layer with ID parameter
    - Parse response into Layer struct
    - _Requirements: 4.2, 14.1_
  
  - [x] 6.3 Implement layer modification methods
    - Implement set_layer_visibility(id, visible)
    - Implement set_layer_opacity(id, opacity)
    - _Requirements: 4.2, 15.1, 16.1_
  
  - [ ]* 6.4 Write unit tests for layer operations
    - Test list_layers with mock responses
    - Test get_layer with valid and invalid IDs
    - Test layer modification methods
    - _Requirements: 4.2_

- [x] 7. Implement commit operation
  - [x] 7.1 Implement commit() method
    - Create JSON-RPC request for commit
    - Handle response
    - _Requirements: 4.3, 17.1_
  
  - [ ]* 7.2 Write unit test for commit operation
    - Test commit with mock response
    - _Requirements: 4.3_

- [x] 8. Checkpoint - Ensure all Rust API tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 9. Implement C FFI bindings
  - [x] 9.1 Create C-compatible types and enums
    - Define IviPosition, IviSize, IviSurface, IviLayer structs
    - Define IviOrientation and IviErrorCode enums
    - Add repr(C) attributes
    - _Requirements: 3.1, 3.2_
  
  - [x] 9.2 Implement C API connection functions
    - Implement ivi_client_connect() with error buffer
    - Implement ivi_client_disconnect()
    - Add null pointer checks
    - _Requirements: 3.2, 3.3, 3.4_
  
  - [x] 9.3 Implement C API surface functions
    - Implement ivi_list_surfaces() with array allocation
    - Implement ivi_get_surface()
    - Implement all surface modification functions
    - Add error handling and buffer writing
    - _Requirements: 3.2, 3.3, 3.4_
  
  - [x] 9.4 Implement C API layer functions
    - Implement ivi_list_layers() with array allocation
    - Implement ivi_get_layer()
    - Implement layer modification functions
    - _Requirements: 3.2, 3.3, 3.4_
  
  - [x] 9.5 Implement C API memory management functions
    - Implement ivi_free_surfaces()
    - Implement ivi_free_layers()
    - Ensure proper cleanup of allocated memory
    - _Requirements: 3.5_
  
  - [ ]* 9.6 Write property test for FFI type translation
    - **Property 3: FFI Type Translation Consistency**
    - **Validates: Requirements 3.2, 3.3**
  
  - [ ]* 9.7 Write property test for FFI error translation
    - **Property 4: FFI Error Translation**
    - **Validates: Requirements 3.4**

- [x] 10. Generate C header file
  - [x] 10.1 Set up cbindgen configuration
    - Create cbindgen.toml with C language settings
    - Create build.rs to run cbindgen
    - _Requirements: 3.1_
  
  - [x] 10.2 Generate and verify C header
    - Run cbindgen to generate include/ivi_client.h
    - Verify header contains all public C API functions
    - Add documentation comments
    - _Requirements: 7.1_

- [x] 11. Checkpoint - Ensure all C API tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 12. Implement CLI argument parsing
  - [x] 12.1 Define CLI structure with clap
    - Create Cli struct with global options (socket path)
    - Define Commands enum for surface, layer, commit
    - Define SurfaceCommands and LayerCommands enums
    - Add help text and version information
    - _Requirements: 7.4, 19.1, 19.2, 19.3_
  
  - [x] 12.2 Implement input validation
    - Validate opacity range (0.0-1.0)
    - Validate positive dimensions for width/height
    - Validate boolean parsing for visibility
    - _Requirements: 11.3, 16.3, 18.1_
  
  - [ ]* 12.3 Write unit tests for argument parsing
    - Test parsing of all command variants
    - Test validation of input ranges
    - Test error handling for invalid inputs
    - _Requirements: 18.1_

- [x] 13. Implement CLI command handlers
  - [x] 13.1 Implement surface command handlers
    - Implement handle_surface_list()
    - Implement handle_surface_get_properties()
    - Implement handle_surface_set_visibility()
    - Implement handle_surface_set_opacity()
    - Implement handle_surface_set_dest_rect()
    - _Requirements: 8.1, 9.1, 10.1, 11.1, 12.1_
  
  - [x] 13.2 Implement layer command handlers
    - Implement handle_layer_list()
    - Implement handle_layer_get_properties()
    - Implement handle_layer_set_visibility()
    - Implement handle_layer_set_opacity()
    - _Requirements: 13.1, 14.1, 15.1, 16.1_
  
  - [x] 13.3 Implement commit command handler
    - Implement handle_commit()
    - Add --commit flag support to modification commands
    - _Requirements: 17.2, 17.3, 17.4_
  
  - [ ]* 13.4 Write integration tests for CLI commands
    - Test each command with mock IVI controller
    - Verify output formatting
    - Verify exit codes
    - _Requirements: 18.3, 18.4_

- [x] 14. Implement CLI output formatting
  - [x] 14.1 Implement list output formatting
    - Format surface IDs as comma-separated list
    - Format layer IDs as comma-separated list
    - _Requirements: 8.2, 13.2, 20.2_
  
  - [x] 14.2 Implement properties output formatting
    - Format surface properties with labels and indentation
    - Format layer properties with labels and indentation
    - Format numeric values with appropriate precision
    - _Requirements: 9.2, 14.2, 20.1, 20.4_
  
  - [x] 14.3 Implement success and error message formatting
    - Format success messages with checkmark
    - Format error messages with cross mark and error details
    - _Requirements: 10.4, 18.2, 20.3_
  
  - [ ]* 14.4 Write unit tests for output formatting
    - Test formatting of different data types
    - Test success and error message formatting
    - _Requirements: 20.1, 20.2, 20.3, 20.4_

- [x] 15. Implement CLI main function and error handling
  - [x] 15.1 Implement main() function
    - Parse CLI arguments
    - Create IviClient connection
    - Route to appropriate command handler
    - Handle errors and display messages
    - Exit with appropriate status code
    - _Requirements: 7.1, 7.2, 7.3, 7.5, 18.3, 18.4_
  
  - [ ]* 15.2 Write integration tests for CLI error handling
    - Test connection failure handling
    - Test invalid parameter handling
    - Test JSON-RPC error handling
    - Verify exit codes
    - _Requirements: 7.2, 7.5, 18.2, 18.3, 18.4_

- [x] 16. Create documentation and examples
  - [x] 16.1 Write library README
    - Create ivi-client/README.md
    - Document installation instructions
    - Document Rust API usage with examples
    - Document C API usage with examples
    - Document building and linking
    - _Requirements: 7.1_
  
  - [x] 16.2 Create Rust example program
    - Create ivi-client/examples/rust_example.rs
    - Demonstrate connecting to controller
    - Demonstrate listing and modifying surfaces
    - Demonstrate error handling
    - _Requirements: 7.1_
  
  - [x] 16.3 Create C example program
    - Create ivi-client/examples/c_example.c
    - Demonstrate connecting to controller
    - Demonstrate listing and modifying surfaces
    - Demonstrate memory management
    - _Requirements: 7.1_
  
  - [x] 16.4 Write CLI README
    - Create ivi-cli/README.md
    - Document installation instructions
    - Document all CLI commands with examples
    - Document global options
    - _Requirements: 19.1, 19.2_
  
  - [x] 16.5 Add rustdoc comments to all public APIs
    - Document all public Rust functions in ivi-client
    - Add usage examples in doc comments
    - Document error conditions
    - _Requirements: 7.1_
  
  - [x] 16.6 Update project root README
    - Update main README.md to document ivi-client library
    - Add section about ivi-cli tool
    - Update build instructions to include new components
    - _Requirements: 7.1_
  
  - [x] 16.7 Create client library documentation
    - Create docs/client_library.md
    - Document library architecture and design
    - Document integration with the controller plugin
    - _Requirements: 7.1_

- [x] 17. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
