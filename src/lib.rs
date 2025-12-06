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
use jlogger_tracing::{jdebug, jerror, jinfo, jwarn, JloggerBuilder, LevelFilter};

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::panic;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use controller::{EventContext, EventListeners, IviLayoutApi, StateManager};
use rpc::RpcHandler;
use transport::{unix_socket::UnixSocketConfig, UnixSocketTransport};

/// Plugin state that persists for the lifetime of the plugin
struct PluginState {
    // Kept alive to maintain shared ownership with RpcHandler and EventContext
    #[allow(dead_code)]
    state_manager: Arc<Mutex<StateManager>>,
    rpc_handler: Arc<RpcHandler>,
    event_listeners: Option<EventListeners>,
}

/// Plugin initialization function called by Weston
///
/// # Arguments
/// * `compositor` - Pointer to the Weston compositor (unused in this implementation)
/// * `argc` - Number of command-line arguments
/// * `argv` - Array of command-line argument strings
///
/// # Returns
/// * 0 on success
/// * -1 on failure
#[no_mangle]
pub extern "C" fn wet_module_init(
    _compositor: *mut c_void,
    argc: c_int,
    argv: *const *const c_char,
) -> c_int {
    // Catch panics to prevent unwinding across FFI boundary
    let result = panic::catch_unwind(|| unsafe { plugin_init_impl(argc, argv) });

    match result {
        Ok(Ok(state)) => {
            // Box the state and return it as an opaque pointer
            let state_ptr = Box::into_raw(Box::new(state));
            state_ptr as c_int
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
unsafe fn plugin_init_impl(argc: c_int, argv: *const *const c_char) -> Result<PluginState, String> {
    JloggerBuilder::new()
        .log_console(true)
        .log_file(Some(("/tmp/weston-ivi-controller.log", false)))
        .max_level(LevelFilter::DEBUG)
        .build();

    jinfo!("Weston IVI Controller plugin initializing...");

    // Parse command-line arguments
    let socket_path = parse_socket_path(argc, argv)
        .unwrap_or_else(|| PathBuf::from("/tmp/weston-ivi-controller.sock"));

    jinfo!("Using socket path: {:?}", socket_path);

    // Retrieve the IVI layout API from Weston
    // Note: In a real implementation, this would be obtained from the compositor
    // For now, we'll need to get it from the compositor's plugin API
    // This is a placeholder - the actual implementation depends on Weston's plugin interface
    let ivi_api_ptr = get_ivi_layout_api();

    if ivi_api_ptr.is_null() {
        jerror!("Failed to retrieve IVI layout API from compositor");
        return Err("Failed to retrieve IVI layout API from compositor".to_string());
    }

    let ivi_api = IviLayoutApi::from_raw(ivi_api_ptr)
        .ok_or_else(|| "Failed to create IVI layout API wrapper".to_string())?;

    let ivi_api = Arc::new(ivi_api);

    jinfo!("IVI layout API retrieved successfully");

    // Create state manager
    let mut state_manager = StateManager::new(Arc::clone(&ivi_api));

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
    let event_context = Arc::new(EventContext::new(Arc::clone(&state_manager), ivi_api_ptr));

    let event_listeners = event_context.register_listeners(ivi_api_ptr).map_err(|e| {
        jerror!("Failed to register event listeners: {}", e);
        format!("Failed to register event listeners: {}", e)
    })?;

    jinfo!("Event listeners registered");

    // Start the transport
    rpc_handler.start_transport().map_err(|e| {
        jerror!("Failed to start transport: {:?}", e);
        format!("Failed to start transport: {:?}", e)
    })?;

    jinfo!("Transport started");
    jinfo!("Weston IVI Controller plugin initialized successfully");

    Ok(PluginState {
        state_manager,
        rpc_handler,
        event_listeners: Some(event_listeners),
    })
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
/// Note: This is a placeholder implementation. In a real Weston plugin,
/// the IVI layout API would be obtained through the compositor's plugin
/// interface, typically through a function like:
/// `weston_plugin_api_get(compositor, IVI_LAYOUT_API_NAME, sizeof(ivi_layout_interface))`
unsafe fn get_ivi_layout_api() -> *const ffi::bindings::ivi_layout_interface {
    // This is a placeholder that returns null
    // In a real implementation, this would call into Weston's plugin API
    // to retrieve the IVI layout interface
    std::ptr::null()
}

/// Plugin cleanup function called by Weston
///
/// # Arguments
/// * `plugin_data` - Opaque pointer to the plugin state returned by wet_module_init
#[no_mangle]
pub extern "C" fn wet_module_destroy(plugin_data: *mut c_void) {
    // Catch panics to prevent unwinding across FFI boundary
    let result = panic::catch_unwind(|| unsafe { plugin_destroy_impl(plugin_data) });

    if let Err(_) = result {
        jerror!("Plugin cleanup panicked");
    }
}

/// Internal implementation of plugin cleanup
///
/// # Safety
/// This function is unsafe because it:
/// - Dereferences raw pointers from C
/// - Reconstructs a Box from a raw pointer
unsafe fn plugin_destroy_impl(plugin_data: *mut c_void) {
    if plugin_data.is_null() {
        return;
    }

    jinfo!("Weston IVI Controller plugin shutting down...");

    // Reconstruct the Box from the raw pointer
    let state = Box::from_raw(plugin_data as *mut PluginState);

    // Stop the transport
    if let Err(e) = state.rpc_handler.stop_transport() {
        jerror!("Error stopping transport: {:?}", e);
    } else {
        jinfo!("Transport stopped");
    }

    // Event listeners will be cleaned up automatically when dropped
    drop(state.event_listeners);
    jinfo!("Event listeners unregistered");

    // State manager and RPC handler will be cleaned up automatically
    jinfo!("Weston IVI Controller plugin shut down successfully");
}
