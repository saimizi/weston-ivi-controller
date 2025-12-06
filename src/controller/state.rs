// State management for IVI surfaces

use super::ivi_wrapper::{IviLayoutApi, Orientation as IviOrientation};
#[allow(unused)]
use jlogger_tracing::{jdebug, jerror, jinfo, jwarn, JloggerBuilder, LevelFilter};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Represents the state of an IVI surface
#[derive(Debug, Clone)]
pub struct SurfaceState {
    pub id: u32,
    pub position: (i32, i32),
    pub size: (i32, i32),
    pub visibility: bool,
    pub opacity: f32,
    pub orientation: Orientation,
    pub z_order: i32,
}

/// Surface orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Orientation {
    Normal,    // 0 degrees
    Rotate90,  // 90 degrees
    Rotate180, // 180 degrees
    Rotate270, // 270 degrees
}

impl From<IviOrientation> for Orientation {
    fn from(ivi_orientation: IviOrientation) -> Self {
        match ivi_orientation {
            IviOrientation::Normal => Orientation::Normal,
            IviOrientation::Rotate90 => Orientation::Rotate90,
            IviOrientation::Rotate180 => Orientation::Rotate180,
            IviOrientation::Rotate270 => Orientation::Rotate270,
        }
    }
}

impl From<Orientation> for IviOrientation {
    fn from(orientation: Orientation) -> Self {
        match orientation {
            Orientation::Normal => IviOrientation::Normal,
            Orientation::Rotate90 => IviOrientation::Rotate90,
            Orientation::Rotate180 => IviOrientation::Rotate180,
            Orientation::Rotate270 => IviOrientation::Rotate270,
        }
    }
}

/// Manages the state of all IVI surfaces
pub struct StateManager {
    surfaces: Arc<Mutex<HashMap<u32, SurfaceState>>>,
    ivi_api: Arc<IviLayoutApi>,
    notification_manager: Arc<Mutex<super::notifications::NotificationManager>>,
    focused_surface: Arc<Mutex<Option<u32>>>,
}

impl StateManager {
    /// Create a new StateManager
    pub fn new(ivi_api: Arc<IviLayoutApi>) -> Self {
        Self {
            surfaces: Arc::new(Mutex::new(HashMap::new())),
            ivi_api,
            notification_manager: Arc::new(Mutex::new(
                super::notifications::NotificationManager::new(),
            )),
            focused_surface: Arc::new(Mutex::new(None)),
        }
    }

    /// Get a reference to the notification manager
    pub fn notification_manager(&self) -> Arc<Mutex<super::notifications::NotificationManager>> {
        Arc::clone(&self.notification_manager)
    }

    /// Get the currently focused surface ID
    pub fn get_focused_surface(&self) -> Option<u32> {
        *self.focused_surface.lock().unwrap()
    }

    /// Set the focused surface and emit focus change notifications
    pub fn set_focused_surface(&mut self, new_focused: Option<u32>) {
        let old_focused = {
            let mut focused = self.focused_surface.lock().unwrap();
            let old = *focused;
            *focused = new_focused;
            old
        };

        // Only emit notification if focus actually changed
        if old_focused != new_focused {
            let notification_manager = self.notification_manager.lock().unwrap();
            notification_manager.emit_focus_change(old_focused, new_focused);
        }
    }

    /// Add a surface to the state manager
    /// This is called when a new surface is created
    pub fn add_surface(&mut self, id: u32, state: SurfaceState) {
        jinfo!("Adding surface {} to state manager", id);
        let mut surfaces = self.surfaces.lock().unwrap();
        surfaces.insert(id, state);
    }

    /// Remove a surface from the state manager
    /// This is called when a surface is destroyed
    pub fn remove_surface(&mut self, id: u32) -> Option<SurfaceState> {
        jinfo!("Removing surface {} from state manager", id);
        let mut surfaces = self.surfaces.lock().unwrap();
        surfaces.remove(&id)
    }

    /// Update surface state
    /// This is called when surface properties change
    pub fn update_surface(&mut self, id: u32, state: SurfaceState) {
        jdebug!("Updating surface {} state", id);
        let mut surfaces = self.surfaces.lock().unwrap();
        surfaces.insert(id, state);
    }

    /// Get surface state by ID
    pub fn get_surface(&self, id: u32) -> Option<SurfaceState> {
        let surfaces = self.surfaces.lock().unwrap();
        surfaces.get(&id).cloned()
    }

    /// Get all surfaces
    pub fn get_all_surfaces(&self) -> Vec<SurfaceState> {
        let surfaces = self.surfaces.lock().unwrap();
        surfaces.values().cloned().collect()
    }

