//! Integration tests for automatic surface ID assignment
//!
//! These tests verify end-to-end functionality of the ID assignment system
//! integrated with the controller architecture.

#[cfg(test)]
mod tests {
    use crate::controller::{
        IdAssignmentConfig, IdAssignmentManager, StateManager,
    };
    use crate::ffi::bindings::ivi_layout_api::IviLayoutApi;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    /// Create a mock IVI API for testing
    /// Note: This is a minimal mock that should not be used for actual IVI operations
    fn create_mock_ivi_api() -> Arc<IviLayoutApi> {
        // For integration tests, we create a mock that won't crash
        // but should not be used for actual IVI operations
        unsafe { 
            Arc::new(IviLayoutApi::from_raw(0x1000 as *const _).unwrap()) 
        }
    }

    /// Test: ID assignment manager creation and basic configuration
    ///
    /// This test verifies that the ID assignment manager can be created
    /// with proper configuration and integrates with the controller architecture.
    #[test]
    fn test_id_assignment_manager_creation() {
        let ivi_api = create_mock_ivi_api();
        let config = IdAssignmentConfig::default();
        
        // Create ID assignment manager
        let id_manager = IdAssignmentManager::new(config.clone(), Arc::clone(&ivi_api));
        assert!(id_manager.is_ok());
        
        let id_manager = id_manager.unwrap();
        
        // Verify configuration
        assert_eq!(id_manager.config().start_id, config.start_id);
        assert_eq!(id_manager.config().max_id, config.max_id);
        assert_eq!(id_manager.config().invalid_id, config.invalid_id);
        
        // Verify invalid ID detection
        assert!(id_manager.is_invalid_id(0xFFFFFFFF));
        assert!(!id_manager.is_invalid_id(0x10000000));
    }

    /// Test: State manager integration with ID assignment info
    ///
    /// This test verifies that the state manager can handle surfaces
    /// with auto-assignment information.
    #[test]
    fn test_state_manager_auto_assignment_integration() {
        let ivi_api = create_mock_ivi_api();
        let mut state_manager = StateManager::new(Arc::clone(&ivi_api));
        
        // Test the enhanced surface creation handler
        state_manager.handle_surface_created_with_assignment_info(
            0x10000000, // surface_id
            true,       // is_auto_assigned
            Some(0xFFFFFFFF) // original_id
        );
        
        // Verify the surface is tracked with auto-assignment info
        assert!(state_manager.has_surface(0x10000000));
        assert!(state_manager.is_surface_auto_assigned(0x10000000));
        assert_eq!(state_manager.get_surface_original_id(0x10000000), Some(0xFFFFFFFF));
        
        // Verify counts
        assert_eq!(state_manager.auto_assigned_surface_count(), 1);
        assert_eq!(state_manager.manual_assigned_surface_count(), 0);
        
        // Test manual surface creation
        state_manager.handle_surface_created_with_assignment_info(
            0x20000000, // surface_id
            false,      // is_auto_assigned
            None        // original_id
        );
        
        // Verify the manual surface is tracked correctly
        assert!(state_manager.has_surface(0x20000000));
        assert!(!state_manager.is_surface_auto_assigned(0x20000000));
        assert_eq!(state_manager.get_surface_original_id(0x20000000), None);
        
        // Verify updated counts
        assert_eq!(state_manager.auto_assigned_surface_count(), 1);
        assert_eq!(state_manager.manual_assigned_surface_count(), 1);
        assert_eq!(state_manager.surface_count(), 2);
    }

    /// Test: Configuration validation integration
    ///
    /// This test verifies that invalid configurations are properly rejected
    /// during manager creation.
    #[test]
    fn test_configuration_validation_integration() {
        let ivi_api = create_mock_ivi_api();
        
        // Test valid configuration
        let valid_config = IdAssignmentConfig::default();
        let result = IdAssignmentManager::new(valid_config, Arc::clone(&ivi_api));
        assert!(result.is_ok());
        
        // Test invalid range (start > max)
        let mut invalid_config = IdAssignmentConfig::default();
        invalid_config.start_id = 0xFFFFFFFE;
        invalid_config.max_id = 0x10000000;
        
        let result = IdAssignmentManager::new(invalid_config, Arc::clone(&ivi_api));
        assert!(result.is_err());
        
        // Test invalid ID within range
        let mut invalid_config = IdAssignmentConfig::default();
        invalid_config.invalid_id = 0x20000000; // Within the default range
        
        let result = IdAssignmentManager::new(invalid_config, Arc::clone(&ivi_api));
        assert!(result.is_err());
        
        // Test zero timeout
        let mut invalid_config = IdAssignmentConfig::default();
        invalid_config.lock_timeout_ms = 0;
        
        let result = IdAssignmentManager::new(invalid_config, Arc::clone(&ivi_api));
        assert!(result.is_err());
    }

    /// Test: Plugin lifecycle integration
    ///
    /// This test verifies that the ID assignment system integrates properly
    /// with plugin lifecycle management.
    #[test]
    fn test_plugin_lifecycle_integration() {
        let ivi_api = create_mock_ivi_api();
        let config = IdAssignmentConfig::default();
        
        let id_manager = Arc::new(
            IdAssignmentManager::new(config, Arc::clone(&ivi_api))
                .expect("Failed to create ID assignment manager")
        );
        
        // Verify initial state
        assert!(!id_manager.is_shutdown_requested());
        
        let stats = id_manager.get_stats().unwrap();
        assert_eq!(stats.total_assignments, 0);
        assert_eq!(stats.active_auto_assigned, 0);
        
        // Simulate plugin shutdown
        id_manager.request_shutdown();
        assert!(id_manager.is_shutdown_requested());
        
        // Verify graceful shutdown
        let result = id_manager.wait_for_completion(Duration::from_millis(100));
        assert!(result.is_ok());
    }

