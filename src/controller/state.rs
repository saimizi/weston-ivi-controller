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

/// Represents the state of an IVI layer
#[derive(Debug, Clone)]
pub struct LayerState {
    pub id: u32,
    pub visibility: bool,
    pub opacity: f32,
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

/// Manages the state of all IVI surfaces and layers
pub struct StateManager {
    surfaces: Arc<Mutex<HashMap<u32, SurfaceState>>>,
    layers: Arc<Mutex<HashMap<u32, LayerState>>>,
    ivi_api: Arc<IviLayoutApi>,
    notification_manager: Arc<Mutex<super::notifications::NotificationManager>>,
    focused_surface: Arc<Mutex<Option<u32>>>,
}

impl StateManager {
    /// Create a new StateManager
    pub fn new(ivi_api: Arc<IviLayoutApi>) -> Self {
        Self {
            surfaces: Arc::new(Mutex::new(HashMap::new())),
            layers: Arc::new(Mutex::new(HashMap::new())),
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

            // Check property changes and emit notifications
            if let Some(old) = old_state {
                // Try to filter using event_mask (0 means unknown/no filter)
                let event_mask = surface.get_event_mask();
                if event_mask == 0 {
                    self.emit_surface_property_changes(surface_id, &old, &new_state);
                } else {
                    self.emit_surface_property_changes_filtered(
                        surface_id, &old, &new_state, event_mask,
                    );
                }
            }

            self.update_surface(surface_id, new_state);
        }
    }

    /// Emit notifications for property changes between two surface states
    fn emit_surface_property_changes(
        &self,
        surface_id: u32,
        old: &SurfaceState,
        new_state: &SurfaceState,
    ) {
        // Geometry (position or size)
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

        // Visibility
        if old.visibility != new_state.visibility {
            let notification_manager = self.notification_manager.lock().unwrap();
            notification_manager.emit_visibility_change(
                surface_id,
                old.visibility,
                new_state.visibility,
            );
        }

        // Opacity
        if (old.opacity - new_state.opacity).abs() > f32::EPSILON {
            let notification_manager = self.notification_manager.lock().unwrap();
            notification_manager.emit_opacity_change(surface_id, old.opacity, new_state.opacity);
        }

        // Orientation
        if old.orientation != new_state.orientation {
            let notification_manager = self.notification_manager.lock().unwrap();
            notification_manager.emit_orientation_change(
                surface_id,
                old.orientation,
                new_state.orientation,
            );
        }
    }

    /// Emit notifications filtered by event_mask from IVI if available
    fn emit_surface_property_changes_filtered(
        &self,
        surface_id: u32,
        old: &SurfaceState,
        new_state: &SurfaceState,
        event_mask: u32,
    ) {
        // Bit definitions from ivi_layout_notification_mask
        const NOTIF_OPACITY: u32 = 1 << 1;
        const NOTIF_DEST_RECT: u32 = 1 << 3;
        const NOTIF_DIMENSION: u32 = 1 << 4;
        const NOTIF_POSITION: u32 = 1 << 5;
        const NOTIF_ORIENTATION: u32 = 1 << 6;
        const NOTIF_VISIBILITY: u32 = 1 << 7;
        let has = |bit: u32| (event_mask & (bit as u32)) != 0;

        // Geometry
        if has(NOTIF_POSITION) || has(NOTIF_DEST_RECT) || has(NOTIF_DIMENSION) {
            if old.position != new_state.position || old.size != new_state.size {
                let nm = self.notification_manager.lock().unwrap();
                nm.emit_geometry_change(
                    surface_id,
                    old.position,
                    new_state.position,
                    old.size,
                    new_state.size,
                );
            }
        }

        // Visibility
        if has(NOTIF_VISIBILITY) {
            if old.visibility != new_state.visibility {
                let nm = self.notification_manager.lock().unwrap();
                nm.emit_visibility_change(surface_id, old.visibility, new_state.visibility);
            }
        }

        // Opacity
        if has(NOTIF_OPACITY) {
            if (old.opacity - new_state.opacity).abs() > f32::EPSILON {
                let nm = self.notification_manager.lock().unwrap();
                nm.emit_opacity_change(surface_id, old.opacity, new_state.opacity);
            }
        }

        // Orientation
        if has(NOTIF_ORIENTATION) {
            if old.orientation != new_state.orientation {
                let nm = self.notification_manager.lock().unwrap();
                nm.emit_orientation_change(surface_id, old.orientation, new_state.orientation);
            }
        }
    }

