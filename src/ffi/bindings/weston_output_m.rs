use super::*;

pub struct WestonOutput {
    handle: *mut weston_output,
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

    pub fn get_name(&self) -> Option<String> {
        unsafe {
            if self.handle.is_null() {
                return None;
            }

            let name_ptr = (*self.handle).name;
            if name_ptr.is_null() {
                None
            } else {
                let c_str = std::ffi::CStr::from_ptr(name_ptr);
                c_str.to_str().ok().map(|s| s.to_string())
            }
        }
    }
}