    /// Test: Error handling integration
    ///
    /// This test verifies that error handling works correctly across
    /// the integrated system.
    #[test]
    fn test_error_handling_integration() {
        let ivi_api = create_mock_ivi_api();
        
        // Test configuration error propagation
        let mut config = IdAssignmentConfig::default();
        config.start_id = 0xFFFFFFFF;
        config.max_id = 0x10000000;
        
        let result = IdAssignmentManager::new(config, Arc::clone(&ivi_api));
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        let error_string = format!("{}", error);
        assert!(error_string.contains("Invalid configuration"));
    }

    /// Test: Thread safety integration
    ///
    /// This test verifies that the integrated system maintains thread safety.
    #[test]
    fn test_thread_safety_integration() {
        let ivi_api = create_mock_ivi_api();
        let config = IdAssignmentConfig::default();
        
        let id_manager = Arc::new(
            IdAssignmentManager::new(config, Arc::clone(&ivi_api))
                .expect("Failed to create ID assignment manager")
        );
        
        let state_manager = Arc::new(Mutex::new(StateManager::new(Arc::clone(&ivi_api))));
        
        // Verify that both components can be shared across threads
        let id_manager_clone = Arc::clone(&id_manager);
        let state_manager_clone = Arc::clone(&state_manager);
        
        // This test verifies that the types are Send + Sync
        // In a real multi-threaded test, we would spawn threads here
        let _handle = std::thread::spawn(move || {
            let _config = id_manager_clone.config();
            let _state = state_manager_clone.lock().unwrap();
        });
    }

    /// Test: Statistics and monitoring integration
    ///
    /// This test verifies that statistics and monitoring work correctly
    /// across the integrated system.
    #[test]
    fn test_statistics_monitoring_integration() {
        let ivi_api = create_mock_ivi_api();
        let config = IdAssignmentConfig::default();
        
        let id_manager = Arc::new(
            IdAssignmentManager::new(config, Arc::clone(&ivi_api))
                .expect("Failed to create ID assignment manager")
        );
        
        // Get initial statistics
        let stats = id_manager.get_stats().unwrap();
        assert_eq!(stats.total_assignments, 0);
        assert_eq!(stats.active_auto_assigned, 0);
        assert_eq!(stats.timeout_errors, 0);
        assert_eq!(stats.deadlock_errors, 0);
        assert_eq!(stats.concurrency_limit_errors, 0);
        
        // Get health status
        let health = id_manager.get_health_status().unwrap();
        assert_eq!(health.utilization_percent, 0.0);
        assert_eq!(health.error_rate, 0.0);
        assert!(!health.is_warning);
        assert!(!health.is_critical);
        
        // Get utilization info
        let utilization = id_manager.get_utilization_info().unwrap();
        assert_eq!(utilization.active_ids, 0);
        assert_eq!(utilization.auto_assigned_ids, 0);
        assert_eq!(utilization.manual_assigned_ids, 0);
        assert_eq!(utilization.utilization_percent, 0.0);
        
        // Get performance metrics
        let metrics = id_manager.get_performance_metrics().unwrap();
        assert_eq!(metrics.sequential_assignments, 0);
        assert_eq!(metrics.reused_assignments, 0);
        assert_eq!(metrics.fragmentation_events, 0);
    }

    /// Test: Consistency validation integration
    ///
    /// This test verifies that consistency validation works across
    /// the integrated system.
    #[test]
    fn test_consistency_validation_integration() {
        let ivi_api = create_mock_ivi_api();
        let config = IdAssignmentConfig::default();
        
        let id_manager = Arc::new(
            IdAssignmentManager::new(config, Arc::clone(&ivi_api))
                .expect("Failed to create ID assignment manager")
        );
        
        // Validate initial consistency
        let result = id_manager.validate_consistency();
        assert!(result.is_ok());
        
        // Get active IDs (should be empty initially)
        let active_ids = id_manager.get_active_ids().unwrap();
        assert!(active_ids.is_empty());
        
        let auto_assigned_ids = id_manager.get_auto_assigned_ids().unwrap();
        assert!(auto_assigned_ids.is_empty());
    }

    /// Test: Component integration verification
    ///
    /// This test verifies that all components integrate correctly
    /// without requiring actual IVI operations.
    #[test]
    fn test_component_integration_verification() {
        let ivi_api = create_mock_ivi_api();
        let config = IdAssignmentConfig::default();
        
        // Create all integrated components
        let id_manager = Arc::new(
            IdAssignmentManager::new(config, Arc::clone(&ivi_api))
                .expect("Failed to create ID assignment manager")
        );
        
        let state_manager = Arc::new(Mutex::new(StateManager::new(Arc::clone(&ivi_api))));
        
        // Verify components can be created together
        assert!(!id_manager.is_shutdown_requested());
        assert_eq!(state_manager.lock().unwrap().surface_count(), 0);
        
        // Verify configuration consistency
        assert_eq!(id_manager.config().start_id, 0x10000000);
        assert_eq!(id_manager.config().max_id, 0xFFFFFFFE);
        assert_eq!(id_manager.config().invalid_id, 0xFFFFFFFF);
        
        // Verify initial state is consistent
        let stats = id_manager.get_stats().unwrap();
        let state_count = state_manager.lock().unwrap().surface_count();
        assert_eq!(stats.active_auto_assigned, 0);
        assert_eq!(state_count, 0);
    }
}