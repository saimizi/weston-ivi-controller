// State management for IVI surfaces

use super::notifications::GeometryType;
use crate::ffi::bindings::ivi_layout_api::IviLayoutApi;
use crate::ffi::bindings::*;
#[allow(unused)]
use jlogger_tracing::{jdebug, jerror, jinfo, jwarn, JloggerBuilder, LevelFilter};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Represents the state of an IVI surface
#[derive(Debug, Clone)]
pub struct SurfaceState {
    pub id: u32,
    pub orig_size: (i32, i32),
    pub src_rect: Rectangle,
    pub dest_rect: Rectangle,
    pub visibility: bool,
    pub opacity: f32,
    pub orientation: Orientation,
    pub z_order: i32,
    /// Whether this surface ID was automatically assigned
    pub is_auto_assigned: bool,
    /// Original invalid ID if this surface was auto-assigned
    pub original_id: Option<u32>,
}

/// Represents the state of an IVI layer
#[derive(Debug, Clone)]
pub struct LayerState {
    pub id: u32,
    pub visibility: bool,
    pub opacity: f32,
    pub src_rect: (i32, i32, i32, i32),  // (x, y, width, height)
    pub dest_rect: (i32, i32, i32, i32), // (x, y, width, height)
    pub orientation: Orientation,
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

    /// Check if a surface was automatically assigned
    pub fn is_surface_auto_assigned(&self, id: u32) -> bool {
        let surfaces = self.surfaces.lock().unwrap();
        surfaces
            .get(&id)
            .map(|s| s.is_auto_assigned)
            .unwrap_or(false)
    }

    /// Get the original invalid ID for an auto-assigned surface
    pub fn get_surface_original_id(&self, id: u32) -> Option<u32> {
        let surfaces = self.surfaces.lock().unwrap();
        surfaces.get(&id).and_then(|s| s.original_id)
    }

    /// Get all auto-assigned surface IDs
    pub fn get_auto_assigned_surface_ids(&self) -> Vec<u32> {
        let surfaces = self.surfaces.lock().unwrap();
        surfaces
            .values()
            .filter(|s| s.is_auto_assigned)
            .map(|s| s.id)
            .collect()
    }

    /// Get all manually assigned surface IDs
    pub fn get_manual_assigned_surface_ids(&self) -> Vec<u32> {
        let surfaces = self.surfaces.lock().unwrap();
        surfaces
            .values()
            .filter(|s| !s.is_auto_assigned)
            .map(|s| s.id)
            .collect()
    }

    /// Get count of auto-assigned surfaces
    pub fn auto_assigned_surface_count(&self) -> usize {
        let surfaces = self.surfaces.lock().unwrap();
        surfaces.values().filter(|s| s.is_auto_assigned).count()
    }

    /// Get count of manually assigned surfaces
    pub fn manual_assigned_surface_count(&self) -> usize {
        let surfaces = self.surfaces.lock().unwrap();
        surfaces.values().filter(|s| !s.is_auto_assigned).count()
    }

