use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WestonCoord {
    pub x: f64,
    pub y: f64,
}

impl From<weston_coord> for WestonCoord {
    fn from(coord: weston_coord) -> Self {
        WestonCoord {
            x: coord.x,
            y: coord.y,
        }
    }
}

impl From<WestonCoord> for weston_coord {
    fn from(coord: WestonCoord) -> Self {
        weston_coord {
            x: coord.x,
            y: coord.y,
        }
    }
}

impl From<weston_coord_global> for WestonCoord {
    fn from(coord: weston_coord_global) -> Self {
        WestonCoord {
            x: coord.c.x,
            y: coord.c.y,
        }
    }
}

impl From<WestonCoord> for weston_coord_global {
    fn from(coord: WestonCoord) -> Self {
        weston_coord_global {
            c: weston_coord {
                x: coord.x,
                y: coord.y,
            },
        }
    }
}

#[derive(Clone)]
pub struct WestonOutput {
    handle: *mut weston_output,
}

/// Screen information structure for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenInfo {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub coord_global: WestonCoord,
    pub transform: Orientation,
    pub enabled: bool,
    pub scale: i32,
}

impl From<WestonOutput> for ScreenInfo {
    fn from(output: WestonOutput) -> Self {
        ScreenInfo {
            name: output.name().unwrap_or_default(),
            width: output.width(),
            height: output.height(),
            coord_global: output.coord_global(),
            transform: output.transform(),
            enabled: output.is_enabled(),
            scale: output.scale(),
        }
    }
}

impl From<WestonOutput> for *mut weston_output {
    fn from(output: WestonOutput) -> Self {
        output.handle
    }
}

impl WestonOutput {
    pub fn from(handle: *mut weston_output) -> Option<Self> {
        if handle.is_null() {
            None
        } else {
            Some(WestonOutput { handle })
        }
    }

    pub fn name(&self) -> Option<String> {
        unsafe {
            let name_ptr = (*self.handle).name;
            let c_str = std::ffi::CStr::from_ptr(name_ptr);
            c_str.to_str().ok().map(|s| s.to_string())
        }
    }

    pub fn width(&self) -> i32 {
        unsafe { (*self.handle).width }
    }

    pub fn height(&self) -> i32 {
        unsafe { (*self.handle).height }
    }

    pub fn coord_global(&self) -> WestonCoord {
        unsafe { (*self.handle).pos.into() }
    }

    pub fn transform(&self) -> Orientation {
        unsafe { Orientation::from((*self.handle).transform) }
    }

    pub fn is_enabled(&self) -> bool {
        unsafe { (*self.handle).destroying == 0 }
    }

    pub fn scale(&self) -> i32 {
        unsafe { (*self.handle).scale }
    }

    pub fn current_scale(&self) -> i32 {
        unsafe { (*self.handle).current_scale }
    }

    pub fn id(&self) -> u32 {
        unsafe { (*self.handle).id }
    }
}
