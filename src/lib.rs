//! Weston IVI Controller - Main library entry point
//!
//! This library implements a Weston compositor plugin that provides programmatic
//! control over IVI (In-Vehicle Infotainment) surfaces through an RPC interface
//! over UNIX domain sockets.
//!
//! # Plugin Architecture
//!
//! The plugin consists of several key components:
//!
//! - **FFI Layer**: C-compatible interface for Weston plugin integration
//! - **IVI Wrapper**: Safe Rust abstractions over the C-based IVI layout API
//! - **State Manager**: Maintains internal representation of IVI surfaces
//! - **RPC Handler**: Processes RPC requests and generates responses
//! - **Transport Layer**: Handles communication with external clients (UNIX sockets)
//! - **Event System**: Listens to IVI surface lifecycle events
//!
//! # Plugin Lifecycle
//!
//! 1. **Initialization** (`wet_module_init`):
//!    - Retrieves IVI layout API from Weston
//!    - Creates state manager and synchronizes with IVI
//!    - Sets up RPC handler with UNIX socket transport
//!    - Registers event listeners for surface lifecycle
//!    - Starts the transport to accept client connections
//!
//! 2. **Operation**:
//!    - Accepts client connections via UNIX domain socket
//!    - Processes RPC requests to control IVI surfaces
//!    - Tracks surface lifecycle events (create, destroy, configure)
//!    - Maintains synchronized state with IVI compositor
//!
//! 3. **Cleanup** (`wet_module_destroy`):
//!    - Stops the transport and closes client connections
//!    - Unregisters event listeners
//!    - Cleans up all allocated resources
//!
//! # Usage
//!
//! To load this plugin in Weston, add it to the compositor configuration:
//!
//! ```text
//! [core]
//! modules=ivi-controller.so,libweston_ivi_controller.so
//! ```
//!
//! The plugin accepts the following command-line arguments:
//!
//! ## Transport Configuration
//! - `--socket-path=<path>`: Path to the UNIX domain socket (default: /tmp/weston-ivi-controller.sock)
//! - `--max-connections=<num>`: Maximum number of client connections (default: 10)
//!
//! ## ID Assignment Configuration
//! - `--id-start=<id>`: Starting ID for auto-assignment range (default: 0x10000000, supports hex with 0x prefix)
//! - `--id-max=<id>`: Maximum ID for auto-assignment range (default: 0xFFFFFFFE, supports hex with 0x prefix)
//! - `--id-invalid=<id>`: Invalid ID that triggers assignment (default: 0xFFFFFFFF, supports hex with 0x prefix)
//! - `--id-lock-timeout=<ms>`: Lock acquisition timeout in milliseconds (default: 5000)
//! - `--id-max-concurrent=<num>`: Maximum concurrent assignments (default: 10)
//! - `--id-assignment-timeout=<ms>`: Assignment operation timeout in milliseconds (default: 10000)
//!
//! ## Environment Variables
//! Configuration can also be set via environment variables (overridden by command-line args):
//! - `WESTON_IVI_SOCKET_PATH`: Socket path
//! - `WESTON_IVI_MAX_CONNECTIONS`: Maximum connections
//! - `WESTON_IVI_ID_START`: ID assignment start ID
//! - `WESTON_IVI_ID_MAX`: ID assignment max ID
//! - `WESTON_IVI_ID_INVALID`: Invalid ID value
//! - `WESTON_IVI_ID_LOCK_TIMEOUT`: Lock timeout in milliseconds
//! - `WESTON_IVI_ID_MAX_CONCURRENT`: Maximum concurrent assignments
//! - `WESTON_IVI_ID_ASSIGNMENT_TIMEOUT`: Assignment timeout in milliseconds
//!
//! # Safety
//!
//! This plugin uses unsafe code to interface with Weston's C API. All unsafe
//! operations are carefully isolated and documented. Panics are caught at FFI
//! boundaries to prevent unwinding into C code.

pub mod controller;
pub mod error;
pub mod ffi;
pub mod rpc;
pub mod transport;

// Re-export commonly used types
pub use error::{ControllerError, ControllerResult};
#[allow(unused)]
use jlogger_tracing::{jdebug, jerror, jinfo, jwarn, JloggerBuilder, LevelFilter, LogTimeFormat};

