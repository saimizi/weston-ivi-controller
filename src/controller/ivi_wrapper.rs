// Safe Rust wrapper around the IVI layout C API

use crate::ffi::bindings::*;
use std::ptr;
use std::sync::Arc;

/// Orientation of a surface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Normal,    // 0 degrees
    Rotate90,  // 90 degrees
    Rotate180, // 180 degrees
    Rotate270, // 270 degrees
}

impl Orientation {
    /// Convert from wl_output_transform
    pub fn from_wl_transform(transform: u32) -> Self {
        match transform {
            0 => Orientation::Normal,
            1 => Orientation::Rotate90,
            2 => Orientation::Rotate180,
            3 => Orientation::Rotate270,
            _ => Orientation::Normal, // Default to normal for unknown values
        }
    }

    /// Convert to wl_output_transform
    pub fn to_wl_transform(&self) -> u32 {
        match self {
            Orientation::Normal => 0,
            Orientation::Rotate90 => 1,
            Orientation::Rotate180 => 2,
            Orientation::Rotate270 => 3,
        }
    }

    /// Create Orientation from degrees with validation
    pub fn from_degrees(degrees: i32) -> Result<Self, String> {
        // Validate orientation
        crate::controller::validation::validate_orientation(degrees).map_err(|e| e.to_string())?;

        // Normalize to 0-359 range
        let normalized = ((degrees % 360) + 360) % 360;

        match normalized {
            0 => Ok(Orientation::Normal),
            90 => Ok(Orientation::Rotate90),
            180 => Ok(Orientation::Rotate180),
            270 => Ok(Orientation::Rotate270),
            _ => Ok(Orientation::Normal), // Should not happen after validation
        }
    }

    /// Convert to degrees
    pub fn to_degrees(&self) -> i32 {
        match self {
            Orientation::Normal => 0,
            Orientation::Rotate90 => 90,
            Orientation::Rotate180 => 180,
            Orientation::Rotate270 => 270,
        }
    }
}

/// Safe wrapper around the IVI layout API
pub struct IviLayoutApi {
    pub(crate) api: *const ivi_layout_interface,
}

// Safety: The IVI layout API is thread-safe as per Weston's design
unsafe impl Send for IviLayoutApi {}
unsafe impl Sync for IviLayoutApi {}

impl IviLayoutApi {
    /// Create a new IviLayoutApi wrapper from a raw pointer
    ///
    /// # Safety
    /// The caller must ensure that the pointer is valid and points to a valid
    /// ivi_layout_interface that will remain valid for the lifetime of this wrapper.
    pub unsafe fn from_raw(api: *const ivi_layout_interface) -> Option<Self> {
        if api.is_null() {
            return None;
        }
        Some(Self { api })
    }

    /// Commit all changes and execute all enqueued commands
    pub fn commit_changes(&self) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let commit_fn = (*self.api)
                .commit_changes
                .ok_or("commit_changes function is null")?;
            let result = commit_fn();