    // ===== Layer Management Methods =====

    /// Add a layer to the state manager
    pub fn add_layer(&mut self, id: u32, state: LayerState) {
        jinfo!("Adding layer {} to state manager", id);
        let mut layers = self.layers.lock().unwrap();
        layers.insert(id, state);
    }

    /// Remove a layer from the state manager
    pub fn remove_layer(&mut self, id: u32) -> Option<LayerState> {
        jinfo!("Removing layer {} from state manager", id);
        let mut layers = self.layers.lock().unwrap();
        layers.remove(&id)
    }

    /// Update layer state
    pub fn update_layer(&mut self, id: u32, state: LayerState) {
        jdebug!("Updating layer {} state", id);
        let mut layers = self.layers.lock().unwrap();
        layers.insert(id, state);
    }

    /// Get layer state by ID
    pub fn get_layer(&self, id: u32) -> Option<LayerState> {
        let layers = self.layers.lock().unwrap();
        layers.get(&id).cloned()
    }

    /// Get all layers
    pub fn get_all_layers(&self) -> Vec<LayerState> {
        let layers = self.layers.lock().unwrap();
        layers.values().cloned().collect()
    }

    /// Get the number of tracked layers
    pub fn layer_count(&self) -> usize {
        let layers = self.layers.lock().unwrap();
        layers.len()
    }

    /// Check if a layer exists
    pub fn has_layer(&self, id: u32) -> bool {
        let layers = self.layers.lock().unwrap();
        layers.contains_key(&id)
    }

    /// Handle layer creation event
    /// This is called by the event listener when a new layer is created
    pub fn handle_layer_created(&mut self, layer_id: u32) {
        // Query the IVI API for the new layer
        if let Some(layer) = self.ivi_api.get_layer_from_id(layer_id) {
            let visibility = layer.get_visibility();
            let opacity = layer.get_opacity();

            let state = LayerState {
                id: layer_id,
                visibility,
                opacity,
            };

            self.add_layer(layer_id, state);

            // Emit layer created notification
            let notification_manager = self.notification_manager.lock().unwrap();
            notification_manager.emit_layer_created(layer_id);
        }
    }

    /// Handle layer destruction event
    /// This is called by the event listener when a layer is destroyed
    pub fn handle_layer_destroyed(&mut self, layer_id: u32) {
        self.remove_layer(layer_id);

        // Emit layer destroyed notification
        let notification_manager = self.notification_manager.lock().unwrap();
        notification_manager.emit_layer_destroyed(layer_id);
    }

