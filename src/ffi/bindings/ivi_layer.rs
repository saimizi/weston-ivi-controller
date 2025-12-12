use super::ivi_layout_api::IviLayoutApi;
use super::ivi_layout_layer_properties_m::IviLayoutLayerProperties;
use super::ivi_surface::IviSurface;
use super::*;

/// Safe wrapper around an IVI layer
pub struct IviLayer {
    handle: *mut ivi_layout_layer,
    api: Arc<IviLayoutApi>,
}

// Safety: IVI layers are thread-safe as per Weston's design
unsafe impl Send for IviLayer {}
unsafe impl Sync for IviLayer {}

impl IviLayer {
    pub(crate) fn handle(&self) -> *mut ivi_layout_layer {
        self.handle
    }

    pub fn new(handle: *mut ivi_layout_layer, api: Arc<IviLayoutApi>) -> Option<Self> {
        if handle.is_null() {
            None
        } else {
            Some(IviLayer { handle, api })
        }
    }

    /// Get the layer ID
    pub fn id(&self) -> u32 {
        self.api.get_id_of_layer(self)
    }

    /// Get layer properties
    fn properties(&self) -> Option<IviLayoutLayerProperties> {
        self.api.get_properties_of_layer(self)
    }

    /// Get layer visibility
    pub fn visibility(&self) -> bool {
        self.api
            .get_properties_of_layer(self)
            .map_or(false, |p| p.visibility())
    }

    /// Get layer event mask (what changed)
    pub fn event_mask(&self) -> u32 {
        self.api
            .get_properties_of_layer(self)
            .map_or(0, |p| p.event_mask())
    }

    /// Get layer source rectangle
    pub fn source_rectangle(&self) -> Option<Rectangle> {
        self.properties().map(|p| p.source_rectangle())
    }

    /// Set layer source rectangle
    pub fn set_source_rectangle(&mut self, rect: Rectangle) -> Result<(), String> {
        self.api
            .layer_set_source_rectangle(self, rect.x, rect.y, rect.width, rect.height)
            .map_err(|e| e.to_string())
    }

    /// Set layer destination rectangle
    pub fn set_destination_rectangle(&mut self, rect: Rectangle) -> Result<(), String> {
        self.api
            .layer_set_destination_rectangle(self, rect.x, rect.y, rect.width, rect.height)
            .map_err(|e| e.to_string())
    }

    /// Set layer visibility
    pub fn set_visibility(&mut self, visibility: bool) -> Result<(), String> {
        self.api
            .layer_set_visibility(self, visibility)
            .map_err(|e| e.to_string())
    }

    /// Get layer opacity
    pub fn opacity(&self) -> f32 {
        self.api
            .get_properties_of_layer(self)
            .map_or(1.0, |props| props.opacity())
    }

    /// Set layer opacity
    pub fn set_opacity(&mut self, opacity: f32) -> Result<(), String> {
        // Validate opacity
        crate::controller::validation::validate_opacity(opacity)
            .map_err(|e| e.to_string())
            .map_err(|_| "Invalid opacity value")?;

        self.api
            .layer_set_opacity(self, opacity)
            .map_err(|e| e.to_string())
    }

    /// Add surface to layer
    pub fn add_surface(&mut self, surface: &IviSurface) -> Result<(), String> {
        self.api
            .layer_add_surface(self, surface)
            .map_err(|e| e.to_string())
    }

    /// Remove surface from layer
    pub fn remove_surface(&mut self, surface: &IviSurface) -> Result<(), String> {
        self.api
            .layer_remove_surface(self, surface)
            .map_err(|e| e.to_string())
    }

    /// Get surfaces on this layer
    pub fn get_surfaces(&self) -> Vec<IviSurface> {
        self.api.get_surfaces_on_layer(self)
    }

    /// Set the render order of surfaces in this layer
    pub fn set_render_order(&mut self, surfaces: &[&IviSurface]) -> Result<(), String> {
        self.api
            .layer_set_render_order(self, surfaces)
            .map_err(|e| e.to_string())
    }

    /// Destroy this layer
    pub fn destroy(self) -> Result<(), String> {
        self.api.layer_destroy(&self).map_err(|e| e.to_string())
    }
}
