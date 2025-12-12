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
//! - `--socket-path=<path>`: Path to the UNIX domain socket (default: /tmp/weston-ivi-controller.sock)
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

use std::ffi::CStr;
use std::panic;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use libc::{c_char, c_int, c_void};

use crate::controller::notifications::NotificationType;
use crate::ffi::bindings::ivi_layout_api::IviLayoutApi;
use controller::{EventContext, EventListeners, StateManager};
use rpc::{NotificationBridge, RpcHandler};
use transport::{unix_socket::UnixSocketConfig, UnixSocketTransport};

/// Plugin state that persists for the lifetime of the plugin
struct PluginState {
    // Kept alive to maintain shared ownership with RpcHandler and EventContext
    #[allow(dead_code)]
    state_manager: Arc<Mutex<StateManager>>,
    rpc_handler: Arc<RpcHandler>,
    // Kept alive to maintain event listener registrations with Weston
    #[allow(dead_code)]
    event_listeners: Option<EventListeners>,
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
        // Stop the transport
        if let Err(e) = state.rpc_handler.stop_transport() {
            jerror!("Error stopping transport: {:?}", e);
        } else {
            jinfo!("Transport stopped");
        }

        // Event listeners, state manager, and RPC handler will be cleaned up
        // automatically when state is dropped
        drop(state);
        jinfo!("Event listeners unregistered");
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
#[no_mangle]
pub extern "C" fn wet_module_init(
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
    let result = panic::catch_unwind(|| unsafe {
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

            unsafe {
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

    // Parse command-line arguments
    let socket_path = parse_socket_path(argc, argv)
        .unwrap_or_else(|| PathBuf::from("/tmp/weston-ivi-controller.sock"));

    jinfo!("Using socket path: {:?}", socket_path);

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

    // Create and register UNIX socket transport
    let transport_config = UnixSocketConfig {
        socket_path,
        max_connections: 10,
    };

    let transport = Box::new(UnixSocketTransport::new(transport_config));

    rpc_handler.register_transport(transport).map_err(|e| {
        jerror!("Failed to register transport: {:?}", e);
        format!("Failed to register transport: {:?}", e)
    })?;

    jinfo!("Transport registered");

    // Register IVI event listeners
    let event_context = Arc::new(EventContext::new(
        Arc::clone(&state_manager),
        Arc::clone(&ivi_api),
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
            event_listeners: Some(event_listeners),
        },
        compositor, // Return compositor pointer for destroy listener registration
    ))
}

/// Parse the socket path from command-line arguments
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers from C
unsafe fn parse_socket_path(argc: c_int, argv: *const *const c_char) -> Option<PathBuf> {
    if argv.is_null() {
        return None;
    }

    // Look for --socket-path argument
    for i in 0..argc as isize {
        let arg_ptr = *argv.offset(i);
        if arg_ptr.is_null() {
            continue;
        }

        let arg = CStr::from_ptr(arg_ptr).to_string_lossy();

        if arg == "--socket-path" && i + 1 < argc as isize {
            let path_ptr = *argv.offset(i + 1);
            if !path_ptr.is_null() {
                let path = CStr::from_ptr(path_ptr).to_string_lossy();
                return Some(PathBuf::from(path.into_owned()));
            }
        } else if arg.starts_with("--socket-path=") {
            let path = arg.strip_prefix("--socket-path=").unwrap();
            return Some(PathBuf::from(path.to_string()));
        }
    }

    None
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
