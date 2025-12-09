//! Rust API Example for IVI Client Library
//!
//! This example demonstrates how to use the IVI client library from Rust to:
//! - Connect to the IVI controller
//! - List and query surfaces and layers
//! - Modify surface and layer properties
//! - Handle errors gracefully
//!
//! Usage:
//!   cargo run --example rust_example

use ivi_client::{IviClient, IviError, Orientation, Result};

fn main() {
    if let Err(e) = run_example() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_example() -> Result<()> {
    println!("=== IVI Client Library - Rust Example ===\n");

    // Connect to the IVI controller
    println!("Connecting to IVI controller...");
    let socket_path = std::env::var("IVI_SOCKET")
        .unwrap_or_else(|_| "/tmp/weston-ivi-controller.sock".to_string());

    let mut client = match IviClient::connect(&socket_path) {
        Ok(client) => {
            println!("✓ Connected to {}\n", socket_path);
            client
        }
        Err(IviError::ConnectionFailed(msg)) => {
            eprintln!("✗ Connection failed: {}", msg);
            eprintln!("\nMake sure the Weston IVI controller is running and listening on:");
            eprintln!("  {}", socket_path);
            return Err(IviError::ConnectionFailed(msg));
        }
        Err(e) => return Err(e),
    };

    // Demonstrate surface operations
    demonstrate_surface_operations(&mut client)?;

    // Demonstrate layer operations
    demonstrate_layer_operations(&mut client)?;

    // Demonstrate error handling
    demonstrate_error_handling(&mut client);

    println!("\n=== Example completed successfully ===");
    Ok(())
}

fn demonstrate_surface_operations(client: &mut IviClient) -> Result<()> {
    println!("--- Surface Operations ---\n");

    // List all surfaces
    println!("Listing all surfaces...");
    let surfaces = client.list_surfaces()?;

    if surfaces.is_empty() {
        println!("  No surfaces found");
        return Ok(());
    }

    println!("  Found {} surface(s):", surfaces.len());
    for surface in &surfaces {
        println!("    Surface ID: {}", surface.id);
        println!(
            "      OrigSize: ({}, {})",
            surface.orig_size.width, surface.orig_size.height
        );
        println!(
            "      SrcPos: ({}, {})",
            surface.src_position.x, surface.src_position.y
        );
        println!(
            "      SrcSize: {}x{}",
            surface.src_size.width, surface.src_size.height
        );
        println!(
            "      DestPos: ({}, {})",
            surface.dest_position.x, surface.dest_position.y
        );
        println!(
            "      DestSize: {}x{}",
            surface.dest_size.width, surface.dest_size.height
        );
        println!("      Visibility: {}", surface.visibility);
        println!("      Opacity: {:.2}", surface.opacity);
        println!("      Orientation: {:?}", surface.orientation);
        println!("      Z-Order: {}", surface.z_order);
        println!();
    }

    // Get properties of the first surface
    if let Some(first_surface) = surfaces.first() {
        let surface_id = first_surface.id;
        println!("Getting properties for surface {}...", surface_id);
        let surface = client.get_surface(surface_id)?;
        println!("  ✓ Retrieved surface {}", surface.id);
        println!("    Current opacity: {:.2}", surface.opacity);
        println!("    Current visibility: {}", surface.visibility);

        // Modify surface properties
        println!("\nModifying surface {} properties...", surface_id);

        // Set position
        println!("  Setting position to (100, 100)...");
        client.set_surface_position(surface_id, 100, 100)?;
        println!("    ✓ Position updated");

        // Set size
        println!("  Setting size to 800x600...");
        client.set_surface_size(surface_id, 800, 600)?;
        println!("    ✓ Size updated");

        // Set visibility
        println!("  Setting visibility to true...");
        client.set_surface_visibility(surface_id, true)?;
        println!("    ✓ Visibility updated");

        // Set opacity
        println!("  Setting opacity to 0.8...");
        client.set_surface_opacity(surface_id, 0.8)?;
        println!("    ✓ Opacity updated");

        // Set orientation
        println!("  Setting orientation to Normal...");
        client.set_surface_orientation(surface_id, Orientation::Normal)?;
        println!("    ✓ Orientation updated");

        // Set z-order
        println!("  Setting z-order to 10...");
        client.set_surface_z_order(surface_id, 10)?;
        println!("    ✓ Z-order updated");

        // Commit all changes atomically
        println!("\nCommitting changes...");
        client.commit()?;
        println!("  ✓ All changes committed successfully");

        // Verify changes
        println!("\nVerifying changes...");
        let updated_surface = client.get_surface(surface_id)?;
        println!(
            "  OrigSize: {}x{}",
            updated_surface.orig_size.width, updated_surface.orig_size.height
        );
        println!(
            "  SrcPos: ({}, {})",
            updated_surface.src_position.x, updated_surface.src_position.y
        );
        println!(
            "  SrcSize: {}x{}",
            updated_surface.src_size.width, updated_surface.src_size.height
        );
        println!(
            "  DestPos: ({}, {})",
            updated_surface.dest_position.x, updated_surface.dest_position.y
        );
        println!(
            "  DestSize: {}x{}",
            updated_surface.dest_size.width, updated_surface.dest_size.height
        );
        println!("  Opacity: {:.2}", updated_surface.opacity);
        println!("  Visibility: {}", updated_surface.visibility);
    }

    println!();
    Ok(())
}

fn demonstrate_layer_operations(client: &mut IviClient) -> Result<()> {
    println!("--- Layer Operations ---\n");

    // List all layers
    println!("Listing all layers...");
    let layers = client.list_layers()?;

    if layers.is_empty() {
        println!("  No layers found");
        return Ok(());
    }

    println!("  Found {} layer(s):", layers.len());
    for layer in &layers {
        println!("    Layer ID: {}", layer.id);
        println!("      Visibility: {}", layer.visibility);
        println!("      Opacity: {:.2}", layer.opacity);
        println!();
    }

    // Get properties of the first layer
    if let Some(first_layer) = layers.first() {
        let layer_id = first_layer.id;
        println!("Getting properties for layer {}...", layer_id);
        let layer = client.get_layer(layer_id)?;
        println!("  ✓ Retrieved layer {}", layer.id);
        println!("    Current opacity: {:.2}", layer.opacity);
        println!("    Current visibility: {}", layer.visibility);

        // Modify layer properties
        println!("\nModifying layer {} properties...", layer_id);

        // Set visibility
        println!("  Setting visibility to true...");
        client.set_layer_visibility(layer_id, true)?;
        println!("    ✓ Visibility updated");

        // Set opacity
        println!("  Setting opacity to 0.9...");
        client.set_layer_opacity(layer_id, 0.9)?;
        println!("    ✓ Opacity updated");

        // Commit changes
        println!("\nCommitting changes...");
        client.commit()?;
        println!("  ✓ All changes committed successfully");

        // Verify changes
        println!("\nVerifying changes...");
        let updated_layer = client.get_layer(layer_id)?;
        println!("  Opacity: {:.2}", updated_layer.opacity);
        println!("  Visibility: {}", updated_layer.visibility);
    }

    println!();
    Ok(())
}

fn demonstrate_error_handling(client: &mut IviClient) {
    println!("--- Error Handling ---\n");

    // Try to get a non-existent surface
    println!("Attempting to get non-existent surface (ID: 99999)...");
    match client.get_surface(99999) {
        Ok(_) => println!("  Unexpected success"),
        Err(IviError::RequestFailed { code, message }) => {
            println!("  ✓ Correctly handled error:");
            println!("    Error code: {}", code);
            println!("    Error message: {}", message);
        }
        Err(e) => {
            println!("  ✓ Correctly handled error: {}", e);
        }
    }

    println!();

    // Try to get a non-existent layer
    println!("Attempting to get non-existent layer (ID: 99999)...");
    match client.get_layer(99999) {
        Ok(_) => println!("  Unexpected success"),
        Err(IviError::RequestFailed { code, message }) => {
            println!("  ✓ Correctly handled error:");
            println!("    Error code: {}", code);
            println!("    Error message: {}", message);
        }
        Err(e) => {
            println!("  ✓ Correctly handled error: {}", e);
        }
    }

    println!();
}