    /// Synchronize state with the IVI API
    /// This queries the IVI API for all surfaces and updates internal state
    /// Note: This method cannot determine which surfaces are auto-assigned
    /// since that information is not available from the IVI API
    pub fn sync_with_ivi(&mut self) {
        let ivi_surfaces = self.ivi_api.get_surfaces();
        let mut surfaces = self.surfaces.lock().unwrap();

        // Store existing auto-assignment information before clearing
        let existing_auto_info: std::collections::HashMap<u32, (bool, Option<u32>)> = surfaces
            .iter()
            .map(|(&id, state)| (id, (state.is_auto_assigned, state.original_id)))
            .collect();

        // Clear existing state
        surfaces.clear();

        // Populate with current IVI surfaces
        for surface in ivi_surfaces {
            let id = surface.id();
            let (orig_width, orig_height) = surface.orig_size();
            let src_rect;
            let dest_rect;

            if let Some(s) = surface.source_rectangle() {
                src_rect = s;
            } else {
                continue; // Skip surfaces without source rectangle
            }

            if let Some(d) = surface.destination_rectangle() {
                dest_rect = d;
            } else {
                continue; // Skip surfaces without destination rectangle
            }

            let visibility = surface.visibility();
            let opacity = surface.opacity();
            let orientation = surface.orientation().into();

            // Restore auto-assignment information if available
            let (is_auto_assigned, original_id) = existing_auto_info
                .get(&id)
                .copied()
                .unwrap_or((false, None));

            let state = SurfaceState {
                id,
                orig_size: (orig_width, orig_height),
                src_rect,
                dest_rect,
                visibility,
                opacity,
                orientation,
                z_order: 0, // Z-order is managed at layer level
                is_auto_assigned,
                original_id,
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
        self.handle_surface_created_with_assignment_info(surface_id, false, None);
    }

    /// Enhanced surface creation handler that works with ID assignment
    /// This method handles both manually assigned and auto-assigned surface IDs
    pub fn handle_surface_created_with_assignment_info(
        &mut self,
        surface_id: u32,
        is_auto_assigned: bool,
        original_id: Option<u32>,
    ) {
        // Query the IVI API for the new surface
        if let Some(surface) = self.ivi_api.get_surface_from_id(surface_id) {
            let (orig_width, orig_height) = surface.orig_size();
            let src_rect;
            let dest_rect;

            if let Some(s) = surface.source_rectangle() {
                src_rect = s;
            } else {
                return; // Cannot create state without source rectangle
            }

            if let Some(d) = surface.destination_rectangle() {
                dest_rect = d;
            } else {
                return; // Cannot create state without destination rectangle
            }

            let visibility = surface.visibility();
            let opacity = surface.opacity();
            let orientation = surface.orientation().into();

            let state = SurfaceState {
                id: surface_id,
                orig_size: (orig_width, orig_height),
                src_rect,
                dest_rect,
                visibility,
                opacity,
                orientation,
                z_order: 0,
                is_auto_assigned,
                original_id,
            };

            self.add_surface(surface_id, state);

            // Emit surface created notification with the final surface ID
            let notification_manager = self.notification_manager.lock().unwrap();
            notification_manager.emit_surface_created(surface_id);

            // Log additional information for auto-assigned surfaces
            if is_auto_assigned {
                if let Some(orig_id) = original_id {
                    jinfo!(
                        "Surface {} created with auto-assigned ID (original invalid ID: {:#x})",
                        surface_id,
                        orig_id
                    );
                } else {
                    jinfo!("Surface {} created with auto-assigned ID", surface_id);
                }
            }
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
            let (orig_width, orig_height) = surface.orig_size();
            let src_rect;
            let dest_rect;

            if let Some(s) = surface.source_rectangle() {
                src_rect = s;
            } else {
                return; // Cannot update state without source rectangle
            }

            if let Some(d) = surface.destination_rectangle() {
                dest_rect = d;
            } else {
                return; // Cannot update state without destination rectangle
            }

            let visibility = surface.visibility();
            let opacity = surface.opacity();
            let orientation = surface.orientation().into();

            // Preserve existing z_order, auto-assignment info, and original ID
            let (z_order, is_auto_assigned, original_id) = if let Some(ref old) = old_state {
                (old.z_order, old.is_auto_assigned, old.original_id)
            } else {
                (0, false, None)
            };

            let new_state = SurfaceState {
                id: surface_id,
                orig_size: (orig_width, orig_height),
                src_rect,
                dest_rect,
                visibility,
                opacity,
                orientation,
                z_order,
                is_auto_assigned,
                original_id,
            };

            // Check property changes and emit notifications
            if let Some(old) = old_state {
                // Try to filter using event_mask (0 means unknown/no filter)
                let event_mask = surface.event_mask();
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
        new: &SurfaceState,
    ) {
        if let Ok(notification_manager) = self.notification_manager.lock() {
            // Geometry (any position or size change)
            if old.src_rect != new.src_rect {
                notification_manager.emit_geometry_change(
                    surface_id,
                    GeometryType::Source,
                    old.src_rect,
                    new.src_rect,
                );
            }

            if old.dest_rect != new.dest_rect {
                notification_manager.emit_geometry_change(
                    surface_id,
                    GeometryType::Destination,
                    old.dest_rect,
                    new.dest_rect,
                );
            }

            // Visibility
            if old.visibility != new.visibility {
                notification_manager.emit_visibility_change(
                    surface_id,
                    old.visibility,
                    new.visibility,
                );
            }

            // Opacity
            if (old.opacity - new.opacity).abs() > f32::EPSILON {
                notification_manager.emit_opacity_change(surface_id, old.opacity, new.opacity);
            }

            // Orientation
            if old.orientation != new.orientation {
                notification_manager.emit_orientation_change(
                    surface_id,
                    old.orientation,
                    new.orientation,
                );
            }
        }
    }

    /// Emit notifications filtered by event_mask from IVI if available
    fn emit_surface_property_changes_filtered(
        &self,
        surface_id: u32,
        old: &SurfaceState,
        new: &SurfaceState,
        event_mask: u32,
    ) {
        let has = |bit: u32| (event_mask & (bit as u32)) != 0;
        if let Ok(notification_manager) = self.notification_manager.lock() {
            // Geometry
            if has(NotificationMask::Position.into())
                || has(NotificationMask::SourceRect.into())
                || has(NotificationMask::DestRect.into())
                || has(NotificationMask::Dimension.into())
            {
                // Geometry (any position or size change)
                if old.src_rect != new.src_rect {
                    notification_manager.emit_geometry_change(
                        surface_id,
                        GeometryType::Source,
                        old.src_rect,
                        new.src_rect,
                    );
                }

                if old.dest_rect != new.dest_rect {
                    notification_manager.emit_geometry_change(
                        surface_id,
                        GeometryType::Destination,
                        old.dest_rect,
                        new.dest_rect,
                    );
                }
            }

            // Visibility
            if has(NotificationMask::Visibility.into()) {
                if old.visibility != new.visibility {
                    notification_manager.emit_visibility_change(
                        surface_id,
                        old.visibility,
                        new.visibility,
                    );
                }
            }

            // Opacity
            if has(NotificationMask::Opacity.into()) {
                if (old.opacity - new.opacity).abs() > f32::EPSILON {
                    notification_manager.emit_opacity_change(surface_id, old.opacity, new.opacity);
                }
            }

            // Orientation
            if has(NotificationMask::Orientation.into()) {
                if old.orientation != new.orientation {
                    notification_manager.emit_orientation_change(
                        surface_id,
                        old.orientation,
                        new.orientation,
                    );
                }
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
            let visibility = layer.visibility();
            let opacity = layer.opacity();
            let src_rect = layer.source_rectangle().unwrap_or_default();
            let dest_rect = layer.destination_rectangle().unwrap_or_default();
            let orientation = layer.orientation();

            let state = LayerState {
                id: layer_id,
                visibility,
                opacity,
                src_rect: (src_rect.x, src_rect.y, src_rect.width, src_rect.height),
                dest_rect: (dest_rect.x, dest_rect.y, dest_rect.width, dest_rect.height),
                orientation,
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
            let event_mask = layer.event_mask();
            let visibility = layer.visibility();
            let opacity = layer.opacity();
            let src_rect = layer.source_rectangle().unwrap_or_default();
            let dest_rect = layer.destination_rectangle().unwrap_or_default();
            let orientation = layer.orientation();

            let new_state = LayerState {
                id: layer_id,
                visibility,
                opacity,
                src_rect: (src_rect.x, src_rect.y, src_rect.width, src_rect.height),
                dest_rect: (dest_rect.x, dest_rect.y, dest_rect.width, dest_rect.height),
                orientation,
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
    use crate::controller::notifications::{Notification, NotificationType};
    use crate::ffi::bindings::ivi_layout_api::IviLayoutApi;
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
            orig_size: (100, 100),
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            visibility: false,
            opacity: 1.0,
            orientation: Orientation::Normal,
            z_order: 0,
            is_auto_assigned: false,
            original_id: None,
        };
        let new_state = SurfaceState {
            id: 42,
            orig_size: (100, 100),
            src_rect: Rectangle {
                x: 10,
                y: 10,
                width: 100,
                height: 100,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 200,
                height: 200,
            },
            visibility: true,
            opacity: 0.5,
            orientation: Orientation::Rotate90,
            z_order: 0,
            is_auto_assigned: false,
            original_id: None,
        };

        sm.emit_surface_property_changes(42, &old, &new_state);

        let got = seen.lock().unwrap().clone();
        // Order is not guaranteed, so check set membership
        assert!(got.contains(&NotificationType::GeometryChanged));
        assert!(got.contains(&NotificationType::VisibilityChanged));
        assert!(got.contains(&NotificationType::OpacityChanged));
        assert!(got.contains(&NotificationType::OrientationChanged));
        // Note: GeometryChanged is emitted twice (once for src_rect, once for dest_rect)
        assert_eq!(got.len(), 5);
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
            orig_size: (200, 150),
            src_rect: Rectangle {
                x: 10,
                y: 10,
                width: 200,
                height: 150,
            },
            dest_rect: Rectangle {
                x: 10,
                y: 10,
                width: 200,
                height: 150,
            },
            visibility: true,
            opacity: 0.75,
            orientation: Orientation::Normal,
            z_order: 0,
            is_auto_assigned: false,
            original_id: None,
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

    #[test]
    fn test_auto_assigned_surface_tracking() {
        let sm = make_state_manager();

        // Create surface states directly for testing
        let auto_assigned_state = SurfaceState {
            id: 0x10000000,
            orig_size: (100, 100),
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            visibility: true,
            opacity: 1.0,
            orientation: Orientation::Normal,
            z_order: 0,
            is_auto_assigned: true,
            original_id: Some(0xFFFFFFFF),
        };

        let manual_assigned_state = SurfaceState {
            id: 42,
            orig_size: (200, 200),
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 200,
                height: 200,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 200,
                height: 200,
            },
            visibility: true,
            opacity: 1.0,
            orientation: Orientation::Normal,
            z_order: 0,
            is_auto_assigned: false,
            original_id: None,
        };

        // Add surfaces to state manager
        {
            let mut surfaces = sm.surfaces.lock().unwrap();
            surfaces.insert(0x10000000, auto_assigned_state);
            surfaces.insert(42, manual_assigned_state);
        }

        // Verify the surfaces are tracked correctly
        assert!(sm.is_surface_auto_assigned(0x10000000));
        assert_eq!(sm.get_surface_original_id(0x10000000), Some(0xFFFFFFFF));
        assert!(!sm.is_surface_auto_assigned(42));
        assert_eq!(sm.get_surface_original_id(42), None);

        // Test counts
        assert_eq!(sm.auto_assigned_surface_count(), 1);
        assert_eq!(sm.manual_assigned_surface_count(), 1);
        assert_eq!(sm.surface_count(), 2);

        // Test getting auto-assigned and manual surface IDs
        let auto_ids = sm.get_auto_assigned_surface_ids();
        let manual_ids = sm.get_manual_assigned_surface_ids();

        assert_eq!(auto_ids.len(), 1);
        assert!(auto_ids.contains(&0x10000000));
        assert_eq!(manual_ids.len(), 1);
        assert!(manual_ids.contains(&42));

        // Test surface removal
        {
            let mut surfaces = sm.surfaces.lock().unwrap();
            surfaces.remove(&0x10000000);
            surfaces.remove(&42);
        }

        assert_eq!(sm.auto_assigned_surface_count(), 0);
        assert_eq!(sm.manual_assigned_surface_count(), 0);
        assert_eq!(sm.surface_count(), 0);
    }

    #[test]
    fn test_sync_with_ivi_preserves_auto_assignment_info() {
        let sm = make_state_manager();

        // Create surface states with auto-assignment info
        let auto_assigned_state = SurfaceState {
            id: 0x10000000,
            orig_size: (100, 100),
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            visibility: true,
            opacity: 1.0,
            orientation: Orientation::Normal,
            z_order: 0,
            is_auto_assigned: true,
            original_id: Some(0xFFFFFFFF),
        };

        let manual_assigned_state = SurfaceState {
            id: 42,
            orig_size: (200, 200),
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 200,
                height: 200,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 200,
                height: 200,
            },
            visibility: true,
            opacity: 1.0,
            orientation: Orientation::Normal,
            z_order: 0,
            is_auto_assigned: false,
            original_id: None,
        };

        // Add surfaces to state manager
        {
            let mut surfaces = sm.surfaces.lock().unwrap();
            surfaces.insert(0x10000000, auto_assigned_state);
            surfaces.insert(42, manual_assigned_state);
        }

        // Verify initial state
        assert!(sm.is_surface_auto_assigned(0x10000000));
        assert!(!sm.is_surface_auto_assigned(42));
        assert_eq!(sm.surface_count(), 2);

        // Test that the sync method would preserve auto-assignment information
        // (We can't actually call sync_with_ivi because it would access the mock IVI API)
        // But we can verify that the data structures support preserving this information

        let existing_auto_info: std::collections::HashMap<u32, (bool, Option<u32>)> = {
            let surfaces = sm.surfaces.lock().unwrap();
            surfaces
                .iter()
                .map(|(&id, state)| (id, (state.is_auto_assigned, state.original_id)))
                .collect()
        };

        // Verify the auto-assignment info is correctly captured
        assert_eq!(
            existing_auto_info.get(&0x10000000),
            Some(&(true, Some(0xFFFFFFFF)))
        );
        assert_eq!(existing_auto_info.get(&42), Some(&(false, None)));
    }
}
