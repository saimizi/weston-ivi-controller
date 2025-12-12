use super::ivi_layer::IviLayer;
use super::ivi_layout_get_api;
use super::ivi_layout_layer_properties_m::IviLayoutLayerProperties;
use super::ivi_layout_surface_properties_m::IviLayoutSurfaceProperties;
use super::ivi_surface::IviSurface;
use super::weston_output_m::WestonOutput;
use super::weston_surface_m::WestonSurface;
use super::IviLayoutTransitionType;
use super::*;
use crate::ffi::weston::weston_compositor;

pub struct SurfaceSize {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
}

pub struct IviLayoutApi {
    api: *const ivi_layout_interface,
}

// Safety: The IVI layout API is thread-safe as per Weston's design
unsafe impl Send for IviLayoutApi {}
unsafe impl Sync for IviLayoutApi {}

impl IviLayoutApi {
    pub fn from_raw(api: *const ivi_layout_interface) -> Option<Self> {
        if api.is_null() {
            None
        } else {
            Some(IviLayoutApi { api })
        }
    }

    pub fn new(compositor: *mut weston_compositor) -> Option<Self> {
        if compositor.is_null() {
            return None;
        }

        let api = ivi_layout_get_api(compositor);
        if api.is_null() {
            None
        } else {
            Some(IviLayoutApi { api })
        }
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

    /// Rebuild view list without applaying any new changes
    pub fn commit_current(&self) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let commit_current_fn = (*self.api)
                .commit_current
                .ok_or("commit_current function is null")?;
            let result = commit_current_fn();

            if result == IVI_SUCCEEDED {
                Ok(())
            } else {
                Err("Failed to commit current state")
            }
        }
    }

    /// Add a listener for notification when ivi surface is created
    pub fn add_listener_create_surface(
        &self,
        listener: *mut wl_listener,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let add_listener_create_surface_fn = (*self.api)
                .add_listener_create_surface
                .ok_or("add_surface_created_listener function is null")?;
            add_listener_create_surface_fn(listener);

            Ok(())
        }
    }

    /// Add a listener for notification when ivi surface is removed
    pub fn add_listener_remove_surface(
        &self,
        listener: *mut wl_listener,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let add_listener_remove_surface_fn = (*self.api)
                .add_listener_remove_surface
                .ok_or("add_surface_created_listener function is null")?;
            add_listener_remove_surface_fn(listener);

            Ok(())
        }
    }

    /// Add a listener of notification when ivi surface is configured
    pub fn add_listener_configure_surface(
        &self,
        listener: *mut wl_listener,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let add_listener_configure_surface_fn = (*self.api)
                .add_listener_configure_surface
                .ok_or("add_listener_configure_surface function is null")?;
            add_listener_configure_surface_fn(listener);

            Ok(())
        }
    }

    /// Add listener for notification when desktop surface is configured
    pub fn add_listener_configure_desktop_surface(
        &self,
        listener: *mut wl_listener,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let add_listener_configure_desktop_surface_fn = (*self.api)
                .add_listener_configure_desktop_surface
                .ok_or("add_listener_configure_desktop_surface function is null")?;
            add_listener_configure_desktop_surface_fn(listener);

            Ok(())
        }
    }

    /// Get all ivi surfaces which are currently registered and managed
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
            let mut array: *mut *mut ivi_layout_surface = std::ptr::null_mut();
            get_surfaces_fn(&mut length, &mut array);

            if array.is_null() || length <= 0 {
                return Vec::new();
            }

            let mut surfaces = Vec::new();

            for i in 0..length as isize {
                let handle = *array.offset(i);
                if !handle.is_null() {
                    let ivi_api = Arc::new(Self { api: self.api });
                    if let Some(surface) = IviSurface::new(handle, ivi_api) {
                        surfaces.push(surface);
                    }
                }
            }

            surfaces
        }
    }

    /// Get ID of ivi surface
    pub fn get_id_of_surface(&self, surface: &IviSurface) -> u32 {
        unsafe {
            match (*self.api).get_id_of_surface {
                Some(f) => f(surface.handle()),
                None => 0,
            }
        }
    }

    /// Get a surface by its ID
    pub fn get_surface_from_id(&self, id: u32) -> Option<IviSurface> {
        unsafe {
            let get_surface_fn = (*self.api).get_surface_from_id?;
            IviSurface::new(get_surface_fn(id), Arc::new(Self { api: self.api }))
        }
    }

    /// Get surface properties
    pub fn get_properties_of_surface(
        &self,
        surface: &IviSurface,
    ) -> Option<IviLayoutSurfaceProperties> {
        unsafe {
            let get_props_fn = (*self.api).get_properties_of_surface?;
            let props = get_props_fn(surface.handle());
            IviLayoutSurfaceProperties::from(props)
        }
    }

    /// Get surfaces on a layer
    pub fn get_surfaces_on_layer(&self, layer: &IviLayer) -> Vec<IviSurface> {
        unsafe {
            let get_surfaces_fn = match (*self.api).get_surfaces_on_layer {
                Some(f) => f,
                None => return Vec::new(),
            };

            let mut length: i32 = 0;
            let mut array: *mut *mut ivi_layout_surface = std::ptr::null_mut();

            get_surfaces_fn(layer.handle(), &mut length, &mut array);

            if array.is_null() || length <= 0 {
                return Vec::new();
            }

            let mut surfaces = Vec::new();
            for i in 0..length as isize {
                let handle = *array.offset(i);
                if let Some(surface) = IviSurface::new(handle, Arc::new(Self { api: self.api })) {
                    surfaces.push(surface);
                }
            }

            surfaces
        }
    }

    /// Set the visibility of a ivi surface
    pub fn surface_set_visibility(
        &self,
        surface: &IviSurface,
        visible: bool,
    ) -> Result<(), &'static str> {
        unsafe {
            let set_visibility_fn = (*self.api)
                .surface_set_visibility
                .ok_or("surface_set_visibility function is null")?;
            set_visibility_fn(surface.handle(), visible);
        }
        Ok(())
    }

    /// Set the opacity of a ivi surface
    pub fn surface_set_opacity(
        &self,
        surface: &IviSurface,
        opacity: f32,
    ) -> Result<(), &'static str> {
        unsafe {
            let set_opacity_fn = (*self.api)
                .surface_set_opacity
                .ok_or("surface_set_opacity function is null")?;
            let ret = set_opacity_fn(surface.handle(), f32_to_wl_fixed_t(opacity));
            if ret != IVI_SUCCEEDED {
                return Err("Failed to set surface opacity");
            }
        }
        Ok(())
    }

    /// Set the area of a ivi surface used for rendering
    pub fn surface_set_source_rectangle(
        &self,
        surface: &IviSurface,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<(), &'static str> {
        unsafe {
            let set_source_rectangle_fn = (*self.api)
                .surface_set_source_rectangle
                .ok_or("surface_set_source_rectangle function is null")?;
            set_source_rectangle_fn(surface.handle(), x, y, width, height);
        }
        Ok(())
    }

    /// Set the destination area of a ivi surface on the output
    pub fn surface_set_destination_rectangle(
        &self,
        surface: &IviSurface,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<(), &'static str> {
        unsafe {
            let set_destination_rectangle_fn = (*self.api)
                .surface_set_destination_rectangle
                .ok_or("surface_set_destination_rectangle function is null")?;
            set_destination_rectangle_fn(surface.handle(), x, y, width, height);
        }
        Ok(())
    }

    /// Add a lisener for notification when surface properties are changed
    pub fn surface_add_listener(
        &self,
        surface: &IviSurface,
        listener: *mut wl_listener,
    ) -> Result<(), &'static str> {
        unsafe {
            let add_listener_fn = (*self.api)
                .surface_add_listener
                .ok_or("surface_add_listener_properties_changed function is null")?;
            add_listener_fn(surface.handle(), listener);
        }
        Ok(())
    }

    /// Get weston surface of ivi surface
    pub fn surface_get_weston_surface(&self, surface: &IviSurface) -> Option<WestonSurface> {
        unsafe {
            let get_weston_surface_fn = (*self.api).surface_get_weston_surface?;
            WestonSurface::from(get_weston_surface_fn(surface.handle()))
        }
    }

    /// Set type of transition animation for a surface
    pub fn surface_set_transition(
        &self,
        surface: &IviSurface,
        transition_type: IviLayoutTransitionType,
        duration: u32,
    ) -> Result<(), &'static str> {
        unsafe {
            let set_transition_fn = (*self.api)
                .surface_set_transition
                .ok_or("surface_set_transition_type function is null")?;
            set_transition_fn(surface.handle(), transition_type.into(), duration);
        }
        Ok(())
    }

    /// Set duration of transition animation for a surface
    pub fn surface_set_transition_duration(
        &self,
        surface: &IviSurface,
        duration: u32,
    ) -> Result<(), &'static str> {
        unsafe {
            let set_transition_duration_fn = (*self.api)
                .surface_set_transition_duration
                .ok_or("surface_set_transition_duration function is null")?;
            set_transition_duration_fn(surface.handle(), duration);
        }
        Ok(())
    }

    /// Set ID of ivi layout surface
    pub fn surface_set_id(&self, surface: &IviSurface, id: u32) -> Result<(), &'static str> {
        unsafe {
            let set_id_fn = (*self.api)
                .surface_set_id
                .ok_or("surface_set_id function is null")?;
            let ret = set_id_fn(surface.handle(), id);
            if ret != IVI_SUCCEEDED {
                return Err("Failed to set surface ID");
            }
        }
        Ok(())
    }

    /// Activate ivi surface
    pub fn surface_activate(&self, surface: &IviSurface) -> Result<(), &'static str> {
        unsafe {
            let activate_fn = (*self.api)
                .surface_activate
                .ok_or("surface_activate function is null")?;
            activate_fn(surface.handle());
        }
        Ok(())
    }

    /// Check if ivi surface is active
    pub fn surface_is_active(&self, surface: &IviSurface) -> bool {
        unsafe {
            let is_active_fn = match (*self.api).surface_is_active {
                Some(f) => f,
                None => return false,
            };
            is_active_fn(surface.handle())
        }
    }

    /// Add a listener for notification when layer is created
    pub fn add_listener_create_layer(
        &self,
        listener: *mut wl_listener,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let add_listener_create_layer_fn = (*self.api)
                .add_listener_create_layer
                .ok_or("add_listener_create_layer function is null")?;
            add_listener_create_layer_fn(listener);

            Ok(())
        }
    }

    /// Add a listener for notification when layer is removed
    pub fn add_listener_remove_layer(
        &self,
        listener: *mut wl_listener,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let add_listener_remove_layer_fn = (*self.api)
                .add_listener_remove_layer
                .ok_or("add_listener_remove_layer function is null")?;
            add_listener_remove_layer_fn(listener);

            Ok(())
        }
    }

    /// Create a ivi layer
    pub fn layer_create_with_dimension(
        &self,
        id: u32,
        width: i32,
        height: i32,
    ) -> Result<IviLayer, &'static str> {
        unsafe {
            let create_layer_fn = (*self.api)
                .layer_create_with_dimension
                .ok_or("layer_create_with_dimension function is null")?;
            let handle = create_layer_fn(id, width, height);
            IviLayer::new(handle, Arc::new(Self { api: self.api })).ok_or("Failed to create layer")
        }
    }

    /// Delete a ivi layer
    pub fn layer_destroy(&self, layer: &IviLayer) -> Result<(), &'static str> {
        unsafe {
            let destroy_layer_fn = (*self.api)
                .layer_destroy
                .ok_or("layer_destroy function is null")?;
            destroy_layer_fn(layer.handle());
        }
        Ok(())
    }

    /// Get layers
    pub fn get_layers(&self) -> Result<Vec<IviLayer>, &'static str> {
        unsafe {
            let get_layers_fn = (*self.api)
                .get_layers
                .ok_or("get_layers function is null")?;

            let mut length: i32 = 0;
            let mut array: *mut *mut ivi_layout_layer = std::ptr::null_mut();

            get_layers_fn(&mut length, &mut array);

            if array.is_null() || length <= 0 {
                return Ok(Vec::new());
            }

            let api = Arc::new(Self { api: self.api });
            let mut layers = Vec::new();

            for i in 0..length as isize {
                let handle = *array.offset(i);
                if let Some(layer) = IviLayer::new(handle, api.clone()) {
                    layers.push(layer);
                }
            }

            Ok(layers)
        }
    }

    /// Get ID of layer
    pub fn get_id_of_layer(&self, layer: &IviLayer) -> u32 {
        unsafe {
            match (*self.api).get_id_of_layer {
                Some(f) => f(layer.handle()),
                None => 0,
            }
        }
    }

    /// Get a layer by its ID
    pub fn get_layer_from_id(&self, id: u32) -> Option<IviLayer> {
        unsafe {
            let get_layer_fn = (*self.api).get_layer_from_id?;
            let handle = get_layer_fn(id);
            IviLayer::new(handle, Arc::new(Self { api: self.api }))
        }
    }

    /// Get ivi layer properties
    pub fn get_properties_of_layer(&self, layer: &IviLayer) -> Option<IviLayoutLayerProperties> {
        unsafe {
            let get_props_fn = (*self.api).get_properties_of_layer?;
            let props = get_props_fn(layer.handle());
            IviLayoutLayerProperties::from(props)
        }
    }

    /// Get all ivi layer under the given ivi surface which means all the ivi layers the ivi-surface was added to
    pub fn get_layers_under_surface(
        &self,
        surface: &IviSurface,
    ) -> Result<Vec<IviLayer>, &'static str> {
        unsafe {
            let get_layers_fn = (*self.api)
                .get_layers_under_surface
                .ok_or("get_layers_under_surface function is null")?;

            let mut length: i32 = 0;
            let mut array: *mut *mut ivi_layout_layer = std::ptr::null_mut();

            get_layers_fn(surface.handle(), &mut length, &mut array);

            if array.is_null() || length <= 0 {
                return Ok(Vec::new());
            }

            let api = Arc::new(Self { api: self.api });
            let mut layers = Vec::new();

            for i in 0..length as isize {
                let handle = *array.offset(i);
                if let Some(layer) = IviLayer::new(handle, api.clone()) {
                    layers.push(layer);
                }
            }

            Ok(layers)
        }
    }

    /// Get all layers of the given weston output
    pub fn get_layers_on_screen(
        &self,
        output: *mut weston_output,
    ) -> Result<Vec<IviLayer>, &'static str> {
        unsafe {
            let get_layers_fn = (*self.api)
                .get_layers_on_screen
                .ok_or("get_layers_on_screen function is null")?;

            let mut length: i32 = 0;
            let mut array: *mut *mut ivi_layout_layer = std::ptr::null_mut();

            get_layers_fn(output, &mut length, &mut array);

            if array.is_null() || length <= 0 {
                return Ok(Vec::new());
            }

            let api = Arc::new(Self { api: self.api });
            let mut layers = Vec::new();

            for i in 0..length as isize {
                let handle = *array.offset(i);
                if let Some(layer) = IviLayer::new(handle, api.clone()) {
                    layers.push(layer);
                }
            }

            Ok(layers)
        }
    }

    /// Set the visibility of a ivi layer
    pub fn layer_set_visibility(
        &self,
        layer: &IviLayer,
        visible: bool,
    ) -> Result<(), &'static str> {
        unsafe {
            let set_visibility_fn = (*self.api)
                .layer_set_visibility
                .ok_or("layer_set_visibility function is null")?;
            set_visibility_fn(layer.handle(), visible);
        }
        Ok(())
    }

    /// Set the opacity of a ivi layer
    pub fn layer_set_opacity(&self, layer: &IviLayer, opacity: f32) -> Result<(), &'static str> {
        unsafe {
            let set_opacity_fn = (*self.api)
                .layer_set_opacity
                .ok_or("layer_set_opacity function is null")?;
            let ret = set_opacity_fn(layer.handle(), f32_to_wl_fixed_t(opacity));
            if ret != IVI_SUCCEEDED {
                return Err("Failed to set layer opacity");
            }
        }
        Ok(())
    }

    /// Set the area of a ivi layer used for rendering
    pub fn layer_set_source_rectangle(
        &self,
        layer: &IviLayer,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<(), &'static str> {
        unsafe {
            let set_source_rectangle_fn = (*self.api)
                .layer_set_source_rectangle
                .ok_or("layer_set_source_rectangle function is null")?;
            set_source_rectangle_fn(layer.handle(), x, y, width, height);
        }
        Ok(())
    }

    /// Set the destination area of a ivi layer on the output
    pub fn layer_set_destination_rectangle(
        &self,
        layer: &IviLayer,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<(), &'static str> {
        unsafe {
            let set_destination_rectangle_fn = (*self.api)
                .layer_set_destination_rectangle
                .ok_or("layer_set_destination_rectangle function is null")?;
            set_destination_rectangle_fn(layer.handle(), x, y, width, height);
        }
        Ok(())
    }

    /// Add a ivi surface to a ivi layer
    pub fn layer_add_surface(
        &self,
        layer: &IviLayer,
        surface: &IviSurface,
    ) -> Result<(), &'static str> {
        unsafe {
            let add_surface_fn = (*self.api)
                .layer_add_surface
                .ok_or("layer_add_surface function is null")?;
            add_surface_fn(layer.handle(), surface.handle());
        }
        Ok(())
    }

    /// Remove a ivi surface from a ivi layer
    pub fn layer_remove_surface(
        &self,
        layer: &IviLayer,
        surface: &IviSurface,
    ) -> Result<(), &'static str> {
        unsafe {
            let remove_surface_fn = (*self.api)
                .layer_remove_surface
                .ok_or("layer_remove_surface function is null")?;
            remove_surface_fn(layer.handle(), surface.handle());
        }
        Ok(())
    }

    /// Set render order of surfaces in a layer
    pub fn layer_set_render_order(
        &self,
        layer: &IviLayer,
        surfaces: &[&IviSurface],
    ) -> Result<(), &'static str> {
        unsafe {
            let set_order_fn = (*self.api)
                .layer_set_render_order
                .ok_or("layer_set_render_order function is null")?;
            let handles: Vec<*mut ivi_layout_surface> =
                surfaces.iter().map(|s| s.handle()).collect();
            set_order_fn(
                layer.handle(),
                handles.as_ptr() as *mut *mut ivi_layout_surface,
                handles.len() as i32,
            );
        }
        Ok(())
    }

    /// Add a listener to listen property changes of ivi layer when a property of the ivi layer is changed.
    pub fn layer_add_listener(
        &self,
        layer: &IviLayer,
        listener: *mut wl_listener,
    ) -> Result<(), &'static str> {
        unsafe {
            let add_listener_fn = (*self.api)
                .layer_add_listener
                .ok_or("layer_add_listener_properties_changed function is null")?;
            add_listener_fn(layer.handle(), listener);
        }
        Ok(())
    }

    /// Set type of transition animation for a layer
    pub fn layer_set_transition(
        &self,
        layer: &IviLayer,
        transition_type: IviLayoutTransitionType,
        duration: u32,
    ) -> Result<(), &'static str> {
        unsafe {
            let set_transition_fn = (*self.api)
                .layer_set_transition
                .ok_or("layer_set_transition_type function is null")?;
            set_transition_fn(layer.handle(), transition_type.into(), duration);
        }
        Ok(())
    }

    /// Get the weston outputs under the given ivi layer
    pub fn get_screens_under_layer(
        &self,
        layer: &IviLayer,
    ) -> Result<Vec<WestonOutput>, &'static str> {
        unsafe {
            let get_screens_fn = (*self.api)
                .get_screens_under_layer
                .ok_or("get_screens_under_layer function is null")?;

            let mut length: i32 = 0;
            let mut array: *mut *mut weston_output = std::ptr::null_mut();

            get_screens_fn(layer.handle(), &mut length, &mut array);

            if array.is_null() || length <= 0 {
                return Ok(Vec::new());
            }

            let mut outputs = Vec::new();

            for i in 0..length as isize {
                let handle = *array.offset(i);
                if let Some(output) = WestonOutput::from(handle) {
                    outputs.push(output);
                }
            }

            Ok(outputs)
        }
    }

    /// Add a ivi layer to a weston output
    pub fn screen_add_layer(
        &self,
        output: WestonOutput,
        layer: &IviLayer,
    ) -> Result<(), &'static str> {
        unsafe {
            let add_layer_fn = (*self.api)
                .screen_add_layer
                .ok_or("screen_add_layer function is null")?;
            add_layer_fn(output.into(), layer.handle());
        }
        Ok(())
    }

    /// Set render order of layers on a weston output
    pub fn screen_set_render_order(
        &self,
        output: WestonOutput,
        layers: &[&IviLayer],
    ) -> Result<(), &'static str> {
        unsafe {
            let set_order_fn = (*self.api)
                .screen_set_render_order
                .ok_or("screen_set_layer_order function is null")?;
            let handles: Vec<*mut ivi_layout_layer> = layers.iter().map(|l| l.handle()).collect();
            set_order_fn(
                output.into(),
                handles.as_ptr() as *mut *mut ivi_layout_layer,
                handles.len() as i32,
            );
        }
        Ok(())
    }

    /// Transition animation for ivi layer
    pub fn transition_move_layer_cancel(&self, layer: &IviLayer) -> Result<(), &'static str> {
        unsafe {
            let cancel_transition_fn = (*self.api)
                .transition_move_layer_cancel
                .ok_or("transition_move_layer_cancel function is null")?;
            cancel_transition_fn(layer.handle());
        }
        Ok(())
    }

    /// Set layer fade animation information
    pub fn layer_set_fade_info(
        &self,
        layer: &IviLayer,
        is_fade_in: u32,
        start_alpha: f64,
        end_alpha: f64,
    ) -> Result<(), &'static str> {
        unsafe {
            let set_fade_info_fn = (*self.api)
                .layer_set_fade_info
                .ok_or("layer_set_fade_info function is null")?;
            set_fade_info_fn(layer.handle(), is_fade_in, start_alpha, end_alpha);
        }
        Ok(())
    }

    /// Surface size
    pub fn surface_get_size(&self, surface: &IviSurface) -> Result<SurfaceSize, &'static str> {
        unsafe {
            let get_size_fn = (*self.api)
                .surface_get_size
                .ok_or("surface_get_size function is null")?;

            let width: *mut i32 = std::ptr::null_mut();
            let height: *mut i32 = std::ptr::null_mut();
            let stride: *mut i32 = std::ptr::null_mut();

            get_size_fn(surface.handle(), width, height, stride);

            if width.is_null() || height.is_null() || stride.is_null() {
                return Err("Failed to get surface size");
            }
            Ok(SurfaceSize {
                width: *width,
                height: *height,
                stride: *stride,
            })
        }
    }

    /// Surface content dumping for debugging
    pub fn surface_dump(
        &self,
        surface: &WestonSurface,
        target: *mut std::ffi::c_void,
        size: usize,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<i32, &'static str> {
        unsafe {
            let dump_fn = (*self.api)
                .surface_dump
                .ok_or("surface_dump function is null")?;

            let ret = dump_fn(surface.handle(), target, size, x, y, width, height);

            if ret < 0 {
                return Err("Failed to dump surface content");
            }
            Ok(ret)
        }
    }

    /// Return the ivi surface or None
    pub fn get_surface(&self, surface: &WestonSurface) -> Option<IviSurface> {
        unsafe {
            let get_surface_fn = (*self.api).get_surface?;
            let handle = get_surface_fn(surface.handle());
            IviSurface::new(handle, Arc::new(Self { api: self.api }))
        }
    }

    /// Remove a ivi layer from a weston output
    pub fn screen_remove_layer(
        &self,
        output: WestonOutput,
        layer: &IviLayer,
    ) -> Result<(), &'static str> {
        unsafe {
            let remove_layer_fn = (*self.api)
                .screen_remove_layer
                .ok_or("screen_remove_layer function is null")?;
            remove_layer_fn(output.into(), layer.handle());
        }
        Ok(())
    }

    /// Add a shell destroy listener only once.
    pub fn shell_add_destroy_listener_once(
        &self,
        listener: *mut wl_listener,
        destroy_handler: wl_notify_func_t,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let add_destroy_listener_once_fn = (*self.api)
                .shell_add_destroy_listener_once
                .ok_or("shell_add_destroy_listener_once function is null")?;
            add_destroy_listener_once_fn(listener, destroy_handler);

            Ok(())
        }
    }

    /// Add a listener for notification when input panel surface is configured.
    pub fn add_listener_configure_input_panel_surface(
        &self,
        listener: *mut wl_listener,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let add_listener_configure_input_panel_surface_fn = (*self.api)
                .add_listener_configure_input_panel_surface
                .ok_or("add_listener_configure_input_panel_surface function is null")?;
            add_listener_configure_input_panel_surface_fn(listener);

            Ok(())
        }
    }

    /// Add a listener for notification when an iput panel surface should be show
    pub fn add_listener_show_input_panel(
        &self,
        listener: *mut wl_listener,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let add_listener_show_input_panel_fn = (*self.api)
                .add_listener_show_input_panel
                .ok_or("add_listener_show_input_panel_surface function is null")?;
            add_listener_show_input_panel_fn(listener);

            Ok(())
        }
    }

    /// Add a listener for notification when an input panel panel surface should be hidden
    pub fn add_listener_hide_input_panel(
        &self,
        listener: *mut wl_listener,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let add_listener_hide_input_panel_fn = (*self.api)
                .add_listener_hide_input_panel
                .ok_or("add_listener_hide_input_panel_surface function is null")?;
            add_listener_hide_input_panel_fn(listener);

            Ok(())
        }
    }

    /// Add a listener for notification when an input panel surface should be updated.
    pub fn add_listener_update_input_panel(
        &self,
        listener: *mut wl_listener,
    ) -> Result<(), &'static str> {
        unsafe {
            if self.api.is_null() {
                return Err("IVI API pointer is null");
            }

            let add_listener_update_input_panel_fn = (*self.api)
                .add_listener_update_input_panel
                .ok_or("add_listener_update_input_panel_surface function is null")?;
            add_listener_update_input_panel_fn(listener);

            Ok(())
        }
    }
}
