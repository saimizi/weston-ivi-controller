// Event handling for IVI surface lifecycle

use super::state::StateManager;
use crate::ffi::bindings::*;
#[allow(unused)]
use jlogger_tracing::{jdebug, jerror, jinfo};
use std::collections::HashMap;
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};

/// Event listener context that holds a reference to the StateManager
pub struct EventContext {
    state_manager: Arc<Mutex<StateManager>>,
    ivi_api: Arc<super::ivi_wrapper::IviLayoutApi>,
    surface_prop_listeners: Mutex<HashMap<u32, *mut wl_listener>>, // per-surface property listeners
    layer_prop_listeners: Mutex<HashMap<u32, *mut wl_listener>>,   // per-layer property listeners
}

// Safety: We ensure thread-safety through the Mutex on StateManager
unsafe impl Send for EventContext {}
unsafe impl Sync for EventContext {}

impl EventContext {
    /// Create a new event context
    pub fn new(
        state_manager: Arc<Mutex<StateManager>>,
        ivi_api: Arc<super::ivi_wrapper::IviLayoutApi>,
    ) -> Self {
        Self {
            state_manager,
            ivi_api,
            surface_prop_listeners: Mutex::new(HashMap::new()),
            layer_prop_listeners: Mutex::new(HashMap::new()),
        }
    }

    /// Helper function to cleanup allocated listeners
    unsafe fn cleanup_listeners(listeners: &[*mut wl_listener]) {
        for &listener in listeners {
            if !listener.is_null() {
                libc::free(listener as *mut c_void);
            }
        }
    }

    /// Helper function to remove listeners from global context map
    fn cleanup_listener_contexts(listeners: &[*mut wl_listener]) {
        let mut contexts = LISTENER_CONTEXTS.lock().unwrap();
        for &listener in listeners {
            if !listener.is_null() {
                contexts.remove(&(listener as usize));
            }
        }
    }

    /// Register all surface lifecycle event listeners
    ///
    /// # Safety
    /// This function is unsafe because it registers C callbacks with raw pointers.
    /// The caller must ensure that:
    /// - The IVI API pointer is valid
    /// - The event context remains alive for the lifetime of the listeners
    pub unsafe fn register_listeners(self: Arc<Self>) -> Result<EventListeners, &'static str> {
        if self.ivi_api.api.is_null() {
            return Err("IVI API pointer is null");
        }

        // Allocate opaque wl_listener structures
        // Since wl_listener is opaque, we allocate raw memory for it
        // The actual structure will be managed by Wayland
        let create_listener = libc::malloc(std::mem::size_of::<wl_listener>()) as *mut wl_listener;
        let remove_listener = libc::malloc(std::mem::size_of::<wl_listener>()) as *mut wl_listener;
        let configure_listener =
            libc::malloc(std::mem::size_of::<wl_listener>()) as *mut wl_listener;
        let layer_create_listener =
            libc::malloc(std::mem::size_of::<wl_listener>()) as *mut wl_listener;
        let layer_remove_listener =
            libc::malloc(std::mem::size_of::<wl_listener>()) as *mut wl_listener;

        // Group all listeners for easier cleanup
        let all_listeners = [
            create_listener,
            remove_listener,
            configure_listener,
            layer_create_listener,
            layer_remove_listener,
        ];

        // Check if any allocation failed
        if all_listeners.iter().any(|&l| l.is_null()) {
            Self::cleanup_listeners(&all_listeners);
            return Err("Failed to allocate listener memory");
        }

        // CRITICAL: Zero-initialize all listener structures
        // wl_listener contains a wl_list link field (prev/next pointers)
        // that MUST be initialized to avoid crashes when Weston adds them to lists
        for &listener in &all_listeners {
            std::ptr::write_bytes(listener, 0, 1);
        }

        // Store context references in global map - single lock acquisition
        {
            let mut contexts = LISTENER_CONTEXTS.lock().unwrap();
            contexts.insert(create_listener as usize, Arc::clone(&self));
            contexts.insert(remove_listener as usize, Arc::clone(&self));
            contexts.insert(configure_listener as usize, Arc::clone(&self));
            contexts.insert(layer_create_listener as usize, Arc::clone(&self));
            contexts.insert(layer_remove_listener as usize, Arc::clone(&self));
        }