use std::env;
use std::ffi::CStr;
use std::panic;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use libc::{c_char, c_int, c_void};

use crate::controller::notifications::NotificationType;
use crate::ffi::bindings::ivi_layout_api::IviLayoutApi;
use controller::{
    EventContext, EventListeners, IdAssignmentConfig, IdAssignmentManager, StateManager,
};
use rpc::{NotificationBridge, RpcHandler};
#[cfg(not(feature = "enable-ipcon"))]
use transport::{unix_socket::UnixSocketConfig, UnixSocketTransport};

#[cfg(feature = "enable-ipcon")]
use transport::ipcon::IpconTransport;

/// Plugin configuration structure
///
/// This struct holds all configuration parameters for the plugin,
/// including socket configuration and ID assignment settings.
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// Path to the UNIX domain socket
    pub socket_path: PathBuf,

    /// Maximum number of client connections
    pub max_connections: usize,

    /// ID assignment configuration
    pub id_assignment: IdAssignmentConfig,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from("/tmp/weston-ivi-controller.sock"),
            max_connections: 10,
            id_assignment: IdAssignmentConfig::default(),
        }
    }
}

impl PluginConfig {
    /// Validate the plugin configuration
    ///
    /// # Returns
    /// * `Ok(())` - Configuration is valid
    /// * `Err(String)` - Configuration is invalid with error message
    pub fn validate(&self) -> Result<(), String> {
        // Validate socket path
        if let Some(parent) = self.socket_path.parent() {
            if !parent.exists() {
                return Err(format!(
                    "Socket directory does not exist: {}",
                    parent.display()
                ));
            }
        }

        // Validate max connections
        if self.max_connections == 0 {
            return Err("max_connections must be greater than 0".to_string());
        }

        if self.max_connections > 1000 {
            return Err("max_connections should not exceed 1000".to_string());
        }

        // Validate ID assignment configuration
        self.id_assignment
            .validate()
            .map_err(|e| format!("ID assignment configuration error: {}", e))?;

        Ok(())
    }
}

/// Plugin state that persists for the lifetime of the plugin
struct PluginState {
    // Kept alive to maintain shared ownership with RpcHandler and EventContext
    #[allow(dead_code)]
    state_manager: Arc<Mutex<StateManager>>,
    rpc_handler: Arc<RpcHandler>,
    // ID assignment manager for automatic surface ID assignment
    #[allow(dead_code)]
    id_assignment_manager: Arc<IdAssignmentManager>,
    // Kept alive to maintain event listener registrations with Weston
    #[allow(dead_code)]
    event_listeners: Option<EventListeners>,
    // Plugin configuration for cleanup reference
    #[allow(dead_code)]
    config: PluginConfig,
}

// Safety: PluginState is used in a single-threaded Weston plugin context.
// The raw pointers in EventListeners are managed by Weston's event loop
// and are only accessed from the main compositor thread.
unsafe impl Send for PluginState {}
unsafe impl Sync for PluginState {}

/// Global storage for plugin state and destroy listener
/// This is required for the standard Weston destroy listener pattern
static PLUGIN_STATE: Mutex<Option<Box<PluginState>>> = Mutex::new(None);