    /// Get the number of tracked surfaces
    pub fn surface_count(&self) -> usize {
        let surfaces = self.surfaces.lock().unwrap();
        surfaces.len()
    }

    /// Check if a surface exists
    pub fn has_surface(&self, id: u32) -> bool {
        let surfaces = self.surfaces.lock().unwrap();
        surfaces.contains_key(&id)
    }

    /// Synchronize state with the IVI API
    /// This queries the IVI API for all surfaces and updates internal state
    pub fn sync_with_ivi(&mut self) {
        let ivi_surfaces = self.ivi_api.get_surfaces();
        let mut surfaces = self.surfaces.lock().unwrap();

        // Clear existing state
        surfaces.clear();

        // Populate with current IVI surfaces
        for surface in ivi_surfaces {
            let id = surface.get_id();
            let position = surface.get_position();
            let size = surface.get_size();
            let visibility = surface.get_visibility();
            let opacity = surface.get_opacity();
            let orientation = surface.get_orientation().into();

            let state = SurfaceState {
                id,
                position,
                size,
                visibility,
                opacity,
                orientation,
                z_order: 0, // Z-order is managed at layer level
            };

            surfaces.insert(id, state);
        }
    }

    /// Get a reference to the IVI API
    pub fn ivi_api(&self) -> &Arc<IviLayoutApi> {
        &self.ivi_api
    }

    /// Register event listeners for surface lifecycle events
    /// This should be called during plugin initialization
    pub fn register_listeners(&mut self) {
        // Note: The actual listener registration requires access to the raw IVI API
        // and callback functions. This will be implemented in the plugin initialization
        // code where we have access to the C FFI layer.
        //
        // The listeners will call:
        // - handle_surface_created() when a surface is created
        // - handle_surface_destroyed() when a surface is destroyed
        // - handle_surface_configured() when a surface is configured
    }

    /// Handle surface creation event
    /// This is called by the event listener when a new surface is created
    pub fn handle_surface_created(&mut self, surface_id: u32) {
        // Query the IVI API for the new surface
        if let Some(surface) = self.ivi_api.get_surface_from_id(surface_id) {
            let position = surface.get_position();
            let size = surface.get_size();
            let visibility = surface.get_visibility();
            let opacity = surface.get_opacity();
            let orientation = surface.get_orientation().into();

            let state = SurfaceState {
                id: surface_id,
                position,
                size,
                visibility,
                opacity,
                orientation,
                z_order: 0,
            };

            self.add_surface(surface_id, state);

            // Emit surface created notification
            let notification_manager = self.notification_manager.lock().unwrap();
            notification_manager.emit_surface_created(surface_id);
        }
    }

    /// Handle surface destruction event
    /// This is called by the event listener when a surface is destroyed
    pub fn handle_surface_destroyed(&mut self, surface_id: u32) {
        self.remove_surface(surface_id);

        // Emit surface destroyed notification
        {
            let notification_manager = self.notification_manager.lock().unwrap();
            notification_manager.emit_surface_destroyed(surface_id);
        }

        // If this was the focused surface, clear focus
        let was_focused = {
            let focused = self.focused_surface.lock().unwrap();
            *focused == Some(surface_id)
        };

        if was_focused {
            self.set_focused_surface(None);
        }
    }

    /// Handle surface configuration event
    /// This is called by the event listener when a surface is configured
    pub fn handle_surface_configured(&mut self, surface_id: u32) {
        // Get old state for comparison
        let old_state = self.get_surface(surface_id);

        // Query the IVI API for updated surface properties
        if let Some(surface) = self.ivi_api.get_surface_from_id(surface_id) {
            let position = surface.get_position();
            let size = surface.get_size();
            let visibility = surface.get_visibility();
            let opacity = surface.get_opacity();
            let orientation = surface.get_orientation().into();

            // Get existing z_order or default to 0
            let z_order = old_state.as_ref().map(|s| s.z_order).unwrap_or(0);

            let new_state = SurfaceState {
                id: surface_id,
                position,
                size,
                visibility,
                opacity,
                orientation,
                z_order,
            };

            // Check if geometry changed and emit notification
            if let Some(old) = old_state {
                if old.position != new_state.position || old.size != new_state.size {
                    let notification_manager = self.notification_manager.lock().unwrap();
                    notification_manager.emit_geometry_change(
                        surface_id,
                        old.position,
                        new_state.position,
                        old.size,
                        new_state.size,
                    );
                }
            }

            self.update_surface(surface_id, new_state);
        }
    }
}
