# Implementation Plan

- [x] 1. Set up core ID assignment data structures and configuration
  - Create `IdAssignmentConfig` struct with default values for start_id (0x10000000), max_id (0xFFFFFFFE), and invalid_id (0xFFFFFFFF)
  - Implement configuration validation and error handling
  - Create error types for ID assignment operations
  - _Requirements: 2.1, 2.3, 2.4_

- [ ]* 1.1 Write property test for configuration validation
  - **Property 5: Assignment range compliance**
  - **Validates: Requirements 2.3**

- [x] 2. Implement Surface ID Registry for tracking active IDs
  - Create `SurfaceIdRegistry` struct using `HashSet<u32>` for active IDs
  - Implement methods for registering, releasing, and checking ID availability
  - Add support for distinguishing auto-assigned vs manual IDs
  - Include registry statistics and monitoring capabilities
  - _Requirements: 3.4, 4.1, 4.2_

- [ ]* 2.1 Write property test for registry accuracy
  - **Property 10: Registry accuracy**
  - **Validates: Requirements 3.4**

- [ ]* 2.2 Write property test for ID reuse after destruction
  - **Property 11: ID reuse after destruction**
  - **Validates: Requirements 4.1, 4.2**

- [x] 3. Implement ID assignment algorithm with wraparound support
  - Create `IdAssigner` struct with sequential assignment logic
  - Implement wraparound behavior when reaching maximum ID
  - Add conflict detection and resolution for occupied IDs
  - Include detailed assignment result reporting
  - _Requirements: 2.2, 2.5, 3.1, 3.2, 3.3_

- [ ]* 3.1 Write property test for sequential ID assignment
  - **Property 4: Sequential ID assignment**
  - **Validates: Requirements 2.1, 2.2**

- [ ]* 3.2 Write property test for wraparound behavior
  - **Property 7: Wraparound behavior**
  - **Validates: Requirements 2.5**

- [ ]* 3.3 Write property test for conflict detection during wraparound
  - **Property 8: Conflict detection during wraparound**
  - **Validates: Requirements 3.1, 3.2**

- [ ]* 3.4 Write property test for persistent conflict resolution
  - **Property 9: Persistent conflict resolution**
  - **Validates: Requirements 3.3**

- [x] 4. Create ID Assignment Manager to coordinate operations
  - Implement `IdAssignmentManager` that combines registry and assigner
  - Add thread-safe operations using `Arc<Mutex<>>` pattern
  - Implement invalid ID detection logic
  - Add comprehensive logging for all assignment operations
  - _Requirements: 1.1, 1.2, 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ]* 4.1 Write property test for invalid ID detection accuracy
  - **Property 1: Invalid ID detection accuracy**
  - **Validates: Requirements 1.1, 1.2**

- [ ]* 4.2 Write property test for valid ID preservation
  - **Property 2: Valid ID preservation**
  - **Validates: Requirements 1.4**

- [ ]* 4.3 Write property test for comprehensive logging
  - **Property 20: Comprehensive logging**
  - **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Integrate ID assignment with existing surface event handling
  - Modify `EventContext` to include `IdAssignmentManager`
  - Enhance `surface_created_callback` to detect invalid IDs and trigger assignment
  - Update `surface_removed_callback` to release auto-assigned IDs
  - Ensure proper error handling and logging integration
  - _Requirements: 1.3, 4.5, 5.4, 5.5_

- [ ]* 6.1 Write property test for comprehensive monitoring
  - **Property 3: Comprehensive monitoring**
  - **Validates: Requirements 1.3**

- [x] 7. Implement surface ID replacement in IVI compositor
  - Add functionality to replace invalid surface ID with assigned ID
  - Implement verification that replacement was successful
  - Add error handling and recovery for failed replacements
  - Ensure surface remains accessible with new ID
  - _Requirements: 5.1, 5.2, 5.3_

- [ ]* 7.1 Write property test for surface accessibility after assignment
  - **Property 14: Surface accessibility after assignment**
  - **Validates: Requirements 5.2, 5.3**

