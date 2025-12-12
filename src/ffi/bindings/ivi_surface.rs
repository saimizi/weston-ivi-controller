use super::ivi_layout_api::IviLayoutApi;
use super::ivi_layout_surface_properties_m::IviLayoutSurfaceProperties;
use super::*;

/// Safe wrapper around an IVI surface
#[derive(Clone)]
pub struct IviSurface {
    handle: *mut ivi_layout_surface,
    api: Arc<IviLayoutApi>,
}

// Safety: IVI surfaces are thread-safe as per Weston's design
unsafe impl Send for IviSurface {}
unsafe impl Sync for IviSurface {}

impl IviSurface {
    pub(crate) fn handle(&self) -> *mut ivi_layout_surface {
        self.handle
    }

    pub fn new(handle: *mut ivi_layout_surface, api: Arc<IviLayoutApi>) -> Option<Self> {
        if handle.is_null() {
            return None;
        }

        Some(IviSurface { handle, api })
    }

    /// Get the surface ID
    pub fn id(&self) -> u32 {
        self.api.get_id_of_surface(self)
    }

    /// Get surface properties
    fn properties(&self) -> Option<IviLayoutSurfaceProperties> {
        self.api.get_properties_of_surface(self)
    }

    /// Get destination
    pub fn destination_rectangle(&self) -> Option<Rectangle> {
        self.api
            .get_properties_of_surface(self)
            .map(|props| props.destination_rectangle())
    }

    /// Get the original buffer size of the surface
    pub fn orig_size(&self) -> (i32, i32) {
        self.api
            .surface_get_weston_surface(self)
            .map_or((0, 0), |weston_surface| {
                (weston_surface.width(), weston_surface.height())
            })
    }

    /// Get surface source rectangle
    pub fn source_rectangle(&self) -> Option<Rectangle> {
        self.api
            .get_properties_of_surface(self)
            .map(|props| props.source_rectangle())
    }

    /// Set surface size (destination rectangle)
    pub fn set_source_rectangle(&mut self, rect: Rectangle) -> Result<(), String> {
        // Validate size
        crate::controller::validation::validate_position(rect.x, rect.y)
            .map_err(|e| e.to_string())?;
        crate::controller::validation::validate_size(rect.width, rect.height)
            .map_err(|e| e.to_string())?;

        self.api
            .surface_set_source_rectangle(self, rect.x, rect.y, rect.width, rect.height)
            .map_err(|e| e.to_string())
    }

    /// Set surface destination rectangle (position and size)
    pub fn set_destination_rectangle(&mut self, rect: Rectangle) -> Result<(), String> {
        // Validate position and size
        crate::controller::validation::validate_position(rect.x, rect.y)
            .map_err(|e| e.to_string())?;
        crate::controller::validation::validate_size(rect.width, rect.height)
            .map_err(|e| e.to_string())?;

        self.api
            .surface_set_destination_rectangle(self, rect.x, rect.y, rect.width, rect.height)
            .map_err(|e| e.to_string())
    }

    /// Get surface visibility
    pub fn visibility(&self) -> bool {
        self.api
            .get_properties_of_surface(self)
            .map(|props| props.visibility())
            .unwrap_or(false)
    }

    /// Get surface event mask (what changed)
    pub fn event_mask(&self) -> u32 {
        self.api
            .get_properties_of_surface(self)
            .map(|props| props.event_mask())
            .unwrap_or(0)
    }

    /// Set surface visibility
    pub fn set_visibility(&mut self, visible: bool) -> Result<(), String> {
        self.api
            .surface_set_visibility(self, visible)
            .map_err(|e| e.to_string())
    }

    /// Get surface opacity (returns value in range 0.0 to 1.0)
    pub fn opacity(&self) -> f32 {
        self.api
            .get_properties_of_surface(self)
            .map_or(1.0, |props| props.opacity())
    }

    /// Set surface opacity (value should be in range 0.0 to 1.0)
    pub fn set_opacity(&mut self, opacity: f32) -> Result<(), String> {
        // Validate opacity
        crate::controller::validation::validate_opacity(opacity).map_err(|e| e.to_string())?;

        self.api
            .surface_set_opacity(self, opacity)
            .map_err(|e| e.to_string())
    }

    /// Get surface orientation
    pub fn orientation(&self) -> Orientation {
        self.api
            .get_properties_of_surface(self)
            .map(|props| props.orientation())
            .unwrap_or(Orientation::Normal)
    }

    /// Set surface orientation from degrees (not supported in current IVI API)
    pub fn set_orientation(&mut self, degrees: i32) -> Result<(), String> {
        // Validate to provide consistent error messages, then report unsupported
        let _ = Orientation::from_degrees(degrees)?;
        Err("Orientation control not supported by current IVI API".to_string())
    }

    /// Set keyboard focus to this surface
    /// Note: The IVI layout API doesn't separate keyboard and pointer focus,
    /// so this activates the surface which sets both keyboard and pointer focus
    pub fn set_keyboard_focus(&mut self) -> Result<(), String> {
        self.api.surface_activate(self).map_err(|e| e.to_string())
    }

    /// Set pointer focus to this surface
    /// Note: The IVI layout API doesn't separate keyboard and pointer focus,
    /// so this activates the surface which sets both keyboard and pointer focus
    pub fn set_pointer_focus(&mut self) -> Result<(), String> {
        self.api.surface_activate(self).map_err(|e| e.to_string())
    }

    /// Activate the surface (set input focus)
    /// This sets both keyboard and pointer focus
    pub fn activate(&mut self) {
        match self.api.surface_activate(self) {
            Ok(_) => {}
            Err(e) => eprintln!("Error activating surface: {}", e),
        }
    }

    /// Check if the surface is active (has focus)
    pub fn is_active(&self) -> bool {
        self.api.surface_is_active(self)
    }

    /// Check if the surface has keyboard focus
    /// Note: The IVI layout API doesn't separate keyboard and pointer focus,
    /// so this returns the same as is_active()
    pub fn has_keyboard_focus(&self) -> bool {
        self.is_active()
    }

    /// Check if the surface has pointer focus
    /// Note: The IVI layout API doesn't separate keyboard and pointer focus,
    /// so this returns the same as is_active()
    pub fn has_pointer_focus(&self) -> bool {
        self.is_active()
    }

    /// Set surface z-order within its layer
    /// Note: Z-order is managed through layer render order in IVI shell.
    /// This method validates the z-order value but actual implementation
    /// requires layer context.
    pub fn set_z_order(&mut self, z_order: i32, min: i32, max: i32) -> Result<(), String> {
        // Validate z-order
        crate::controller::validation::validate_z_order(z_order, min, max)
            .map_err(|e| e.to_string())?;

        let mut layers = self
            .api
            .get_layers_under_surface(self)
            .map_err(|e| e.to_string())?;

        for layer in layers.iter_mut() {
            let mut surfaces = layer.get_surfaces();
            surfaces.sort_by_key(|s| s.id());

            // Remove the surface if it exists
            surfaces.retain(|s| s.id() != self.id());

            // Insert at the desired z-order position
            let insert_index = z_order.min(surfaces.len() as i32) as usize;
            surfaces.insert(insert_index, self.clone());

            // Update layer render order
            let surface_refs: Vec<&IviSurface> = surfaces.iter().collect();
            layer
                .set_render_order(&surface_refs)
                .map_err(|e| format!("Failed to set render order: {}", e))?;
        }
        Ok(())
    }
}
