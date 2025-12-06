use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=weston/ivi-shell/ivi-layout-export.h");
    println!("cargo:rerun-if-changed=weston/libweston/plugin-registry.h");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_path = out_path.join("ivi_bindings.rs");
    let weston_bindings_path = out_path.join("weston_bindings.rs");

    // Generate IVI layout bindings
    let ivi_result = bindgen::Builder::default()
        .header("weston/ivi-shell/ivi-layout-export.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("ivi_layout_.*")
        .allowlist_function("ivi_layout_.*")
        .allowlist_var("IVI_.*")
        .allowlist_type("wl_.*")
        .allowlist_type("weston_.*")
        .generate();

    // Generate Weston plugin API bindings
    let weston_result = bindgen::Builder::default()
        .header("weston/libweston/plugin-registry.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("weston_plugin_api_.*")
        .allowlist_type("weston_compositor")
        .generate();

    // Handle IVI bindings
    match ivi_result {
        Ok(bindings) => {
            bindings
                .write_to_file(&bindings_path)
                .expect("Couldn't write IVI bindings!");
        }
        Err(e) => {
            eprintln!("Warning: Failed to generate IVI bindings: {}", e);
            eprintln!("Creating stub IVI bindings file for development");

            // Create a more complete stub file that matches the IVI layout interface
            let stub = r#"
// Stub bindings - actual bindings require Weston headers
// This allows the project to compile for development purposes

use libc::c_void;

pub type wl_fixed_t = i32;

#[repr(C)]
pub struct weston_compositor {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct weston_surface {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct weston_output {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct wl_listener {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct ivi_layout_surface {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct ivi_layout_layer {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ivi_layout_surface_properties {
    pub opacity: wl_fixed_t,
    pub source_x: i32,
    pub source_y: i32,
    pub source_width: i32,
    pub source_height: i32,
    pub start_x: i32,
    pub start_y: i32,
    pub start_width: i32,
    pub start_height: i32,
    pub dest_x: i32,
    pub dest_y: i32,
    pub dest_width: i32,
    pub dest_height: i32,
    pub orientation: u32,
    pub visibility: bool,
    pub transition_type: i32,
    pub transition_duration: u32,
    pub event_mask: u32,
    pub surface_type: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ivi_layout_layer_properties {
    pub opacity: wl_fixed_t,
    pub source_x: i32,
    pub source_y: i32,
    pub source_width: i32,
    pub source_height: i32,
    pub dest_x: i32,
    pub dest_y: i32,
    pub dest_width: i32,
    pub dest_height: i32,
    pub orientation: u32,
    pub visibility: bool,
    pub transition_type: i32,
    pub transition_duration: u32,
    pub start_alpha: f64,
    pub end_alpha: f64,
    pub is_fade_in: u32,
    pub event_mask: u32,
}

#[repr(C)]
pub struct ivi_layout_interface {
    pub commit_changes: Option<unsafe extern "C" fn() -> i32>,
    pub commit_current: Option<unsafe extern "C" fn() -> i32>,
    pub add_listener_create_surface: Option<unsafe extern "C" fn(listener: *mut wl_listener)>,
    pub add_listener_remove_surface: Option<unsafe extern "C" fn(listener: *mut wl_listener)>,
    pub add_listener_configure_surface: Option<unsafe extern "C" fn(listener: *mut wl_listener)>,
    pub add_listener_configure_desktop_surface: Option<unsafe extern "C" fn(listener: *mut wl_listener)>,
    pub get_surfaces: Option<unsafe extern "C" fn(pLength: *mut i32, ppArray: *mut *mut *mut ivi_layout_surface)>,
    pub get_id_of_surface: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface) -> u32>,
    pub get_surface_from_id: Option<unsafe extern "C" fn(id_surface: u32) -> *mut ivi_layout_surface>,
    pub get_properties_of_surface: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface) -> *const ivi_layout_surface_properties>,
    pub get_surfaces_on_layer: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer, pLength: *mut i32, ppArray: *mut *mut *mut ivi_layout_surface)>,
    pub surface_set_visibility: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface, newVisibility: bool)>,
    pub surface_set_opacity: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface, opacity: wl_fixed_t) -> i32>,
    pub surface_set_source_rectangle: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface, x: i32, y: i32, width: i32, height: i32)>,
    pub surface_set_destination_rectangle: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface, x: i32, y: i32, width: i32, height: i32)>,
    pub surface_add_listener: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface, listener: *mut wl_listener)>,
    pub surface_get_weston_surface: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface) -> *mut weston_surface>,
    pub surface_set_transition: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface, type_: u32, duration: u32)>,
    pub surface_set_transition_duration: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface, duration: u32)>,
    pub surface_set_id: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface, id_surface: u32) -> i32>,
    pub surface_activate: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface)>,
    pub surface_is_active: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface) -> bool>,
    pub add_listener_create_layer: Option<unsafe extern "C" fn(listener: *mut wl_listener)>,
    pub add_listener_remove_layer: Option<unsafe extern "C" fn(listener: *mut wl_listener)>,
    pub layer_create_with_dimension: Option<unsafe extern "C" fn(id_layer: u32, width: i32, height: i32) -> *mut ivi_layout_layer>,
    pub layer_destroy: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer)>,
    pub get_layers: Option<unsafe extern "C" fn(pLength: *mut i32, ppArray: *mut *mut *mut ivi_layout_layer)>,
    pub get_id_of_layer: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer) -> u32>,
    pub get_layer_from_id: Option<unsafe extern "C" fn(id_layer: u32) -> *mut ivi_layout_layer>,
    pub get_properties_of_layer: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer) -> *const ivi_layout_layer_properties>,
    pub get_layers_under_surface: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface, pLength: *mut i32, ppArray: *mut *mut *mut ivi_layout_layer)>,
    pub get_layers_on_screen: Option<unsafe extern "C" fn(output: *mut weston_output, pLength: *mut i32, ppArray: *mut *mut *mut ivi_layout_layer)>,
    pub layer_set_visibility: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer, newVisibility: bool)>,
    pub layer_set_opacity: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer, opacity: wl_fixed_t) -> i32>,
    pub layer_set_source_rectangle: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer, x: i32, y: i32, width: i32, height: i32)>,
    pub layer_set_destination_rectangle: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer, x: i32, y: i32, width: i32, height: i32)>,
    pub layer_add_surface: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer, addsurf: *mut ivi_layout_surface)>,
    pub layer_remove_surface: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer, remsurf: *mut ivi_layout_surface)>,
    pub layer_set_render_order: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer, pSurface: *mut *mut ivi_layout_surface, number: i32)>,
    pub layer_add_listener: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer, listener: *mut wl_listener)>,
    pub layer_set_transition: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer, type_: u32, duration: u32)>,
    pub get_screens_under_layer: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer, pLength: *mut i32, ppArray: *mut *mut *mut weston_output)>,
    pub screen_add_layer: Option<unsafe extern "C" fn(output: *mut weston_output, addlayer: *mut ivi_layout_layer)>,
    pub screen_set_render_order: Option<unsafe extern "C" fn(output: *mut weston_output, pLayer: *mut *mut ivi_layout_layer, number: i32)>,
    pub transition_move_layer_cancel: Option<unsafe extern "C" fn(layer: *mut ivi_layout_layer)>,
    pub layer_set_fade_info: Option<unsafe extern "C" fn(ivilayer: *mut ivi_layout_layer, is_fade_in: u32, start_alpha: f64, end_alpha: f64)>,
    pub surface_get_size: Option<unsafe extern "C" fn(ivisurf: *mut ivi_layout_surface, width: *mut i32, height: *mut i32, stride: *mut i32)>,
    pub surface_dump: Option<unsafe extern "C" fn(surface: *mut weston_surface, target: *mut c_void, size: usize, x: i32, y: i32, width: i32, height: i32) -> i32>,
    pub get_surface: Option<unsafe extern "C" fn(surface: *mut weston_surface) -> *mut ivi_layout_surface>,
    pub screen_remove_layer: Option<unsafe extern "C" fn(output: *mut weston_output, removelayer: *mut ivi_layout_layer)>,
    pub shell_add_destroy_listener_once: Option<unsafe extern "C" fn(listener: *mut wl_listener, destroy_handler: *mut c_void) -> i32>,
    pub add_listener_configure_input_panel_surface: Option<unsafe extern "C" fn(listener: *mut wl_listener)>,
    pub add_listener_show_input_panel: Option<unsafe extern "C" fn(listener: *mut wl_listener)>,
    pub add_listener_hide_input_panel: Option<unsafe extern "C" fn(listener: *mut wl_listener)>,
    pub add_listener_update_input_panel: Option<unsafe extern "C" fn(listener: *mut wl_listener)>,
}

