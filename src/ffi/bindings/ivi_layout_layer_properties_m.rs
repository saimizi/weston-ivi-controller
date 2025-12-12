use super::*;

pub struct IviLayoutLayerProperties {
    handle: *mut ivi_layout_layer_properties,
}

impl IviLayoutLayerProperties {
    pub fn from(props_ptr: *const ivi_layout_layer_properties) -> Option<Self> {
        if props_ptr.is_null() {
            return None;
        }

        Some(IviLayoutLayerProperties {
            handle: props_ptr as *mut ivi_layout_layer_properties,
        })
    }

    pub fn opacity(&self) -> f32 {
        unsafe { wl_fixed_t_to_f32((*self.handle).opacity) }
    }

    pub fn source_rectangle(&self) -> Rectangle {
        unsafe {
            let prop = *self.handle;
            Rectangle {
                x: prop.source_x,
                y: prop.source_y,
                width: prop.source_width,
                height: prop.source_height,
            }
        }
    }

    pub fn destination_rectangle(&self) -> Rectangle {
        unsafe {
            let prop = *self.handle;
            Rectangle {
                x: prop.dest_x,
                y: prop.dest_y,
                width: prop.dest_width,
                height: prop.dest_height,
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

    pub fn start_alpha(&self) -> f64 {
        unsafe { (*self.handle).start_alpha }
    }

    pub fn end_alpha(&self) -> f64 {
        unsafe { (*self.handle).end_alpha }
    }

    pub fn is_fade_in_transition(&self) -> bool {
        unsafe { (*self.handle).is_fade_in != 0 }
    }

    pub fn event_mask(&self) -> u32 {
        unsafe { (*self.handle).event_mask }
    }
}