    /// Handle layer configuration event
    /// This is called by the event listener when a layer is configured
    pub fn handle_layer_configured(&mut self, layer_id: u32) {
        // Get old state for comparison
        let old_state = self.get_layer(layer_id);

        // Query the IVI API for updated layer properties
        if let Some(layer) = self.ivi_api.get_layer_from_id(layer_id) {
            let event_mask = layer.get_event_mask();
            let visibility = layer.get_visibility();
            let opacity = layer.get_opacity();

            let new_state = LayerState {
                id: layer_id,
                visibility,
                opacity,
            };

            // Check if visibility changed and emit notification
            if let Some(ref old) = old_state {
                // Bit definitions from ivi_layout_notification_mask
                const NOTIF_OPACITY: u32 = 1 << 1;
                const NOTIF_VISIBILITY: u32 = 1 << 7;
                let has = |bit: u32| (event_mask & (bit as u32)) != 0;

                if has(NOTIF_VISIBILITY) {
                    if old.visibility != new_state.visibility {
                        let nm = self.notification_manager.lock().unwrap();
                        nm.emit_layer_visibility_change(
                            layer_id,
                            old.visibility,
                            new_state.visibility,
                        );
                    }
                }

                if has(NOTIF_OPACITY) {
                    if (old.opacity - new_state.opacity).abs() > f32::EPSILON {
                        let nm = self.notification_manager.lock().unwrap();
                        nm.emit_layer_opacity_change(layer_id, old.opacity, new_state.opacity);
                    }
                }
            }

            self.update_layer(layer_id, new_state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::ivi_wrapper::IviLayoutApi;
    use crate::controller::notifications::{Notification, NotificationType};
    use std::sync::{Arc, Mutex};

    fn make_state_manager() -> StateManager {
        let ivi_api = unsafe { Arc::new(IviLayoutApi::from_raw(1 as *const _).unwrap()) };
        StateManager::new(ivi_api)
    }

    #[test]
    fn emits_visibility_opacity_orientation_and_geometry_changes() {
        let mut sm = make_state_manager();

        // Collect notifications
        let seen: Arc<Mutex<Vec<NotificationType>>> = Arc::new(Mutex::new(Vec::new()));
        let nm_arc = sm.notification_manager();
        {
            let mut nm = nm_arc.lock().unwrap();
            for nt in [
                NotificationType::GeometryChanged,
                NotificationType::VisibilityChanged,
                NotificationType::OpacityChanged,
                NotificationType::OrientationChanged,
            ] {
                let seen_clone = Arc::clone(&seen);
                nm.register_callback(
                    nt,
                    Arc::new(move |n: &Notification| {
                        seen_clone.lock().unwrap().push(n.notification_type);
                    }),
                );
            }
        }

        let old = SurfaceState {
            id: 42,
            position: (0, 0),
            size: (100, 100),
            visibility: false,
            opacity: 1.0,
            orientation: Orientation::Normal,
            z_order: 0,
        };
        let new_state = SurfaceState {
            id: 42,
            position: (10, 20),
            size: (120, 110),
            visibility: true,
            opacity: 0.5,
            orientation: Orientation::Rotate90,
            z_order: 0,
        };

        sm.emit_surface_property_changes(42, &old, &new_state);

        let got = seen.lock().unwrap().clone();
        // Order is not guaranteed, so check set membership
        assert!(got.contains(&NotificationType::GeometryChanged));
        assert!(got.contains(&NotificationType::VisibilityChanged));
        assert!(got.contains(&NotificationType::OpacityChanged));
        assert!(got.contains(&NotificationType::OrientationChanged));
        assert_eq!(got.len(), 4);
    }

    #[test]
    fn emits_z_order_change_via_notification_manager() {
        let sm = make_state_manager();
        let nm_arc = sm.notification_manager();

        // Register a ZOrderChanged callback
        let flag = Arc::new(Mutex::new(None::<(i32, i32)>));
        {
            let mut nm = nm_arc.lock().unwrap();
            let flag_clone = Arc::clone(&flag);
            nm.register_callback(
                NotificationType::ZOrderChanged,
                Arc::new(move |n: &Notification| {
                    if let crate::controller::notifications::NotificationData::ZOrderChange(z) =
                        &n.data
                    {
                        *flag_clone.lock().unwrap() = Some((z.old_z_order, z.new_z_order));
                    }
                }),
            );
        }

        // Emit z-order change
        nm_arc.lock().unwrap().emit_z_order_change(7, 1, 5);

        let got = flag.lock().unwrap().clone();
        assert_eq!(got, Some((1, 5)));
    }

    #[test]
    fn emits_only_orientation_change_when_only_orientation_differs() {
        let mut sm = make_state_manager();

        let seen: Arc<Mutex<Vec<NotificationType>>> = Arc::new(Mutex::new(Vec::new()));
        let nm_arc = sm.notification_manager();
        {
            let mut nm = nm_arc.lock().unwrap();
            for nt in [
                NotificationType::GeometryChanged,
                NotificationType::VisibilityChanged,
                NotificationType::OpacityChanged,
                NotificationType::OrientationChanged,
            ] {
                let seen_clone = Arc::clone(&seen);
                nm.register_callback(
                    nt,
                    Arc::new(move |n: &Notification| {
                        seen_clone.lock().unwrap().push(n.notification_type);
                    }),
                );
            }
        }

        let old = SurfaceState {
            id: 1,
            position: (10, 10),
            size: (200, 150),
            visibility: true,
            opacity: 0.75,
            orientation: Orientation::Normal,
            z_order: 0,
        };
        let new_state = SurfaceState {
            orientation: Orientation::Rotate180,
            ..old.clone()
        };

        sm.emit_surface_property_changes(1, &old, &new_state);

        let got = seen.lock().unwrap().clone();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0], NotificationType::OrientationChanged);
    }
}
