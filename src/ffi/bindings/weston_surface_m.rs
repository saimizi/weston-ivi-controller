use super::*;

pub struct WestonSurface {
    handle: *mut weston_surface,
}

impl WestonSurface {
    pub fn from(surface: *mut weston_surface) -> Option<Self> {
        if surface.is_null() {
            None
        } else {
            Some(WestonSurface { handle: surface })
        }
    }

    pub fn handle(&self) -> *mut weston_surface {
        self.handle
    }

    pub fn width(&self) -> i32 {
        unsafe { (*self.handle).width }
    }

    pub fn height(&self) -> i32 {
        unsafe { (*self.handle).height }
    }
}