        // Set notify callbacks on listeners
        (*create_listener).notify = Some(surface_created_callback);
        (*remove_listener).notify = Some(surface_removed_callback);
        (*configure_listener).notify = Some(surface_configured_callback);

        // Layer callbacks (optional)
        (*layer_create_listener).notify = Some(layer_created_callback);
        (*layer_remove_listener).notify = Some(layer_removed_callback);

        // Register listeners with the IVI API
        if let Some(add_create_fn) = (*self.ivi_api.api).add_listener_create_surface {
            jdebug!("Registering create surface listener");
            add_create_fn(create_listener);
            jdebug!("Registered create surface listener");
        } else {
            // Clean up on error
            Self::cleanup_listeners(&all_listeners);
            Self::cleanup_listener_contexts(&all_listeners);
            return Err("add_listener_create_surface not available");
        }

        if let Some(add_remove_fn) = (*self.ivi_api.api).add_listener_remove_surface {
            add_remove_fn(remove_listener);
        } else {
            // Clean up on error
            Self::cleanup_listeners(&all_listeners);
            Self::cleanup_listener_contexts(&all_listeners);
            return Err("add_listener_remove_surface not available");
        }

        if let Some(add_configure_fn) = (*self.ivi_api.api).add_listener_configure_surface {
            add_configure_fn(configure_listener);
        } else {
            // Clean up on error
            Self::cleanup_listeners(&all_listeners);
            Self::cleanup_listener_contexts(&all_listeners);
            return Err("add_listener_configure_surface not available");
        }

        // Register layer listeners if available on this Weston build
        if let Some(add_create_layer_fn) = (*self.ivi_api.api).add_listener_create_layer {
            add_create_layer_fn(layer_create_listener);
        }
        if let Some(add_remove_layer_fn) = (*self.ivi_api.api).add_listener_remove_layer {
            add_remove_layer_fn(layer_remove_listener);
        }