/// Compositor destroy callback - called by Weston when compositor shuts down
///
/// # Safety
/// This function is called by Weston's C code and must handle the cleanup safely
#[no_mangle]
pub unsafe extern "C" fn compositor_destroy_handler(
    listener: *mut ffi::wl_listener,
    _data: *mut c_void,
) {
    jinfo!("Compositor destroy handler called - cleaning up plugin");

    // Take ownership of the plugin state from the global
    let state = PLUGIN_STATE.lock().unwrap().take();

    if let Some(state) = state {
        // Request shutdown of ID assignment system
        jinfo!("Requesting shutdown of ID assignment system");
        state.id_assignment_manager.request_shutdown();

        // Wait for concurrent assignments to complete (with timeout)
        if let Err(e) = state
            .id_assignment_manager
            .wait_for_completion(std::time::Duration::from_millis(5000))
        {
            jwarn!(
                "Timeout waiting for ID assignment completion during shutdown: {:?}",
                e
            );
        } else {
            jinfo!("All ID assignment operations completed successfully");
        }

        // Get final statistics before cleanup
        if let Ok(stats) = state.id_assignment_manager.get_stats() {
            jinfo!(
                "ID assignment final statistics: total_assignments={}, wraparounds={}, conflicts_resolved={}, active_auto_assigned={}, timeout_errors={}, deadlock_errors={}, concurrency_limit_errors={}",
                stats.total_assignments,
                stats.wraparounds,
                stats.conflicts_resolved,
                stats.active_auto_assigned,
                stats.timeout_errors,
                stats.deadlock_errors,
                stats.concurrency_limit_errors
            );
        }

        // Stop the transport
        if let Err(e) = state.rpc_handler.stop_transport() {
            jerror!("Error stopping transport: {:?}", e);
        } else {
            jinfo!("Transport stopped");
        }

        // Event listeners, state manager, ID assignment manager, and RPC handler
        // will be cleaned up automatically when state is dropped
        drop(state);
        jinfo!("Event listeners unregistered");
        jinfo!("ID assignment manager cleaned up");
        jinfo!("Plugin state cleaned up");
    }

    // The listener is passed to us by the compositor.
    // We convert it back to a Box to deallocate the memory.
    if !listener.is_null() {
        libc::free(listener as *mut libc::c_void);
    }

    jinfo!("Weston IVI Controller plugin destroyed successfully");
}

/// Plugin initialization function called by Weston
///
/// # Arguments
/// * `compositor` - Pointer to the Weston compositor
/// * `argc` - Number of command-line arguments
/// * `argv` - Array of command-line argument strings
///
/// # Returns
/// * 0 on success
/// * -1 on failure
/// # Safety
/// This function is unsafe because it interacts with C FFI and dereferences raw pointers
#[no_mangle]
pub unsafe extern "C" fn wet_module_init(
    compositor: *mut c_void,
    argc: c_int,
    argv: *const *const c_char,
) -> c_int {
    // Initialize logging
    JloggerBuilder::new()
        .log_console(true)
        .log_file(Some(("/tmp/weston-ivi-controller.log", false)))
        .log_time(LogTimeFormat::TimeStamp)
        .max_level(LevelFilter::DEBUG)
        .build();

    // Catch panics to prevent unwinding across FFI boundary
    let result = panic::catch_unwind(|| {
        plugin_init_impl(compositor as *mut ffi::weston_compositor, argc, argv)
    });

    match result {
        Ok(Ok((state, compositor_ptr))) => {
            // Store plugin state in global
            *PLUGIN_STATE.lock().unwrap() = Some(Box::new(state));

            // Create and register destroy listener following standard Weston pattern
            // Allocate and zero-initialize to ensure wl_list link field is cleared
            let listener_ptr = unsafe {
                let ptr =
                    libc::malloc(std::mem::size_of::<ffi::wl_listener>()) as *mut ffi::wl_listener;
                if ptr.is_null() {
                    jerror!("Failed to allocate destroy listener");
                    PLUGIN_STATE.lock().unwrap().take();
                    return -1;
                }
                std::ptr::write_bytes(ptr, 0, 1);
                (*ptr).notify = Some(compositor_destroy_handler);
                ptr
            };

            let success = ffi::weston_compositor_add_destroy_listener_once(
                compositor_ptr,
                listener_ptr,
                compositor_destroy_handler,
            );

            if !success {
                jerror!("Failed to register compositor destroy listener");
                // Clean up on failure
                PLUGIN_STATE.lock().unwrap().take();
                libc::free(listener_ptr as *mut libc::c_void);
                return -1;
            }

            // The listener is now managed by the compositor and freed in the destroy handler.
            // We don't need to store it in a global variable.
            jinfo!("Compositor destroy listener registered");
            0 // Return 0 on success per standard Weston plugin convention
        }
        Ok(Err(e)) => {
            jerror!("Plugin initialization failed: {}", e);
            -1
        }
        Err(_) => {
            jerror!("Plugin initialization panicked");
            -1
        }
    }
}