            if result == IVI_SUCCEEDED {
                Ok(())
            } else {
                Err("Failed to commit changes")
            }
        }
    }

    /// Get a surface by its ID
    pub fn get_surface_from_id(&self, id: u32) -> Option<IviSurface> {
        unsafe {
            if self.api.is_null() {
                return None;
            }

            let get_surface_fn = (*self.api).get_surface_from_id?;
            let handle = get_surface_fn(id);

            if handle.is_null() {
                None
            } else {
                Some(IviSurface {
                    handle,
                    api: Arc::new(Self { api: self.api }),
                })
            }
        }
    }

    /// Get all surfaces
    pub fn get_surfaces(&self) -> Vec<IviSurface> {
        unsafe {
            if self.api.is_null() {
                return Vec::new();
            }

            let get_surfaces_fn = match (*self.api).get_surfaces {
                Some(f) => f,
                None => return Vec::new(),
            };

            let mut length: i32 = 0;
            let mut array: *mut *mut ivi_layout_surface = ptr::null_mut();

            get_surfaces_fn(&mut length, &mut array);

            if array.is_null() || length <= 0 {
                return Vec::new();
            }

            let api = Arc::new(Self { api: self.api });
            let mut surfaces = Vec::new();

            for i in 0..length as isize {
                let handle = *array.offset(i);
                if !handle.is_null() {
                    surfaces.push(IviSurface {
                        handle,
                        api: api.clone(),
                    });
                }
            }

            surfaces
        }
    }

    /// Get a layer by its ID
    pub fn get_layer_from_id(&self, id: u32) -> Option<IviLayer> {
        unsafe {
            if self.api.is_null() {
                return None;
            }

            let get_layer_fn = (*self.api).get_layer_from_id?;
            let handle = get_layer_fn(id);

            if handle.is_null() {
                None
            } else {
                Some(IviLayer {
                    handle,
                    api: Arc::new(Self { api: self.api }),
                })
            }
        }
    }

    /// Get all layers
    pub fn get_layers(&self) -> Vec<IviLayer> {
        unsafe {
            if self.api.is_null() {
                return Vec::new();
            }

            let get_layers_fn = match (*self.api).get_layers {
                Some(f) => f,
                None => return Vec::new(),
            };

            let mut length: i32 = 0;
            let mut array: *mut *mut ivi_layout_layer = std::ptr::null_mut();

            get_layers_fn(&mut length, &mut array);

            if array.is_null() || length <= 0 {
                return Vec::new();
            }

            let api = Arc::new(Self { api: self.api });
            let mut layers = Vec::new();

            for i in 0..length as isize {
                let handle = *array.offset(i);
                if !handle.is_null() {
                    layers.push(IviLayer {
                        handle,
                        api: api.clone(),
                    });
                }
            }

            layers
        }
    }

    /// Create a new layer with the given ID and dimensions
    pub fn create_layer(&self, id: u32, width: i32, height: i32) -> Option<IviLayer> {
        unsafe {
            if self.api.is_null() {
                return None;
            }

            let create_layer_fn = (*self.api).layer_create_with_dimension?;
            let handle = create_layer_fn(id, width, height);

            if handle.is_null() {
                None
            } else {
                Some(IviLayer {
                    handle,
                    api: Arc::new(Self { api: self.api }),
                })
            }
        }
    }
}

/// Safe wrapper around an IVI surface
pub struct IviSurface {
    handle: *mut ivi_layout_surface,
    api: Arc<IviLayoutApi>,
}

// Safety: IVI surfaces are thread-safe as per Weston's design
unsafe impl Send for IviSurface {}
unsafe impl Sync for IviSurface {}

impl IviSurface {
    /// Get the surface ID
    pub fn get_id(&self) -> u32 {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return 0;
            }

