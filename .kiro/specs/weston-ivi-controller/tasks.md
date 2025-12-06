# Implementation Plan

- [x] 1. Set up project structure and build system
  - Update Cargo.toml with dependencies and build configuration
  - Create build.rs script to generate IVI bindings using bindgen
  - Set up module structure (ffi, controller, rpc, transport)
  - Create basic module files with placeholder implementations
  - _Requirements: 1.1, 1.2, 1.4_

- [x] 2. Generate and wrap IVI FFI bindings
- [x] 2.1 Implement build script for bindgen
  - Write build.rs to generate bindings from ivi-shell/ivi-layout-export.h
  - Configure bindgen to allowlist IVI types, functions, and constants
  - Set up bindings module to include generated code
  - _Requirements: 1.4_

- [x] 2.2 Create safe IVI API wrapper
  - Implement IviLayoutApi struct wrapping the C API pointer
  - Implement IviSurface and IviLayer safe wrapper types
  - Add methods for surface property queries and modifications
  - Add methods for layer management
  - Implement commit operations
  - _Requirements: 2.1, 2.2, 3.1, 3.2, 4.1, 5.1, 6.1, 7.1, 7.2_

- [ ]* 2.3 Write property test for position update
  - **Property 1: Position update correctness**
  - **Validates: Requirements 2.1**

- [ ]* 2.4 Write property test for size update
  - **Property 2: Size update correctness**
  - **Validates: Requirements 2.2**

- [-] 3. Implement input validation
- [x] 3.1 Add validation for position and size parameters
  - Implement bounds checking for position coordinates
  - Implement validation for size dimensions (positive non-zero)
  - Implement validation for opacity range [0.0, 1.0]
  - Implement validation for orientation values (multiples of 90)
  - Implement validation for z-order ranges
  - _Requirements: 2.3, 2.4, 4.4, 5.4, 6.2, 6.3_

- [ ]* 3.2 Write property test for position bounds validation
  - **Property 3: Position bounds validation**
  - **Validates: Requirements 2.3**

- [ ]* 3.3 Write property test for size validation
  - **Property 4: Size validation**
  - **Validates: Requirements 2.4**

- [ ]* 3.4 Write property test for opacity validation
  - **Property 15: Opacity validation**
  - **Validates: Requirements 6.3**

- [ ]* 3.5 Write property test for orientation validation
  - **Property 13: Orientation validation**
  - **Validates: Requirements 5.4**

- [x] 4. Implement state management
- [x] 4.1 Create state manager with surface tracking
  - Define SurfaceState and related data structures
  - Implement StateManager with HashMap for surface storage
  - Add methods to add, remove, and update surface state
  - Add methods to query surface state
  - _Requirements: 8.1, 8.2, 8.3, 8.5_

- [x] 4.2 Implement surface lifecycle event handlers
  - Register listeners for surface creation events
  - Register listeners for surface destruction events
  - Register listeners for surface configuration events
  - Update internal state on lifecycle events
  - _Requirements: 8.1, 8.2, 8.3_

- [ ]* 4.3 Write property test for surface creation tracking
  - **Property 20: Surface creation tracking**
  - **Validates: Requirements 8.1**

- [ ]* 4.4 Write property test for surface destruction tracking
  - **Property 21: Surface destruction tracking**
  - **Validates: Requirements 8.2**

- [ ]* 4.5 Write property test for state synchronization
  - **Property 22: State synchronization**
  - **Validates: Requirements 8.3**

- [ ]* 4.6 Write property test for visibility independence
  - **Property 8: Visibility independence**
  - **Validates: Requirements 3.4**

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Implement RPC protocol
- [x] 6.1 Define RPC message types
  - Define RpcRequest and RpcResponse structures
  - Define RpcMethod enum with all supported operations
  - Define RpcError structure
  - Implement serialization/deserialization with serde
  - _Requirements: 9.2, 9.3_

- [x] 6.2 Implement RPC request handler
  - Create request router that maps methods to operations
  - Implement handlers for each RPC method (list_surfaces, get_surface, set_position, etc.)
  - Integrate with StateManager for state queries
  - Integrate with IVI wrapper for control operations
  - Implement error handling and response generation
  - _Requirements: 9.2, 9.5, 12.1, 12.2, 12.4, 13.1, 13.2_

- [ ]* 6.3 Write property test for RPC request round-trip
  - **Property 25: RPC request round-trip**
  - **Validates: Requirements 9.2**

- [ ]* 6.4 Write property test for RPC response round-trip
  - **Property 26: RPC response round-trip**
  - **Validates: Requirements 9.3**

- [ ]* 6.5 Write property test for invalid request error handling
  - **Property 36: Invalid request error handling**
  - **Validates: Requirements 13.1**

- [ ]* 6.6 Write property test for surface list completeness
  - **Property 32: Surface list completeness**
  - **Validates: Requirements 12.1**

- [ ]* 6.7 Write property test for property retrieval completeness
  - **Property 33: Property retrieval completeness**
  - **Validates: Requirements 12.2**