/// Internal implementation of plugin initialization
///
/// # Safety
/// This function is unsafe because it:
/// - Dereferences raw pointers from C
/// - Calls FFI functions
/// - Assumes the IVI layout API pointer is valid
///
/// # Returns
/// Returns (PluginState, compositor pointer) on success
unsafe fn plugin_init_impl(
    compositor: *mut ffi::weston_compositor,
    argc: c_int,
    argv: *const *const c_char,
) -> Result<(PluginState, *mut ffi::weston_compositor), String> {
    jinfo!("Weston IVI Controller plugin initializing...");

    // Parse command-line arguments and environment variables
    let config = parse_plugin_config(argc, argv);

    // Validate configuration
    if let Err(e) = config.validate() {
        jerror!("Invalid plugin configuration: {}", e);
        return Err(format!("Invalid plugin configuration: {}", e));
    }

    #[cfg(not(feature = "enable-ipcon"))]
    {
        jinfo!("Using socket path: {:?}", config.socket_path);
        jinfo!("Max connections: {}", config.max_connections);
    }
    jinfo!(
        "ID assignment range: {:#x} - {:#x}",
        config.id_assignment.start_id,
        config.id_assignment.max_id
    );
    jinfo!(
        "ID assignment invalid ID: {:#x}",
        config.id_assignment.invalid_id
    );
    jinfo!(
        "ID assignment lock timeout: {}ms",
        config.id_assignment.lock_timeout_ms
    );
    jinfo!(
        "ID assignment max concurrent: {}",
        config.id_assignment.max_concurrent_assignments
    );
    jinfo!(
        "ID assignment operation timeout: {}ms",
        config.id_assignment.assignment_timeout_ms
    );

    // Retrieve the IVI layout API from Weston compositor
    let ivi_api = Arc::new(
        get_ivi_layout_api(compositor)
            .ok_or_else(|| "Failed to retrieve IVI layout API".to_string())?,
    );

    jinfo!("IVI layout API retrieved successfully");

    // Create state manager
    let mut state_manager = StateManager::new(ivi_api.clone());

    // Synchronize initial state with IVI
    state_manager.sync_with_ivi();

    let state_manager = Arc::new(Mutex::new(state_manager));

    jinfo!("State manager created");

    // Create RPC handler
    let rpc_handler = RpcHandler::new(Arc::clone(&state_manager));

    jinfo!("RPC handler created");

    #[cfg(feature = "enable-ipcon")]
    {
        let transport = Box::new(IpconTransport::new(None).map_err(|e| {
            jerror!("Failed to create IPCon transport: {:?}", e);
            format!("Failed to create IPCon transport: {:?}", e)
        })?);

        rpc_handler.register_transport(transport).map_err(|e| {
            jerror!("Failed to register transport: {:?}", e);
            format!("Failed to register transport: {:?}", e)
        })?;

        jinfo!("Ipcon Transport registered");
    }

    #[cfg(not(feature = "enable-ipcon"))]
    {
        // Create and register UNIX socket transport
        let transport_config = UnixSocketConfig {
            socket_path: config.socket_path.clone(),
            max_connections: config.max_connections,
        };

        let transport = Box::new(UnixSocketTransport::new(transport_config));
        rpc_handler.register_transport(transport).map_err(|e| {
            jerror!("Failed to register transport: {:?}", e);
            format!("Failed to register transport: {:?}", e)
        })?;

        jinfo!("UnixDomainSocket Transport registered");
    }

    // Create ID assignment manager with parsed configuration
    let id_assignment_manager = Arc::new(
        IdAssignmentManager::new(config.id_assignment.clone(), Arc::clone(&ivi_api))
            .map_err(|e| format!("Failed to create ID assignment manager: {}", e))?,
    );

    jinfo!("ID assignment manager created");

    // Register IVI event listeners
    let event_context = Arc::new(EventContext::new(
        Arc::clone(&state_manager),
        Arc::clone(&ivi_api),
        Arc::clone(&id_assignment_manager),
    ));

    let event_listeners = Arc::clone(&event_context)
        .register_listeners()
        .map_err(|e| {
            jerror!("Failed to register event listeners: {}", e);
            format!("Failed to register event listeners: {}", e)
        })?;

    jinfo!("Event listeners registered");

    // Register per-surface property listeners for existing surfaces
    unsafe {
        let existing_ids: Vec<u32> = {
            state_manager
                .lock()
                .unwrap()
                .get_all_surfaces()
                .iter()
                .map(|s| s.id)
                .collect()
        };
        for id in existing_ids {
            let _ = event_context.register_surface_property_listener_by_id(id);
        }
    }

    // Register per-layer property listeners for existing layers
    unsafe {
        let existing_layer_ids: Vec<u32> = {
            let layers = ivi_api.get_layers()?;
            layers.iter().map(|l| l.id()).collect()
        };
        for id in existing_layer_ids {
            let _ = event_context.register_layer_property_listener_by_id(id);
        }
    }

    // Bridge notifications -> subscriptions, and register callbacks
    {
        let bridge = Arc::new(NotificationBridge::new(rpc_handler.subscription_manager()));

        let notification_manager_arc = {
            // Get Arc<Mutex<NotificationManager>> from state manager
            let sm = state_manager.lock().unwrap();
            sm.notification_manager()
        };

        let mut nm = notification_manager_arc.lock().unwrap();

        // Helper to register a callback for a notification type
        let mut register = |nt: NotificationType| {
            let bridge_cloned = Arc::clone(&bridge);
            nm.register_callback(
                nt,
                Arc::new(move |n| {
                    bridge_cloned.handle_notification(n);
                }),
            );
        };

        // Register for all supported notification types
        register(NotificationType::GeometryChanged);
        register(NotificationType::FocusChanged);
        register(NotificationType::SurfaceCreated);
        register(NotificationType::SurfaceDestroyed);
        register(NotificationType::VisibilityChanged);
        register(NotificationType::OpacityChanged);
        register(NotificationType::OrientationChanged);
        register(NotificationType::ZOrderChanged);
        // Layer notifications
        register(NotificationType::LayerCreated);
        register(NotificationType::LayerDestroyed);
        register(NotificationType::LayerVisibilityChanged);
        register(NotificationType::LayerOpacityChanged);
    }

    // Start the transport
    rpc_handler.start_transport().map_err(|e| {
        jerror!("Failed to start transport: {:?}", e);
        format!("Failed to start transport: {:?}", e)
    })?;

    // Start background notification delivery to subscribed clients
    rpc_handler.start_notification_delivery();

    jinfo!("Transport started");
    jinfo!("Weston IVI Controller plugin initialized successfully");

    Ok((
        PluginState {
            state_manager,
            rpc_handler,
            id_assignment_manager,
            event_listeners: Some(event_listeners),
            config,
        },
        compositor, // Return compositor pointer for destroy listener registration
    ))
}