        Ok(EventListeners {
            ctx: Arc::clone(&self),
            create_listener,
            remove_listener,
            configure_listener,
            layer_create_listener,
            layer_remove_listener,
        })
    }

    /// Register a per-surface property change listener by surface id
    pub unsafe fn register_surface_property_listener_by_id(
        &self,
        surface_id: u32,
    ) -> Result<(), &'static str> {
        if self.ivi_api.api.is_null() {
            return Err("IVI API pointer is null");
        }
        let get_surface_fn = (*self.ivi_api.api)
            .get_surface_from_id
            .ok_or("get_surface_from_id not available")?;
        let surf = get_surface_fn(surface_id);
        if surf.is_null() {
            return Err("Surface not found for property listener");
        }

        // Allocate wl_listener and set notify
        let listener = libc::malloc(std::mem::size_of::<wl_listener>()) as *mut wl_listener;
        if listener.is_null() {
            return Err("Failed to allocate surface property listener");
        }
        // Zero-initialize to clear wl_list link field
        std::ptr::write_bytes(listener, 0, 1);
        (*listener).notify = Some(surface_property_changed_callback);

        // Store context for callback
        LISTENER_CONTEXTS.lock().unwrap().insert(
            listener as usize,
            Arc::new(Self {
                state_manager: Arc::clone(&self.state_manager),
                ivi_api: Arc::clone(&self.ivi_api),
                surface_prop_listeners: Mutex::new(HashMap::new()),
                layer_prop_listeners: Mutex::new(HashMap::new()),
            }),
        );

        // Add to per-surface map for cleanup
        self.surface_prop_listeners
            .lock()
            .unwrap()
            .insert(surface_id, listener);

        // Register with IVI API
        let add_fn = (*self.ivi_api.api)
            .surface_add_listener
            .ok_or("surface_add_listener not available")?;
        add_fn(surf, listener);

        Ok(())
    }

    /// Remove and free a per-surface property listener by surface id
    pub unsafe fn remove_surface_property_listener(&self, surface_id: u32) {
        if let Some(listener) = self
            .surface_prop_listeners
            .lock()
            .unwrap()
            .remove(&surface_id)
        {
            LISTENER_CONTEXTS
                .lock()
                .unwrap()
                .remove(&(listener as usize));
            libc::free(listener as *mut c_void);
        }
    }

    /// Clear all per-surface property listeners
    pub unsafe fn clear_property_listeners(&self) {
        let mut map = self.surface_prop_listeners.lock().unwrap();
        for (_, listener) in map.drain() {
            LISTENER_CONTEXTS
                .lock()
                .unwrap()
                .remove(&(listener as usize));
            libc::free(listener as *mut c_void);
        }
    }

    /// Register a per-layer property change listener by layer id
    pub unsafe fn register_layer_property_listener_by_id(
        &self,
        layer_id: u32,
    ) -> Result<(), &'static str> {
        if self.ivi_api.api.is_null() {
            return Err("IVI API pointer is null");
        }
        let get_layer_fn = (*self.ivi_api.api)
            .get_layer_from_id
            .ok_or("get_layer_from_id not available")?;
        let layer = get_layer_fn(layer_id);
        if layer.is_null() {
            return Err("Layer not found for property listener");
        }

        // Allocate wl_listener and set notify
        let listener = libc::malloc(std::mem::size_of::<wl_listener>()) as *mut wl_listener;
        if listener.is_null() {
            return Err("Failed to allocate layer property listener");
        }
        // Zero-initialize to clear wl_list link field
        std::ptr::write_bytes(listener, 0, 1);
        (*listener).notify = Some(layer_property_changed_callback);

        // Store context for callback
        LISTENER_CONTEXTS.lock().unwrap().insert(
            listener as usize,
            Arc::new(Self {
                state_manager: Arc::clone(&self.state_manager),
                ivi_api: Arc::clone(&self.ivi_api),
                surface_prop_listeners: Mutex::new(HashMap::new()),
                layer_prop_listeners: Mutex::new(HashMap::new()),
            }),
        );

        // Add to per-layer map for cleanup
        self.layer_prop_listeners
            .lock()
            .unwrap()
            .insert(layer_id, listener);

        // Register with IVI API
        let add_fn = (*self.ivi_api.api)
            .layer_add_listener
            .ok_or("layer_add_listener not available")?;
        add_fn(layer, listener);

        Ok(())
    }

    /// Remove and free a per-layer property listener by layer id
    pub unsafe fn remove_layer_property_listener(&self, layer_id: u32) {
        if let Some(listener) = self.layer_prop_listeners.lock().unwrap().remove(&layer_id) {
            LISTENER_CONTEXTS
                .lock()
                .unwrap()
                .remove(&(listener as usize));
            libc::free(listener as *mut c_void);
        }
    }

    /// Clear all per-layer property listeners
    pub unsafe fn clear_layer_property_listeners(&self) {
        let mut map = self.layer_prop_listeners.lock().unwrap();
        for (_, listener) in map.drain() {
            LISTENER_CONTEXTS
                .lock()
                .unwrap()
                .remove(&(listener as usize));
            libc::free(listener as *mut c_void);
        }
    }
}

/// Holds the registered event listeners
pub struct EventListeners {
    ctx: Arc<EventContext>,
    create_listener: *mut wl_listener,
    remove_listener: *mut wl_listener,
    configure_listener: *mut wl_listener,
    // Layer listeners (optional based on API availability)
    layer_create_listener: *mut wl_listener,
    layer_remove_listener: *mut wl_listener,
}