            match (*self.api.api).get_id_of_surface {
                Some(f) => f(self.handle),
                None => 0,
            }
        }
    }

    /// Get surface properties
    fn get_properties(&self) -> Option<&ivi_layout_surface_properties> {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return None;
            }

            let get_props_fn = (*self.api.api).get_properties_of_surface?;
            let props = get_props_fn(self.handle);

            if props.is_null() {
                None
            } else {
                Some(&*props)
            }
        }
    }

    /// Get surface position (destination rectangle position)
    pub fn get_position(&self) -> (i32, i32) {
        self.get_properties()
            .map(|props| (props.dest_x, props.dest_y))
            .unwrap_or((0, 0))
    }

    /// Set surface position (destination rectangle)
    pub fn set_position(&mut self, x: i32, y: i32) -> Result<(), String> {
        // Validate position
        crate::controller::validation::validate_position(x, y).map_err(|e| e.to_string())?;

        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return Err("Invalid API or surface handle".to_string());
            }

            if let Some(set_dest_fn) = (*self.api.api).surface_set_destination_rectangle {
                let (_, _, width, height) = self.get_size_full();
                set_dest_fn(self.handle, x, y, width, height);
                Ok(())
            } else {
                Err("surface_set_destination_rectangle function not available".to_string())
            }
        }
    }

    /// Get surface size (destination rectangle size)
    pub fn get_size(&self) -> (i32, i32) {
        self.get_properties()
            .map(|props| (props.dest_width, props.dest_height))
            .unwrap_or((0, 0))
    }

    /// Get the original buffer size of the surface
    pub fn get_orig_size(&self) -> (i32, i32, i32) {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return (0, 0, 0);
            }
            match (*self.api.api).surface_get_size {
                Some(f) => {
                    let mut width = 0;
                    let mut height = 0;
                    let mut stride = 0;
                    f(self.handle, &mut width, &mut height, &mut stride);
                    (width, height, stride)
                }
                None => (0, 0, 0),
            }
        }
    }

    /// Get surface source rectangle position
    pub fn get_source_position(&self) -> (i32, i32) {
        self.get_properties()
            .map(|props| (props.source_x, props.source_y))
            .unwrap_or((0, 0))
    }

    /// Get surface source rectangle size
    pub fn get_source_size(&self) -> (i32, i32) {
        self.get_properties()
            .map(|props| (props.source_width, props.source_height))
            .unwrap_or((0, 0))
    }

    /// Get full size information (x, y, width, height)
    fn get_size_full(&self) -> (i32, i32, i32, i32) {
        self.get_properties()
            .map(|props| {
                (
                    props.dest_x,
                    props.dest_y,
                    props.dest_width,
                    props.dest_height,
                )
            })
            .unwrap_or((0, 0, 0, 0))
    }

    /// Set surface size (destination rectangle)
    pub fn set_size(&mut self, width: i32, height: i32) -> Result<(), String> {
        // Validate size
        crate::controller::validation::validate_size(width, height).map_err(|e| e.to_string())?;

        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return Err("Invalid API or surface handle".to_string());
            }

            if let Some(set_dest_fn) = (*self.api.api).surface_set_destination_rectangle {
                let (x, y, _, _) = self.get_size_full();
                set_dest_fn(self.handle, x, y, width, height);
                Ok(())
            } else {
                Err("surface_set_destination_rectangle function not available".to_string())
            }
        }
    }

    /// Set surface destination rectangle (position and size)
    pub fn set_destination_rectangle(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<(), String> {
        // Validate position and size
        crate::controller::validation::validate_position(x, y).map_err(|e| e.to_string())?;
        crate::controller::validation::validate_size(width, height).map_err(|e| e.to_string())?;

        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return Err("Invalid API or surface handle".to_string());
            }

            if let Some(set_dest_fn) = (*self.api.api).surface_set_destination_rectangle {
                set_dest_fn(self.handle, x, y, width, height);
                Ok(())
            } else {
                Err("surface_set_destination_rectangle function not available".to_string())
            }
        }
    }

    /// Get surface source rectangle
    pub fn get_source_rectangle(&self) -> (i32, i32, i32, i32) {
        self.get_properties()
            .map(|props| {
                (
                    props.source_x,
                    props.source_y,
                    props.source_width,
                    props.source_height,
                )
            })
            .unwrap_or((0, 0, 0, 0))
    }

    /// Set surface source rectangle
    pub fn set_source_rectangle(&mut self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return;
            }

            if let Some(set_src_fn) = (*self.api.api).surface_set_source_rectangle {
                set_src_fn(self.handle, x, y, width, height);
            }
        }
    }

    /// Get surface visibility
    pub fn get_visibility(&self) -> bool {
        self.get_properties()
            .map(|props| props.visibility)
            .unwrap_or(false)
    }

    /// Get surface event mask (what changed)
    pub fn get_event_mask(&self) -> u32 {
        self.get_properties()
            .map(|props| props.event_mask)
            .unwrap_or(0)
    }

    /// Set surface visibility
    pub fn set_visibility(&mut self, visible: bool) {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return;
            }

            if let Some(set_vis_fn) = (*self.api.api).surface_set_visibility {
                set_vis_fn(self.handle, visible);
            }
        }
    }

    /// Get surface opacity (returns value in range 0.0 to 1.0)
    pub fn get_opacity(&self) -> f32 {
        self.get_properties()
            .map(|props| {
                // wl_fixed_t is a 24.8 fixed point number
                // Convert to float by dividing by 256.0
                let fixed_val = props.opacity as f64;
                (fixed_val / 256.0) as f32
            })
            .unwrap_or(1.0)
    }

    /// Set surface opacity (value should be in range 0.0 to 1.0)
    pub fn set_opacity(&mut self, opacity: f32) -> Result<(), String> {
        // Validate opacity
        crate::controller::validation::validate_opacity(opacity).map_err(|e| e.to_string())?;

        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return Err("Invalid API or surface handle".to_string());
            }

            if let Some(set_opacity_fn) = (*self.api.api).surface_set_opacity {
                // Convert float to wl_fixed_t (24.8 fixed point)
                let fixed_opacity = (opacity * 256.0) as i32;
                let result = set_opacity_fn(self.handle, fixed_opacity);

                if result == IVI_SUCCEEDED {
                    Ok(())
                } else {
                    Err("Failed to set opacity".to_string())
                }
            } else {
                Err("surface_set_opacity function not available".to_string())
            }
        }
    }

    /// Get surface orientation
    pub fn get_orientation(&self) -> Orientation {
        self.get_properties()
            .map(|props| Orientation::from_wl_transform(props.orientation))
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
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return Err("Invalid API or surface handle".to_string());
            }

            if let Some(activate_fn) = (*self.api.api).surface_activate {
                activate_fn(self.handle);
                Ok(())
            } else {
                Err("surface_activate function not available".to_string())
            }
        }
    }

    /// Set pointer focus to this surface
    /// Note: The IVI layout API doesn't separate keyboard and pointer focus,
    /// so this activates the surface which sets both keyboard and pointer focus
    pub fn set_pointer_focus(&mut self) -> Result<(), String> {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return Err("Invalid API or surface handle".to_string());
            }

            if let Some(activate_fn) = (*self.api.api).surface_activate {
                activate_fn(self.handle);
                Ok(())
            } else {
                Err("surface_activate function not available".to_string())
            }
        }
    }

    /// Activate the surface (set input focus)
    /// This sets both keyboard and pointer focus
    pub fn activate(&mut self) {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return;
            }

            if let Some(activate_fn) = (*self.api.api).surface_activate {
                activate_fn(self.handle);
            }
        }
    }

    /// Check if the surface is active (has focus)
    pub fn is_active(&self) -> bool {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return false;
            }

            match (*self.api.api).surface_is_active {
                Some(f) => f(self.handle),
                None => false,
            }
        }
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

        // TODO: Implement z-order setting through layer render order
        // This requires layer context and reordering surfaces within the layer
        Ok(())
    }

    /// Get the raw surface handle (for internal use)
    #[allow(dead_code)]
    pub(crate) fn handle(&self) -> *mut ivi_layout_surface {
        self.handle
    }
}