/// Parse plugin configuration from command-line arguments and environment variables
///
/// This function parses both command-line arguments and environment variables
/// to build the complete plugin configuration, including ID assignment settings.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers from C
///
/// # Arguments
/// * `argc` - Number of command-line arguments
/// * `argv` - Array of command-line argument strings
///
/// # Returns
/// A `PluginConfig` struct with parsed configuration values
unsafe fn parse_plugin_config(argc: c_int, argv: *const *const c_char) -> PluginConfig {
    let mut config = PluginConfig::default();

    // Parse command-line arguments
    if !argv.is_null() {
        for i in 0..argc as isize {
            let arg_ptr = *argv.offset(i);
            if arg_ptr.is_null() {
                continue;
            }

            let arg = CStr::from_ptr(arg_ptr).to_string_lossy();

            // Socket path configuration
            if arg == "--socket-path" && i + 1 < argc as isize {
                let path_ptr = *argv.offset(i + 1);
                if !path_ptr.is_null() {
                    let path = CStr::from_ptr(path_ptr).to_string_lossy();
                    config.socket_path = PathBuf::from(path.into_owned());
                }
            } else if arg.starts_with("--socket-path=") {
                let path = arg.strip_prefix("--socket-path=").unwrap();
                config.socket_path = PathBuf::from(path.to_string());
            }
            // Max connections configuration
            else if arg == "--max-connections" && i + 1 < argc as isize {
                let value_ptr = *argv.offset(i + 1);
                if !value_ptr.is_null() {
                    let value = CStr::from_ptr(value_ptr).to_string_lossy();
                    if let Ok(max_conn) = value.parse::<usize>() {
                        config.max_connections = max_conn;
                    }
                }
            } else if arg.starts_with("--max-connections=") {
                let value = arg.strip_prefix("--max-connections=").unwrap();
                if let Ok(max_conn) = value.parse::<usize>() {
                    config.max_connections = max_conn;
                }
            }
            // ID assignment start ID
            else if arg == "--id-start" && i + 1 < argc as isize {
                let value_ptr = *argv.offset(i + 1);
                if !value_ptr.is_null() {
                    let value = CStr::from_ptr(value_ptr).to_string_lossy();
                    if let Ok(start_id) = parse_hex_or_decimal(&value) {
                        config.id_assignment.start_id = start_id;
                    }
                }
            } else if arg.starts_with("--id-start=") {
                let value = arg.strip_prefix("--id-start=").unwrap();
                if let Ok(start_id) = parse_hex_or_decimal(value) {
                    config.id_assignment.start_id = start_id;
                }
            }
            // ID assignment max ID
            else if arg == "--id-max" && i + 1 < argc as isize {
                let value_ptr = *argv.offset(i + 1);
                if !value_ptr.is_null() {
                    let value = CStr::from_ptr(value_ptr).to_string_lossy();
                    if let Ok(max_id) = parse_hex_or_decimal(&value) {
                        config.id_assignment.max_id = max_id;
                    }
                }
            } else if arg.starts_with("--id-max=") {
                let value = arg.strip_prefix("--id-max=").unwrap();
                if let Ok(max_id) = parse_hex_or_decimal(value) {
                    config.id_assignment.max_id = max_id;
                }
            }
            // ID assignment invalid ID
            else if arg == "--id-invalid" && i + 1 < argc as isize {
                let value_ptr = *argv.offset(i + 1);
                if !value_ptr.is_null() {
                    let value = CStr::from_ptr(value_ptr).to_string_lossy();
                    if let Ok(invalid_id) = parse_hex_or_decimal(&value) {
                        config.id_assignment.invalid_id = invalid_id;
                    }
                }
            } else if arg.starts_with("--id-invalid=") {
                let value = arg.strip_prefix("--id-invalid=").unwrap();
                if let Ok(invalid_id) = parse_hex_or_decimal(value) {
                    config.id_assignment.invalid_id = invalid_id;
                }
            }
            // ID assignment lock timeout
            else if arg == "--id-lock-timeout" && i + 1 < argc as isize {
                let value_ptr = *argv.offset(i + 1);
                if !value_ptr.is_null() {
                    let value = CStr::from_ptr(value_ptr).to_string_lossy();
                    if let Ok(timeout) = value.parse::<u64>() {
                        config.id_assignment.lock_timeout_ms = timeout;
                    }
                }
            } else if arg.starts_with("--id-lock-timeout=") {
                let value = arg.strip_prefix("--id-lock-timeout=").unwrap();
                if let Ok(timeout) = value.parse::<u64>() {
                    config.id_assignment.lock_timeout_ms = timeout;
                }
            }
            // ID assignment max concurrent assignments
            else if arg == "--id-max-concurrent" && i + 1 < argc as isize {
                let value_ptr = *argv.offset(i + 1);
                if !value_ptr.is_null() {
                    let value = CStr::from_ptr(value_ptr).to_string_lossy();
                    if let Ok(max_concurrent) = value.parse::<usize>() {
                        config.id_assignment.max_concurrent_assignments = max_concurrent;
                    }
                }
            } else if arg.starts_with("--id-max-concurrent=") {
                let value = arg.strip_prefix("--id-max-concurrent=").unwrap();
                if let Ok(max_concurrent) = value.parse::<usize>() {
                    config.id_assignment.max_concurrent_assignments = max_concurrent;
                }
            }
            // ID assignment operation timeout
            else if arg == "--id-assignment-timeout" && i + 1 < argc as isize {
                let value_ptr = *argv.offset(i + 1);
                if !value_ptr.is_null() {
                    let value = CStr::from_ptr(value_ptr).to_string_lossy();
                    if let Ok(timeout) = value.parse::<u64>() {
                        config.id_assignment.assignment_timeout_ms = timeout;
                    }
                }
            } else if arg.starts_with("--id-assignment-timeout=") {
                let value = arg.strip_prefix("--id-assignment-timeout=").unwrap();
                if let Ok(timeout) = value.parse::<u64>() {
                    config.id_assignment.assignment_timeout_ms = timeout;
                }
            }
        }
    }

    // Parse environment variables (they override defaults but are overridden by command-line args)
    parse_environment_config(&mut config);

    config
}