impl Drop for EventListeners {
    fn drop(&mut self) {
        unsafe {
            // Clear all per-surface listeners first
            self.ctx.clear_property_listeners();
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
            if !self.layer_create_listener.is_null() {
                LISTENER_CONTEXTS
                    .lock()
                    .unwrap()
                    .remove(&(self.layer_create_listener as usize));
            }
            if !self.layer_remove_listener.is_null() {
                LISTENER_CONTEXTS
                    .lock()
                    .unwrap()
                    .remove(&(self.layer_remove_listener as usize));
            }

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
            if !self.layer_create_listener.is_null() {
                libc::free(self.layer_create_listener as *mut c_void);
            }
            if !self.layer_remove_listener.is_null() {
                libc::free(self.layer_remove_listener as *mut c_void);
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
        if let Some(get_id_fn) = (*context.ivi_api.api).get_id_of_surface {
            let surface_id = get_id_fn(surface);
            if let Ok(mut state_manager) = context.state_manager.lock() {
                state_manager.handle_surface_created(surface_id);
            }
            // Register per-surface property listener for this surface
            context
                .register_surface_property_listener_by_id(surface_id)
                .ok();
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

        if let Some(get_id_fn) = (*context.ivi_api.api).get_id_of_surface {
            let surface_id = get_id_fn(surface);
            if let Ok(mut state_manager) = context.state_manager.lock() {
                state_manager.handle_surface_destroyed(surface_id);
            }
            // Remove and free property listener for this surface
            context.remove_surface_property_listener(surface_id);
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

        if let Some(get_id_fn) = (*context.ivi_api.api).get_id_of_surface {
            let surface_id = get_id_fn(surface);
            if let Ok(mut state_manager) = context.state_manager.lock() {
                state_manager.handle_surface_configured(surface_id);
            }
        }
    }
}

/// C callback for per-surface property change events
#[no_mangle]
pub unsafe extern "C" fn surface_property_changed_callback(
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

        if let Some(get_id_fn) = (*context.ivi_api.api).get_id_of_surface {
            let surface_id = get_id_fn(surface);
            if let Ok(mut state_manager) = context.state_manager.lock() {
                // Recompute and emit property change notifications
                state_manager.handle_surface_configured(surface_id);
            }
        }
    }
}

/// C callback for layer creation events
#[no_mangle]
pub unsafe extern "C" fn layer_created_callback(listener: *mut wl_listener, data: *mut c_void) {
    if listener.is_null() || data.is_null() {
        return;
    }

    let context = {
        let contexts = LISTENER_CONTEXTS.lock().unwrap();
        contexts.get(&(listener as usize)).cloned()
    };

    if let Some(context) = context {
        // data is a pointer to ivi_layout_layer
        let layer = data as *mut ivi_layout_layer;

        if let Some(get_id_fn) = (*context.ivi_api.api).get_id_of_layer {
            let layer_id = get_id_fn(layer);
            if let Ok(mut state_manager) = context.state_manager.lock() {
                state_manager.handle_layer_created(layer_id);
            }
            // Register per-layer property listener for this layer
            context
                .register_layer_property_listener_by_id(layer_id)
                .ok();
        }
    }
}

/// C callback for layer removal events
#[no_mangle]
pub unsafe extern "C" fn layer_removed_callback(listener: *mut wl_listener, data: *mut c_void) {
    if listener.is_null() || data.is_null() {
        return;
    }

    let context = {
        let contexts = LISTENER_CONTEXTS.lock().unwrap();
        contexts.get(&(listener as usize)).cloned()
    };

    if let Some(context) = context {
        let layer = data as *mut ivi_layout_layer;

        if let Some(get_id_fn) = (*context.ivi_api.api).get_id_of_layer {
            let layer_id = get_id_fn(layer);
            if let Ok(mut state_manager) = context.state_manager.lock() {
                state_manager.handle_layer_destroyed(layer_id);
            }
            // Remove and free per-layer listener
            context.remove_layer_property_listener(layer_id);
        }
    }
}

/// C callback for per-layer property change events
#[no_mangle]
pub unsafe extern "C" fn layer_property_changed_callback(
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
        let layer = data as *mut ivi_layout_layer;

        if let Some(get_id_fn) = (*context.ivi_api.api).get_id_of_layer {
            let layer_id = get_id_fn(layer);
            if let Ok(mut state_manager) = context.state_manager.lock() {
                state_manager.handle_layer_configured(layer_id);
            }
        }
    }
}
