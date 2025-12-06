// Event handling for IVI surface lifecycle

use super::state::StateManager;
use crate::ffi::bindings::*;
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};

/// Event listener context that holds a reference to the StateManager
pub struct EventContext {
    state_manager: Arc<Mutex<StateManager>>,
    api: *const ivi_layout_interface,
}

// Safety: We ensure thread-safety through the Mutex on StateManager
unsafe impl Send for EventContext {}
unsafe impl Sync for EventContext {}

impl EventContext {
    /// Create a new event context
    pub fn new(state_manager: Arc<Mutex<StateManager>>, api: *const ivi_layout_interface) -> Self {
        Self { state_manager, api }
    }

    /// Register all surface lifecycle event listeners
    ///
    /// # Safety
    /// This function is unsafe because it registers C callbacks with raw pointers.
    /// The caller must ensure that:
    /// - The IVI API pointer is valid
    /// - The event context remains alive for the lifetime of the listeners
    pub unsafe fn register_listeners(
        self: Arc<Self>,
        api: *const ivi_layout_interface,
    ) -> Result<EventListeners, &'static str> {
        if api.is_null() {
            return Err("IVI API pointer is null");
        }

        // Allocate opaque wl_listener structures
        // Since wl_listener is opaque, we allocate raw memory for it
        // The actual structure will be managed by Wayland
        let create_listener = libc::malloc(std::mem::size_of::<wl_listener>()) as *mut wl_listener;
        let remove_listener = libc::malloc(std::mem::size_of::<wl_listener>()) as *mut wl_listener;
        let configure_listener =
            libc::malloc(std::mem::size_of::<wl_listener>()) as *mut wl_listener;

        if create_listener.is_null() || remove_listener.is_null() || configure_listener.is_null() {
            if !create_listener.is_null() {
                libc::free(create_listener as *mut c_void);
            }
            if !remove_listener.is_null() {
                libc::free(remove_listener as *mut c_void);
            }
            if !configure_listener.is_null() {
                libc::free(configure_listener as *mut c_void);
            }
            return Err("Failed to allocate listener memory");
        }

        // Store context references in global map
        let create_key = create_listener as usize;
        let remove_key = remove_listener as usize;
        let configure_key = configure_listener as usize;

        LISTENER_CONTEXTS
            .lock()
            .unwrap()
            .insert(create_key, Arc::clone(&self));
        LISTENER_CONTEXTS
            .lock()
            .unwrap()
            .insert(remove_key, Arc::clone(&self));
        LISTENER_CONTEXTS
            .lock()
            .unwrap()
            .insert(configure_key, Arc::clone(&self));

        // Register listeners with the IVI API
        if let Some(add_create_fn) = (*api).add_listener_create_surface {
            add_create_fn(create_listener);
        } else {
            return Err("add_listener_create_surface not available");
        }

        if let Some(add_remove_fn) = (*api).add_listener_remove_surface {
            add_remove_fn(remove_listener);
        } else {
            return Err("add_listener_remove_surface not available");
        }

        if let Some(add_configure_fn) = (*api).add_listener_configure_surface {
            add_configure_fn(configure_listener);
        } else {
            return Err("add_listener_configure_surface not available");
        }

        Ok(EventListeners {
            create_listener,
            remove_listener,
            configure_listener,
        })
    }
}

/// Holds the registered event listeners
pub struct EventListeners {
    create_listener: *mut wl_listener,
    remove_listener: *mut wl_listener,
    configure_listener: *mut wl_listener,
}

impl Drop for EventListeners {
    fn drop(&mut self) {
        unsafe {
            // Clean up listener contexts from global map
            LISTENER_CONTEXTS
                .lock()
                .unwrap()
                .remove(&(self.create_listener as usize));
            LISTENER_CONTEXTS
                .lock()
                .unwrap()
                .remove(&(self.remove_listener as usize));
            LISTENER_CONTEXTS
                .lock()
                .unwrap()
                .remove(&(self.configure_listener as usize));

            // Free the listener structures
            if !self.create_listener.is_null() {
                libc::free(self.create_listener as *mut c_void);
            }
            if !self.remove_listener.is_null() {
                libc::free(self.remove_listener as *mut c_void);
            }
            if !self.configure_listener.is_null() {
                libc::free(self.configure_listener as *mut c_void);
            }
        }
    }
}

// Global map to store listener contexts
// This is needed because C callbacks don't have a way to pass user data
lazy_static::lazy_static! {
    static ref LISTENER_CONTEXTS: Mutex<std::collections::HashMap<usize, Arc<EventContext>>> =
        Mutex::new(std::collections::HashMap::new());
}

/// C callback for surface creation events
#[no_mangle]
pub unsafe extern "C" fn surface_created_callback(listener: *mut wl_listener, data: *mut c_void) {
    if listener.is_null() || data.is_null() {
        return;
    }

    // Get the context from the global map
    let context = {
        let contexts = LISTENER_CONTEXTS.lock().unwrap();
        contexts.get(&(listener as usize)).cloned()
    };

    if let Some(context) = context {
        // data is a pointer to ivi_layout_surface
        let surface = data as *mut ivi_layout_surface;

        // Get the surface ID using the stored API pointer
        if let Some(get_id_fn) = (*context.api).get_id_of_surface {
            let surface_id = get_id_fn(surface);
            if let Ok(mut state_manager) = context.state_manager.lock() {
                state_manager.handle_surface_created(surface_id);
            }
        }
    }
}

/// C callback for surface removal events
#[no_mangle]
pub unsafe extern "C" fn surface_removed_callback(listener: *mut wl_listener, data: *mut c_void) {
    if listener.is_null() || data.is_null() {
        return;
    }

    let context = {
        let contexts = LISTENER_CONTEXTS.lock().unwrap();
        contexts.get(&(listener as usize)).cloned()
    };

    if let Some(context) = context {
        let surface = data as *mut ivi_layout_surface;

        if let Some(get_id_fn) = (*context.api).get_id_of_surface {
            let surface_id = get_id_fn(surface);
            if let Ok(mut state_manager) = context.state_manager.lock() {
                state_manager.handle_surface_destroyed(surface_id);
            }
        }
    }
}

/// C callback for surface configuration events
#[no_mangle]
pub unsafe extern "C" fn surface_configured_callback(
    listener: *mut wl_listener,
    data: *mut c_void,
) {
    if listener.is_null() || data.is_null() {
        return;
    }

    let context = {
        let contexts = LISTENER_CONTEXTS.lock().unwrap();
        contexts.get(&(listener as usize)).cloned()
    };

    if let Some(context) = context {
        let surface = data as *mut ivi_layout_surface;

        if let Some(get_id_fn) = (*context.api).get_id_of_surface {
            let surface_id = get_id_fn(surface);
            if let Ok(mut state_manager) = context.state_manager.lock() {
                state_manager.handle_surface_configured(surface_id);
            }
        }
    }
}