/// Parse environment variables for plugin configuration
///
/// This function reads environment variables to configure the plugin.
/// Environment variables are prefixed with "WESTON_IVI_" to avoid conflicts.
///
/// # Arguments
/// * `config` - Mutable reference to the configuration to update
fn parse_environment_config(config: &mut PluginConfig) {
    // Socket path
    if let Ok(socket_path) = env::var("WESTON_IVI_SOCKET_PATH") {
        config.socket_path = PathBuf::from(socket_path);
    }

    // Max connections
    if let Ok(max_conn_str) = env::var("WESTON_IVI_MAX_CONNECTIONS") {
        if let Ok(max_conn) = max_conn_str.parse::<usize>() {
            config.max_connections = max_conn;
        }
    }

    // ID assignment start ID
    if let Ok(start_id_str) = env::var("WESTON_IVI_ID_START") {
        if let Ok(start_id) = parse_hex_or_decimal(&start_id_str) {
            config.id_assignment.start_id = start_id;
        }
    }

    // ID assignment max ID
    if let Ok(max_id_str) = env::var("WESTON_IVI_ID_MAX") {
        if let Ok(max_id) = parse_hex_or_decimal(&max_id_str) {
            config.id_assignment.max_id = max_id;
        }
    }

    // ID assignment invalid ID
    if let Ok(invalid_id_str) = env::var("WESTON_IVI_ID_INVALID") {
        if let Ok(invalid_id) = parse_hex_or_decimal(&invalid_id_str) {
            config.id_assignment.invalid_id = invalid_id;
        }
    }

    // ID assignment lock timeout
    if let Ok(timeout_str) = env::var("WESTON_IVI_ID_LOCK_TIMEOUT") {
        if let Ok(timeout) = timeout_str.parse::<u64>() {
            config.id_assignment.lock_timeout_ms = timeout;
        }
    }

    // ID assignment max concurrent assignments
    if let Ok(max_concurrent_str) = env::var("WESTON_IVI_ID_MAX_CONCURRENT") {
        if let Ok(max_concurrent) = max_concurrent_str.parse::<usize>() {
            config.id_assignment.max_concurrent_assignments = max_concurrent;
        }
    }

    // ID assignment operation timeout
    if let Ok(timeout_str) = env::var("WESTON_IVI_ID_ASSIGNMENT_TIMEOUT") {
        if let Ok(timeout) = timeout_str.parse::<u64>() {
            config.id_assignment.assignment_timeout_ms = timeout;
        }
    }
}

