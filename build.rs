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
    let bindings_ivi = bindgen::Builder::default()
        .header("weston/ivi-shell/ivi-layout-export.h")
        .blocklist_item("IVI_SUCCEEDED")
        .clang_arg("-Iweston/include/")
        .clang_arg("-I/usr/include/pixman-1/")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("ivi_layout_.*")
        .allowlist_function("ivi_layout_.*")
        .allowlist_var("IVI_.*")
        .allowlist_type("wl_.*")
        .allowlist_type("weston_.*")
        .generate()
        .unwrap();

    // Generate Weston plugin API bindings
    let bindings_weston = bindgen::Builder::default()
        .header("weston/include/libweston/plugin-registry.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("weston_plugin_api_.*")
        .allowlist_type("weston_compositor")
        .generate()
        .unwrap();

    // Handle IVI bindings
    bindings_ivi
        .write_to_file(&bindings_path)
        .expect("Couldn't write IVI bindings!");

    bindings_weston
        .write_to_file(&weston_bindings_path)
        .expect("Couldn't write Weston bindings!");

    // Handle Weston plugin API bindings
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