/// Safe wrapper around an IVI layer
pub struct IviLayer {
    handle: *mut ivi_layout_layer,
    api: Arc<IviLayoutApi>,
}

// Safety: IVI layers are thread-safe as per Weston's design
unsafe impl Send for IviLayer {}
unsafe impl Sync for IviLayer {}

impl IviLayer {
    /// Get the layer ID
    pub fn get_id(&self) -> u32 {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return 0;
            }
            match (*self.api.api).get_id_of_layer {
                Some(f) => f(self.handle),
                None => 0,
            }
        }
    }

    /// Get layer properties
    fn get_properties(&self) -> Option<&ivi_layout_layer_properties> {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return None;
            }

            let get_props_fn = (*self.api.api).get_properties_of_layer?;
            let props = get_props_fn(self.handle);

            if props.is_null() {
                None
            } else {
                Some(&*props)
            }
        }
    }

    /// Get layer visibility
    pub fn get_visibility(&self) -> bool {
        self.get_properties()
            .map(|props| props.visibility)
            .unwrap_or(false)
    }

    /// Get layer event mask (what changed)
    pub fn get_event_mask(&self) -> u32 {
        self.get_properties()
            .map(|props| props.event_mask)
            .unwrap_or(0)
    }

    /// Set layer visibility
    pub fn set_visibility(&mut self, visible: bool) {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return;
            }

            if let Some(set_vis_fn) = (*self.api.api).layer_set_visibility {
                set_vis_fn(self.handle, visible);
            }
        }
    }

    /// Get layer opacity
    pub fn get_opacity(&self) -> f32 {
        self.get_properties()
            .map(|props| {
                let fixed_val = props.opacity as f64;
                (fixed_val / 256.0) as f32
            })
            .unwrap_or(1.0)
    }

    /// Set layer opacity
    pub fn set_opacity(&mut self, opacity: f32) -> Result<(), String> {
        // Validate opacity
        crate::controller::validation::validate_opacity(opacity).map_err(|e| e.to_string())?;

        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return Err("Invalid API or layer handle".to_string());
            }

            if let Some(set_opacity_fn) = (*self.api.api).layer_set_opacity {
                let fixed_opacity = (opacity * 256.0) as i32;
                let result = set_opacity_fn(self.handle, fixed_opacity);

                if result == IVI_SUCCEEDED {
                    Ok(())
                } else {
                    Err("Failed to set opacity".to_string())
                }
            } else {
                Err("layer_set_opacity function not available".to_string())
            }
        }
    }

    /// Add surface to layer
    pub fn add_surface(&mut self, surface: &IviSurface) {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return;
            }

            if let Some(add_surface_fn) = (*self.api.api).layer_add_surface {
                add_surface_fn(self.handle, surface.handle);
            }
        }
    }

    /// Remove surface from layer
    pub fn remove_surface(&mut self, surface: &IviSurface) {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return;
            }

            if let Some(remove_surface_fn) = (*self.api.api).layer_remove_surface {
                remove_surface_fn(self.handle, surface.handle);
            }
        }
    }

    /// Get surfaces on this layer
    pub fn get_surfaces(&self) -> Vec<IviSurface> {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return Vec::new();
            }

            let get_surfaces_fn = match (*self.api.api).get_surfaces_on_layer {
                Some(f) => f,
                None => return Vec::new(),
            };

            let mut length: i32 = 0;
            let mut array: *mut *mut ivi_layout_surface = ptr::null_mut();

            get_surfaces_fn(self.handle, &mut length, &mut array);

            if array.is_null() || length <= 0 {
                return Vec::new();
            }

            let mut surfaces = Vec::new();

            for i in 0..length as isize {
                let handle = *array.offset(i);
                if !handle.is_null() {
                    surfaces.push(IviSurface {
                        handle,
                        api: self.api.clone(),
                    });
                }
            }

            surfaces
        }
    }

    /// Set the render order of surfaces in this layer
    pub fn set_render_order(&mut self, surfaces: &[&IviSurface]) {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return;
            }

            if let Some(set_order_fn) = (*self.api.api).layer_set_render_order {
                let handles: Vec<*mut ivi_layout_surface> =
                    surfaces.iter().map(|s| s.handle).collect();

                set_order_fn(
                    self.handle,
                    handles.as_ptr() as *mut *mut ivi_layout_surface,
                    handles.len() as i32,
                );
            }
        }
    }

    /// Destroy this layer
    pub fn destroy(self) {
        unsafe {
            if self.api.api.is_null() || self.handle.is_null() {
                return;
            }

            if let Some(destroy_fn) = (*self.api.api).layer_destroy {
                destroy_fn(self.handle);
            }
        }
    }

    /// Get the raw layer handle (for internal use)
    #[allow(dead_code)]
    pub(crate) fn handle(&self) -> *mut ivi_layout_layer {
        self.handle
    }
}