/// Parse a string as either hexadecimal (with 0x prefix) or decimal
///
/// # Arguments
/// * `value` - The string value to parse
///
/// # Returns
/// * `Ok(u32)` - Successfully parsed value
/// * `Err(std::num::ParseIntError)` - Parse error
fn parse_hex_or_decimal(value: &str) -> Result<u32, std::num::ParseIntError> {
    if value.starts_with("0x") || value.starts_with("0X") {
        u32::from_str_radix(&value[2..], 16)
    } else {
        value.parse::<u32>()
    }
}

/// Retrieve the IVI layout API from the Weston compositor
///
/// # Safety
/// This function is unsafe because it interacts with C FFI
///
/// # Arguments
/// * `compositor` - Pointer to the Weston compositor
///
/// # Returns
/// Pointer to the IVI layout interface, or null if not available
fn get_ivi_layout_api(compositor: *mut ffi::weston_compositor) -> Option<IviLayoutApi> {
    IviLayoutApi::new(compositor)
}

/// Plugin cleanup function - NO-OP in standard Weston pattern
///
/// In the standard Weston plugin pattern, cleanup is handled via the
/// compositor destroy listener (compositor_destroy_handler), not through
/// this function. This function is kept for compatibility but does nothing.
///
/// # Arguments
/// * `plugin_data` - Ignored (was opaque pointer in old pattern)
#[no_mangle]
pub extern "C" fn wet_module_destroy(_plugin_data: *mut c_void) {
    // No-op: Cleanup is handled by compositor_destroy_handler registered
    // via weston_compositor_add_destroy_listener_once in wet_module_init
    jdebug!("wet_module_destroy called (no-op - cleanup handled by destroy listener)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert_eq!(
            config.socket_path,
            PathBuf::from("/tmp/weston-ivi-controller.sock")
        );
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.id_assignment.start_id, 0x10000000);
        assert_eq!(config.id_assignment.max_id, 0xFFFFFFFE);
        assert_eq!(config.id_assignment.invalid_id, 0xFFFFFFFF);
    }

    #[test]
    fn test_plugin_config_validation_success() {
        let config = PluginConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_plugin_config_validation_invalid_max_connections() {
        let mut config = PluginConfig {
            max_connections: 10,
            ..Default::default()
        };
        config.max_connections = 0;
        assert!(config.validate().is_err());

        config.max_connections = 2000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_parse_hex_or_decimal() {
        assert_eq!(parse_hex_or_decimal("42").unwrap(), 42);
        assert_eq!(parse_hex_or_decimal("0x2A").unwrap(), 42);
        assert_eq!(parse_hex_or_decimal("0X2A").unwrap(), 42);
        assert_eq!(parse_hex_or_decimal("0xFFFFFFFF").unwrap(), 0xFFFFFFFF);
        assert!(parse_hex_or_decimal("invalid").is_err());
    }

    #[test]
    fn test_parse_plugin_config_defaults() {
        unsafe {
            let config = parse_plugin_config(0, std::ptr::null());
            assert_eq!(
                config.socket_path,
                PathBuf::from("/tmp/weston-ivi-controller.sock")
            );
            assert_eq!(config.max_connections, 10);
            assert_eq!(config.id_assignment.start_id, 0x10000000);
        }
    }

    #[test]
    fn test_parse_plugin_config_with_args() {
        unsafe {
            // Create test arguments
            let socket_path_arg = CString::new("--socket-path=/tmp/test.sock").unwrap();
            let max_conn_arg = CString::new("--max-connections=5").unwrap();
            let id_start_arg = CString::new("--id-start=0x20000000").unwrap();
            let id_max_arg = CString::new("--id-max=0x30000000").unwrap();

            let args = [
                socket_path_arg.as_ptr(),
                max_conn_arg.as_ptr(),
                id_start_arg.as_ptr(),
                id_max_arg.as_ptr(),
            ];

            let config = parse_plugin_config(args.len() as i32, args.as_ptr());

            assert_eq!(config.socket_path, PathBuf::from("/tmp/test.sock"));
            assert_eq!(config.max_connections, 5);
            assert_eq!(config.id_assignment.start_id, 0x20000000);
            assert_eq!(config.id_assignment.max_id, 0x30000000);
        }
    }

    #[test]
    fn test_parse_environment_config() {
        // Set test environment variables
        env::set_var("WESTON_IVI_SOCKET_PATH", "/tmp/env-test.sock");
        env::set_var("WESTON_IVI_MAX_CONNECTIONS", "15");
        env::set_var("WESTON_IVI_ID_START", "0x40000000");

        let mut config = PluginConfig::default();
        parse_environment_config(&mut config);

        assert_eq!(config.socket_path, PathBuf::from("/tmp/env-test.sock"));
        assert_eq!(config.max_connections, 15);
        assert_eq!(config.id_assignment.start_id, 0x40000000);

        // Clean up environment variables
        env::remove_var("WESTON_IVI_SOCKET_PATH");
        env::remove_var("WESTON_IVI_MAX_CONNECTIONS");
        env::remove_var("WESTON_IVI_ID_START");
    }
}