- [x] 7. Implement transport abstraction
- [x] 7.1 Define transport trait and types
  - Define Transport trait with start, stop, send, and register_handler methods
  - Define MessageHandler trait for handling incoming messages
  - Define ClientId and TransportError types
  - _Requirements: 10.1, 10.5_

- [x] 7.2 Implement transport integration in RPC module
  - Add transport registration to RPC module
  - Implement MessageHandler for RPC request processing
  - Wire transport message reception to request handler
  - Wire response generation to transport send
  - _Requirements: 10.2, 10.3, 10.4_

- [ ]* 7.3 Write property test for RPC validation independence
  - **Property 27: RPC validation independence**
  - **Validates: Requirements 9.5**

- [x] 8. Implement UNIX domain socket transport
- [x] 8.1 Create UNIX socket transport implementation
  - Implement UnixSocketTransport struct
  - Implement socket creation and binding
  - Implement connection acceptance
  - Implement non-blocking I/O with mio or tokio
  - Implement message framing (length-prefixed or newline-delimited)
  - _Requirements: 11.1, 11.2, 11.3_

- [x] 8.2 Add concurrent connection handling
  - Implement connection tracking with unique client IDs
  - Handle multiple concurrent connections
  - Implement per-client message queues if needed
  - _Requirements: 11.4_

- [x] 8.3 Implement connection cleanup
  - Detect client disconnections
  - Clean up resources for disconnected clients
  - Notify message handler of disconnections
  - _Requirements: 11.5, 13.3_

- [ ]* 8.4 Write property test for connection acceptance
  - **Property 29: Connection acceptance**
  - **Validates: Requirements 11.3**

- [ ]* 8.5 Write property test for concurrent connection handling
  - **Property 30: Concurrent connection handling**
  - **Validates: Requirements 11.4**

- [ ]* 8.6 Write property test for connection cleanup
  - **Property 31: Connection cleanup**
  - **Validates: Requirements 11.5**

- [ ]* 8.7 Write property test for client isolation
  - **Property 38: Client isolation**
  - **Validates: Requirements 13.3**

- [x] 9. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. Implement Weston plugin interface
- [x] 10.1 Create FFI plugin entry points
  - Implement wet_module_init function
  - Implement wet_module_destroy function
  - Handle plugin initialization arguments
  - Retrieve IVI layout API from Weston compositor
  - _Requirements: 1.3, 1.4_

- [x] 10.2 Wire components together in plugin initialization
  - Create IviLayoutApi wrapper
  - Create StateManager
  - Create RPC handler
  - Create and register UNIX socket transport
  - Register IVI event listeners
  - Start transport
  - _Requirements: 1.3, 10.2_

- [x] 10.3 Implement plugin cleanup
  - Stop transport
  - Clean up all resources
  - Unregister event listeners
  - _Requirements: 1.5_

- [ ]* 10.4 Write unit test for plugin initialization
  - Test that plugin initializes without errors
  - _Requirements: 1.3_

- [ ]* 10.5 Write unit test for plugin cleanup
  - Test that plugin cleans up all resources
  - _Requirements: 1.5_

- [x] 11. Implement error handling and logging
- [x] 11.1 Define error types
  - Create ControllerError enum with all error variants
  - Implement Display and Error traits
  - Add error context and details
  - _Requirements: 13.1, 13.2_

- [x] 11.2 Add error logging
  - Initialize logging framework
  - Add error logging throughout the codebase
  - Add debug logging for important operations
  - _Requirements: 13.4_

- [ ]* 11.3 Write property test for error logging
  - **Property 39: Error logging**
  - **Validates: Requirements 13.4**

- [ ]* 11.4 Write property test for IVI error propagation
  - **Property 37: IVI error propagation**
  - **Validates: Requirements 13.2**

- [x] 12. Implement remaining surface control operations
- [x] 12.1 Add focus control operations
  - Implement keyboard focus setting
  - Implement pointer focus setting
  - Implement focus change notifications
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

- [x] 12.2 Add notification system
  - Implement geometry change notifications
  - Implement focus change notifications
  - Wire notifications to IVI event system
  - _Requirements: 2.5, 7.4_

- [ ]* 12.3 Write property test for keyboard focus correctness
  - **Property 16: Keyboard focus correctness**
  - **Validates: Requirements 7.1**

- [ ]* 12.4 Write property test for pointer focus correctness
  - **Property 17: Pointer focus correctness**
  - **Validates: Requirements 7.2**

- [ ]* 12.5 Write property test for focus exclusivity
  - **Property 18: Focus exclusivity**
  - **Validates: Requirements 7.3**

- [ ]* 12.6 Write property test for focus change notification
  - **Property 19: Focus change notification**
  - **Validates: Requirements 7.4**

- [ ]* 12.7 Write property test for geometry change notification
  - **Property 5: Geometry change notification**
  - **Validates: Requirements 2.5**

- [x] 13. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
