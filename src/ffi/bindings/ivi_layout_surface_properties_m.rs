use super::*;

pub enum IviLayoutSurfaceType {
    Ivi,
    Desktop,
    InputPanel,
}

impl From<ivi_layout_surface_type> for IviLayoutSurfaceType {
    fn from(value: ivi_layout_surface_type) -> Self {
        match value {
            0 => IviLayoutSurfaceType::Ivi,
            1 => IviLayoutSurfaceType::Desktop,
            2 => IviLayoutSurfaceType::InputPanel,
            _ => IviLayoutSurfaceType::Ivi, // Default to Ivi for unknown values
        }
    }
}

impl From<IviLayoutSurfaceType> for ivi_layout_surface_type {
    fn from(value: IviLayoutSurfaceType) -> Self {
        match value {
            IviLayoutSurfaceType::Ivi => 0,
            IviLayoutSurfaceType::Desktop => 1,
            IviLayoutSurfaceType::InputPanel => 2,
        }
    }
}

pub struct IviLayoutSurfaceProperties {
    handle: *mut ivi_layout_surface_properties,
}

impl IviLayoutSurfaceProperties {
    pub fn from(props_ptr: *const ivi_layout_surface_properties) -> Option<Self> {
        if props_ptr.is_null() {
            return None;
        }

        Some(IviLayoutSurfaceProperties {
            handle: props_ptr as *mut ivi_layout_surface_properties,
        })
    }

    pub fn opacity(&self) -> f32 {
        unsafe { wl_fixed_t_to_f32((*self.handle).opacity) }
    }

    pub fn source_rectangle(&self) -> Rectangle {
        unsafe {
            Rectangle {
                x: (*self.handle).source_x,
                y: (*self.handle).source_y,
                width: (*self.handle).source_width,
                height: (*self.handle).source_height,
            }
        }
    }

    pub fn start_rectangle(&self) -> Rectangle {
        unsafe {
            Rectangle {
                x: (*self.handle).start_x,
                y: (*self.handle).start_y,
                width: (*self.handle).start_width,
                height: (*self.handle).start_height,
            }
        }
    }

    pub fn destination_rectangle(&self) -> Rectangle {
        unsafe {
            Rectangle {
                x: (*self.handle).dest_x,
                y: (*self.handle).dest_y,
                width: (*self.handle).dest_width,
                height: (*self.handle).dest_height,
            }
        }
    }

    pub fn orientation(&self) -> Orientation {
        unsafe { (*self.handle).orientation.into() }
    }

    pub fn visibility(&self) -> bool {
        unsafe { (*self.handle).visibility }
    }

    pub fn transition_type(&self) -> i32 {
        unsafe { (*self.handle).transition_type }
    }

    pub fn transition_duration(&self) -> u32 {
        unsafe { (*self.handle).transition_duration }
    }

    pub fn event_mask(&self) -> u32 {
        unsafe { (*self.handle).event_mask }
    }

    pub fn surface_type(&self) -> IviLayoutSurfaceType {
        unsafe { (*self.handle).surface_type.into() }
    }
}