pub const IVI_SUCCEEDED: i32 = 0;
pub const IVI_FAILED: i32 = -1;
"#;
            fs::write(&bindings_path, stub).expect("Couldn't write stub IVI bindings!");
        }
    }

    // Handle Weston plugin API bindings
    match weston_result {
        Ok(bindings) => {
            bindings
                .write_to_file(&weston_bindings_path)
                .expect("Couldn't write Weston bindings!");
        }
        Err(e) => {
            eprintln!("Warning: Failed to generate Weston bindings: {}", e);
            eprintln!("Creating stub Weston bindings file for development");

            let stub = r#"
// Stub Weston bindings - actual bindings require Weston headers

use libc::{c_char, c_void};

#[repr(C)]
pub struct weston_compositor {
    _unused: [u8; 0],
}

extern "C" {
    pub fn weston_plugin_api_get(
        compositor: *mut weston_compositor,
        api_name: *const c_char,
        version: usize,
    ) -> *const c_void;
}
"#;
            fs::write(&weston_bindings_path, stub).expect("Couldn't write stub Weston bindings!");
        }
    }

    // Create symbolic link in target directory
    // Note: This creates the link even if the library doesn't exist yet
    create_symlink();
}

fn create_symlink() {
    use std::os::unix::fs::symlink;

    // Get the profile (debug or release)
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // Get the target triple (e.g., aarch64-unknown-linux-gnu, x86_64-unknown-linux-gnu)
    let target = env::var("TARGET").unwrap_or_else(|_| "".to_string());

    // Build the target directory path
    let target_dir = if target.is_empty() {
        // Native build: target/<profile>
        PathBuf::from("target").join(&profile)
    } else {
        // Cross-compilation: target/<target>/<profile>
        PathBuf::from("target").join(&target).join(&profile)
    };

    // Create target directory if it doesn't exist
    let _ = fs::create_dir_all(&target_dir);

    let lib_name = "libweston_ivi_controller.so";
    let link_name = "weston-ivi-controller.so";
    let link_path = target_dir.join(link_name);

    // Remove existing symlink if it exists
    let _ = fs::remove_file(&link_path);

    // Create symlink (relative path)
    if let Err(e) = symlink(lib_name, &link_path) {
        eprintln!("Warning: Failed to create symlink {}: {}", link_name, e);
    } else {
        println!(
            "Created symlink: {} -> {} in {}",
            link_name,
            lib_name,
            target_dir.display()
        );
    }
}