- [ ]* 7.2 Write property test for ID replacement verification
  - **Property 15: ID replacement verification**
  - **Validates: Requirements 5.4**

- [x] 8. Enhance StateManager integration for auto-assigned surfaces
  - Update `StateManager::handle_surface_created` to work with ID assignment
  - Ensure auto-assigned IDs are properly tracked in surface state
  - Update surface queries to return auto-assigned IDs consistently
  - Ensure notifications include auto-assigned IDs
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [ ]* 8.1 Write property test for ID persistence during lifetime
  - **Property 16: ID persistence during lifetime**
  - **Validates: Requirements 6.1, 6.2**

- [ ]* 8.2 Write property test for query consistency
  - **Property 17: Query consistency**
  - **Validates: Requirements 6.3**

- [ ]* 8.3 Write property test for notification inclusion
  - **Property 18: Notification inclusion**
  - **Validates: Requirements 6.4**

- [ ]* 8.4 Write property test for operational equivalence
  - **Property 19: Operational equivalence**
  - **Validates: Requirements 6.5**

- [x] 9. Implement thread safety and concurrency support
  - Add proper synchronization for concurrent ID assignments
  - Implement atomic operations for registry updates
  - Add timeout mechanisms to prevent deadlocks
  - Ensure unique ID assignment under concurrent access
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [ ]* 9.1 Write property test for concurrent assignment uniqueness
  - **Property 21: Concurrent assignment uniqueness**
  - **Validates: Requirements 8.2, 8.5**

- [ ]* 9.2 Write property test for thread-safe operations
  - **Property 22: Thread-safe operations**
  - **Validates: Requirements 8.1, 8.3**

- [ ]* 9.3 Write property test for registry consistency under concurrency
  - **Property 23: Registry consistency under concurrency**
  - **Validates: Requirements 8.4**

- [x] 10. Add configuration and initialization support
  - Integrate ID assignment configuration with plugin initialization
  - Add command-line and configuration file support for ID assignment settings
  - Implement proper cleanup during plugin shutdown
  - Add configuration validation and error reporting
  - _Requirements: All requirements (system-wide configuration)_

- [x] 11. Implement advanced features and optimizations
  - Add support for sequential assignment priority over reuse
  - Implement ID space utilization monitoring
  - Add performance optimizations for high-frequency assignments
  - Include detailed statistics and health monitoring
  - _Requirements: 4.4, plus performance and monitoring_

- [ ]* 11.1 Write property test for sequential assignment priority
  - **Property 13: Sequential assignment priority**
  - **Validates: Requirements 4.4**

- [ ]* 11.2 Write property test for reuse during wraparound
  - **Property 12: Reuse during wraparound**
  - **Validates: Requirements 4.3**

- [x] 12. Add comprehensive error handling and recovery
  - Implement error recovery mechanisms for ID assignment failures
  - Add fallback strategies for edge cases (ID exhaustion)
  - Implement registry corruption detection and recovery
  - Add detailed error logging and diagnostics
  - _Requirements: 3.5, plus error handling requirements_

- [ ]* 12.1 Write unit tests for error handling scenarios
  - Test ID exhaustion handling
  - Test IVI API failure recovery
  - Test registry corruption detection
  - _Requirements: 3.5, 5.5_

- [x] 13. Final integration and validation
  - Integrate all components with existing controller architecture
  - Validate that existing functionality remains unaffected
  - Ensure proper plugin lifecycle management
  - Add integration tests for end-to-end scenarios
  - _Requirements: All requirements (integration validation)_

- [ ]* 13.1 Write integration tests for end-to-end scenarios
  - Test complete surface creation with ID assignment flow
  - Test concurrent surface creation scenarios
  - Test registry persistence and recovery
  - _Requirements: All requirements (end-to-end validation)_

- [x] 14. Final Checkpoint - Make sure all tests are passing
  - Ensure all tests pass, ask the user if questions arise.