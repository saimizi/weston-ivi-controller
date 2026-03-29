#![allow(dead_code)]

//! Output formatting utilities for the CLI
//!
//! This module provides functions to format CLI output in a consistent,
//! human-readable manner.
use ivi_client::{IviLayer, IviScreen, IviSurface};

/// Format a list of surfaces
///
/// # Arguments
/// * `surfaces` - Vector of surfaces to format
/// * `ids_only` - If true, only show surface IDs
///
/// # Returns
/// A formatted string with surface information, or a message if no surfaces exist
pub fn format_surface_list(surfaces: &[IviSurface], ids_only: bool) -> String {
    if surfaces.is_empty() {
        return "No surfaces available".to_string();
    }

    if ids_only {
        let ids: Vec<String> = surfaces.iter().map(|s| s.id.to_string()).collect();
        return ids.join(" ");
    }

    let mut output = format!("Found {} surface(s):\n", surfaces.len());
    for surface in surfaces {
        output.push_str(&format!("  Surface {}:\n", surface.id));
        output.push_str(&format!("    OrigSize: {}\n", surface.orig_size));
        output.push_str(&format!("    SrcRect: {}\n", surface.src_rect));
        output.push_str(&format!("    DestRect: {}\n", surface.dest_rect));
        output.push_str(&format!("    Visibility: {}\n", surface.visibility));
        output.push_str(&format!("    Opacity: {:.2}\n", surface.opacity));
        output.push_str(&format!("    Orientation: {}\n", surface.orientation));
        output.push_str(&format!("    Z-Order: {}\n", surface.z_order));
    }
    output.trim_end().to_string()
}

/// Format a list of layers
///
/// # Arguments
/// * `layers` - Vector of layers to format
/// * `ids_only` - If true, only show layer IDs
///
/// # Returns
/// A formatted string with layer information, or a message if no layers exist
pub fn format_layer_list(layers: &[IviLayer], ids_only: bool) -> String {
    if layers.is_empty() {
        return "No layers available".to_string();
    }

    if ids_only {
        let ids: Vec<String> = layers.iter().map(|l| l.id.to_string()).collect();
        return ids.join(" ");
    }

    let mut output = format!("Found {} layer(s):\n", layers.len());
    for layer in layers {
        output.push_str(&format!("  Layer {}:\n", layer.id));
        output.push_str(&format!("    SrcRect: {}\n", layer.src_rect));
        output.push_str(&format!("    DestRect: {}\n", layer.dest_rect));
        output.push_str(&format!("    Visibility: {}\n", layer.visibility));
        output.push_str(&format!("    Opacity: {:.2}\n", layer.opacity));
        output.push_str(&format!("    Orientation: {}\n", layer.orientation));
    }
    output.trim_end().to_string()
}

pub fn format_layer_create_success(id: u32) -> String {
    format_success(&format!("Layer {} created", id))
}

pub fn format_layer_destroy_success(id: u32) -> String {
    format_success(&format!("Layer {} destroyed", id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ivi_client::{IviOrientation, IviSize, Rectangle};

    #[test]
    fn test_format_surface_list_empty() {
        let surfaces = vec![];
        assert_eq!(
            format_surface_list(&surfaces, false),
            "No surfaces available"
        );
        assert_eq!(
            format_surface_list(&surfaces, true),
            "No surfaces available"
        );
    }

    #[test]
    fn test_format_surface_list_ids_only() {
        let surfaces = vec![IviSurface {
            id: 1000,
            orig_size: IviSize {
                width: 100,
                height: 100,
            },
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            visibility: true,
            opacity: 1.0,
            orientation: IviOrientation::Normal,
            z_order: 0,
        }];
        assert_eq!(format_surface_list(&surfaces, true), "1000");
    }

    #[test]
    fn test_format_surface_list_detailed() {
        let surfaces = vec![IviSurface {
            id: 1000,
            orig_size: IviSize {
                width: 100,
                height: 100,
            },
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            visibility: true,
            opacity: 1.0,
            orientation: IviOrientation::Normal,
            z_order: 0,
        }];
        let output = format_surface_list(&surfaces, false);
        assert!(output.contains("Found 1 surface(s):"));
        assert!(output.contains("Surface 1000:"));
        assert!(output.contains("OrigSize: 100x100"));
        assert!(output.contains("Visibility: true"));
        assert!(output.contains("Opacity: 1.00"));
        assert!(output.contains("Z-Order: 0"));
    }

    #[test]
    fn test_format_surface_list_multiple_ids_only() {
        let surfaces = vec![
            IviSurface {
                id: 1000,
                orig_size: IviSize {
                    width: 100,
                    height: 100,
                },
                src_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 100,
                    height: 100,
                },
                dest_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 100,
                    height: 100,
                },
                visibility: true,
                opacity: 1.0,
                orientation: IviOrientation::Normal,
                z_order: 0,
            },
            IviSurface {
                id: 1001,
                orig_size: IviSize {
                    width: 200,
                    height: 200,
                },
                src_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 200,
                    height: 200,
                },
                dest_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 200,
                    height: 200,
                },
                visibility: false,
                opacity: 0.5,
                orientation: IviOrientation::Rotate90,
                z_order: 1,
            },
            IviSurface {
                id: 1002,
                orig_size: IviSize {
                    width: 300,
                    height: 300,
                },
                src_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 300,
                    height: 300,
                },
                dest_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 300,
                    height: 300,
                },
                visibility: true,
                opacity: 0.75,
                orientation: IviOrientation::Rotate180,
                z_order: 2,
            },
        ];
        assert_eq!(format_surface_list(&surfaces, true), "1000 1001 1002");
    }

    #[test]
    fn test_format_surface_list_multiple_detailed() {
        let surfaces = vec![
            IviSurface {
                id: 1000,
                orig_size: IviSize {
                    width: 100,
                    height: 100,
                },
                src_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 100,
                    height: 100,
                },
                dest_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 100,
                    height: 100,
                },
                visibility: true,
                opacity: 1.0,
                orientation: IviOrientation::Normal,
                z_order: 0,
            },
            IviSurface {
                id: 1001,
                orig_size: IviSize {
                    width: 200,
                    height: 200,
                },
                src_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 200,
                    height: 200,
                },
                dest_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 200,
                    height: 200,
                },
                visibility: false,
                opacity: 0.5,
                orientation: IviOrientation::Rotate90,
                z_order: 1,
            },
        ];
        let output = format_surface_list(&surfaces, false);
        assert!(output.contains("Found 2 surface(s):"));
        assert!(output.contains("Surface 1000:"));
        assert!(output.contains("Surface 1001:"));
        assert!(output.contains("Visibility: false"));
        assert!(output.contains("Opacity: 0.50"));
        assert!(output.contains("Orientation: 90 degrees"));
    }

    #[test]
    fn test_format_layer_list_empty() {
        let layers = vec![];
        assert_eq!(format_layer_list(&layers, false), "No layers available");
        assert_eq!(format_layer_list(&layers, true), "No layers available");
    }

    #[test]
    fn test_format_layer_list_ids_only() {
        let layers = vec![IviLayer {
            id: 2000,
            visibility: true,
            opacity: 1.0,
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            orientation: IviOrientation::Normal,
        }];
        assert_eq!(format_layer_list(&layers, true), "2000");
    }

    #[test]
    fn test_format_layer_list_detailed() {
        let layers = vec![IviLayer {
            id: 2000,
            visibility: true,
            opacity: 1.0,
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            orientation: IviOrientation::Normal,
        }];
        let output = format_layer_list(&layers, false);
        assert!(output.contains("Found 1 layer(s):"));
        assert!(output.contains("Layer 2000:"));
        assert!(output.contains("Visibility: true"));
        assert!(output.contains("Opacity: 1.00"));
    }

    #[test]
    fn test_format_layer_list_multiple_ids_only() {
        let layers = vec![
            IviLayer {
                id: 2000,
                visibility: true,
                opacity: 1.0,
                src_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                dest_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                orientation: IviOrientation::Normal,
            },
            IviLayer {
                id: 2001,
                visibility: false,
                opacity: 0.5,
                src_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                dest_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                orientation: IviOrientation::Normal,
            },
            IviLayer {
                id: 2002,
                visibility: true,
                opacity: 0.75,
                src_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                dest_rect: Rectangle {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                orientation: IviOrientation::Normal,
            },
        ];
        assert_eq!(format_layer_list(&layers, true), "2000 2001 2002");
    }
}

/// Format surface properties with labels and indentation
///
/// # Arguments
/// * `surface` - The surface to format
///
/// # Returns
/// A formatted string with all surface properties
///
/// # Examples
/// ```
/// let surface = Surface {
///     id: 1000,
///     position: Position { x: 100, y: 200 },
///     size: Size { width: 1920, height: 1080 },
///     visibility: true,
///     opacity: 1.0,
///     orientation: Orientation::Normal,
///     z_order: 0,
/// };
/// let output = format_surface_properties(&surface);
/// ```
pub fn format_surface_properties(surface: &IviSurface) -> String {
    format!(
        "Surface {}:\n  OrigSize: {}\n  SrcRect: {}\n  DestRect: {}\n Visibility: {}\n  Opacity: {:.2}\n  Orientation: {}\n  Z-Order: {}",
        surface.id,
        surface.orig_size,
        surface.src_rect,
        surface.dest_rect,
        surface.visibility,
        surface.opacity,
        surface.orientation,
        surface.z_order
    )
}

/// Format layer properties with labels and indentation
///
/// # Arguments
/// * `layer` - The layer to format
///
/// # Returns
/// A formatted string with all layer properties
///
/// # Examples
/// ```
/// let layer = Layer {
///     id: 2000,
///     visibility: true,
///     opacity: 0.75,
/// };
/// let output = format_layer_properties(&layer);
/// ```
pub fn format_layer_properties(layer: &IviLayer) -> String {
    format!(
        "Layer {}:\n  SrcRect: {}\n  DestRect: {}\n Visibility: {}\n  Opacity: {:.2}\n  Orientation: {}",
        layer.id,
        layer.src_rect,
        layer.dest_rect,
        layer.visibility,
        layer.opacity,
        layer.orientation
    )
}

#[cfg(test)]
mod properties_tests {
    use super::*;
    use ivi_client::{IviOrientation, IviSize, Rectangle};

    #[test]
    fn test_format_surface_properties() {
        let surface = IviSurface {
            id: 1000,
            orig_size: IviSize {
                width: 1920,
                height: 1080,
            },
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            dest_rect: Rectangle {
                x: 100,
                y: 200,
                width: 1920,
                height: 1080,
            },
            visibility: true,
            opacity: 1.0,
            orientation: IviOrientation::Normal,
            z_order: 0,
        };

        let output = format_surface_properties(&surface);
        assert!(output.contains("Surface 1000:"));
        assert!(output.contains("OrigSize: 1920x1080"));
        assert!(output.contains("SrcRect: 1920x1080@(0, 0)"));
        assert!(output.contains("DestRect: 1920x1080@(100, 200)"));
        assert!(output.contains("Visibility: true"));
        assert!(output.contains("Opacity: 1.00"));
        assert!(output.contains("Orientation: 0 degrees"));
        assert!(output.contains("Z-Order: 0"));
    }

    #[test]
    fn test_format_surface_properties_with_rotation() {
        let surface = IviSurface {
            id: 1001,
            orig_size: IviSize {
                width: 800,
                height: 600,
            },
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            },
            dest_rect: Rectangle {
                x: -50,
                y: -100,
                width: 1280,
                height: 720,
            },
            visibility: false,
            opacity: 0.5,
            orientation: IviOrientation::Rotate90,
            z_order: -1,
        };

        let output = format_surface_properties(&surface);
        assert!(output.contains("Surface 1001:"));
        assert!(output.contains("OrigSize: 800x600"));
        assert!(output.contains("SrcRect: 800x600@(0, 0)"));
        assert!(output.contains("DestRect: 1280x720@(-50, -100)"));
        assert!(output.contains("Visibility: false"));
        assert!(output.contains("Opacity: 0.50"));
        assert!(output.contains("Orientation: 90 degrees"));
        assert!(output.contains("Z-Order: -1"));
    }

    #[test]
    fn test_format_surface_properties_opacity_precision() {
        let surface = IviSurface {
            id: 1002,
            orig_size: IviSize {
                width: 100,
                height: 100,
            },
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            visibility: true,
            opacity: 0.123456,
            orientation: IviOrientation::Normal,
            z_order: 0,
        };

        let output = format_surface_properties(&surface);
        // Should format with 2 decimal places
        assert!(output.contains("Opacity: 0.12"));
    }

    #[test]
    fn test_format_layer_properties() {
        let layer = IviLayer {
            id: 2000,
            visibility: true,
            opacity: 1.0,
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            orientation: IviOrientation::Normal,
        };

        let output = format_layer_properties(&layer);
        assert!(output.contains("Layer 2000:"));
        assert!(output.contains("Visibility: true"));
        assert!(output.contains("Opacity: 1.00"));
    }

    #[test]
    fn test_format_layer_properties_partial_opacity() {
        let layer = IviLayer {
            id: 2001,
            visibility: false,
            opacity: 0.75,
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            orientation: IviOrientation::Normal,
        };

        let output = format_layer_properties(&layer);
        assert!(output.contains("Layer 2001:"));
        assert!(output.contains("Visibility: false"));
        assert!(output.contains("Opacity: 0.75"));
    }

    #[test]
    fn test_format_layer_properties_opacity_precision() {
        let layer = IviLayer {
            id: 2002,
            visibility: true,
            opacity: 0.987654,
            src_rect: Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            dest_rect: Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
            orientation: IviOrientation::Normal,
        };

        let output = format_layer_properties(&layer);
        // Should format with 2 decimal places
        assert!(output.contains("Opacity: 0.99"));
    }
}

/// Format a success message with a check mark
///
/// # Arguments
/// * `message` - The success message to format
///
/// # Returns
/// A formatted success message with a check mark prefix
///
/// # Examples
/// ```
/// let msg = format_success("Operation completed");
/// assert_eq!(msg, "✓ Operation completed");
/// ```
pub fn format_success(message: &str) -> String {
    format!("✓ {}", message)
}

/// Format an error message with a cross mark
///
/// # Arguments
/// * `error` - The error to format
///
/// # Returns
/// A formatted error message with a cross mark prefix
///
/// # Examples
/// ```
/// let msg = format_error("Connection failed");
/// assert_eq!(msg, "✗ Error: Connection failed");
/// ```
pub fn format_error(error: &dyn std::error::Error) -> String {
    format!("✗ Error: {}", error)
}

/// Format a success message for setting surface visibility
pub fn format_surface_visibility_success(id: u32, visible: bool) -> String {
    format_success(&format!("Surface {} visibility set to {}", id, visible))
}

/// Format a success message for setting surface opacity
pub fn format_surface_opacity_success(id: u32, opacity: f32) -> String {
    format_success(&format!("Surface {} opacity set to {:.2}", id, opacity))
}

/// Format a success message for setting surface source rectangle
pub fn format_surface_source_rect_success(
    id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> String {
    format_success(&format!(
        "Surface {} source rectangle set to position ({}, {}) and size {}x{}",
        id, x, y, width, height
    ))
}

/// Format a success message for setting surface destination rectangle
pub fn format_surface_dest_rect_success(
    id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> String {
    format_success(&format!(
        "Surface {} destination rectangle set to position ({}, {}) and size {}x{}",
        id, x, y, width, height
    ))
}

/// Format a success message for setting surface z-order
pub fn format_surface_z_order_success(id: u32, z_order: i32) -> String {
    format_success(&format!("Surface {} z-order set to {}", id, z_order))
}

/// Format a success message for setting surface focus
pub fn format_surface_focus_success(id: u32) -> String {
    format_success(&format!("Surface {} focus set", id))
}

/// Format a success message for setting layer visibility
pub fn format_layer_visibility_success(id: u32, visible: bool) -> String {
    format_success(&format!("Layer {} visibility set to {}", id, visible))
}

/// Format a success message for setting layer opacity
pub fn format_layer_opacity_success(id: u32, opacity: f32) -> String {
    format_success(&format!("Layer {} opacity set to {:.2}", id, opacity))
}

/// Format a success message for setting layer source rectangle
pub fn format_layer_source_rect_success(
    id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> String {
    format_success(&format!(
        "Layer {} source rectangle set to position ({}, {}) and size {}x{}",
        id, x, y, width, height
    ))
}

/// Format a success message for setting layer destination rectangle
pub fn format_layer_dest_rect_success(id: u32, x: i32, y: i32, width: i32, height: i32) -> String {
    format_success(&format!(
        "Layer {} destination rectangle set to position ({}, {}) and size {}x{}",
        id, x, y, width, height
    ))
}

/// Format screen list output
pub fn format_screen_list(screens: &[ivi_client::IviScreen]) -> String {
    if screens.is_empty() {
        return String::from("No screens found");
    }

    let mut output = format!("Found {} screen(s):\n", screens.len());
    for screen in screens {
        output.push_str(&format!(
            "  {} - {}x{} at ({:.0}, {:.0}), transform: {}, scale: {}, {}\n",
            screen.name,
            screen.width,
            screen.height,
            screen.x,
            screen.y,
            screen.transform,
            screen.scale,
            if screen.enabled {
                "enabled"
            } else {
                "disabled"
            }
        ));
    }
    output.trim_end().to_string()
}

/// Format screen properties output
pub fn format_screen_properties(screen: &ivi_client::IviScreen) -> String {
    format!(
        "Screen: {}\n  Resolution: {}x{}\n  Position: ({:.0}, {:.0})\n  Transform: {}\n  Scale: {}\n  Status: {}",
        screen.name,
        screen.width,
        screen.height,
        screen.x,
        screen.y,
        screen.transform,
        screen.scale,
        if screen.enabled { "enabled" } else { "disabled" }
    )
}

/// Format screen layers output
pub fn format_screen_layers(screen_name: &str, layer_ids: &[u32]) -> String {
    if layer_ids.is_empty() {
        return format!("Screen '{}' has no layers assigned", screen_name);
    }

    let mut output = format!(
        "Screen '{}' has {} layer(s) (top to bottom):\n",
        screen_name,
        layer_ids.len()
    );
    for (index, &layer_id) in layer_ids.iter().enumerate() {
        output.push_str(&format!("  {}. Layer {}\n", index + 1, layer_id));
    }
    output.trim_end().to_string()
}

/// Format layer screens output
pub fn format_layer_screens(layer_id: u32, screen_names: &[String]) -> String {
    if screen_names.is_empty() {
        return format!("Layer {} is not assigned to any screens", layer_id);
    }

    let mut output = format!(
        "Layer {} is assigned to {} screen(s):\n",
        layer_id,
        screen_names.len()
    );
    for screen_name in screen_names {
        output.push_str(&format!("  {}\n", screen_name));
    }
    output.trim_end().to_string()
}

/// Format success message for screen set layers operation
pub fn format_screen_set_layers_success(
    screen_name: &str,
    layer_ids: &[u32],
    auto_commit: bool,
) -> String {
    let layer_list = layer_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let commit_msg = if auto_commit { " and committed" } else { "" };
    format_success(&format!(
        "Screen '{}' layers set to [{}]{}",
        screen_name, layer_list, commit_msg
    ))
}

/// Format success message for screen remove layer operation
pub fn format_screen_remove_layer_success(
    screen_name: &str,
    layer_id: u32,
    auto_commit: bool,
) -> String {
    let commit_msg = if auto_commit { " and committed" } else { "" };
    format_success(&format!(
        "Layer {} removed from screen '{}'{}",
        layer_id, screen_name, commit_msg
    ))
}

/// Format layer surfaces list output
pub fn format_layer_surfaces(layer_id: u32, surface_ids: &[u32]) -> String {
    if surface_ids.is_empty() {
        return format!("Layer {} has no surfaces assigned", layer_id);
    }

    let mut output = format!(
        "Layer {} has {} surface(s) (bottom to top):\n",
        layer_id,
        surface_ids.len()
    );
    for (index, &surface_id) in surface_ids.iter().enumerate() {
        output.push_str(&format!("  {}. Surface {}\n", index + 1, surface_id));
    }
    output.trim_end().to_string()
}

/// Format success message for layer set surfaces operation
pub fn format_layer_set_surfaces_success(
    layer_id: u32,
    surface_ids: &[u32],
    auto_commit: bool,
) -> String {
    let surface_list = surface_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let commit_msg = if auto_commit { " and committed" } else { "" };
    format_success(&format!(
        "Layer {} surfaces set to [{}]{}",
        layer_id, surface_list, commit_msg
    ))
}

/// Format success message for layer add surface operation
pub fn format_layer_add_surface_success(
    layer_id: u32,
    surface_id: u32,
    auto_commit: bool,
) -> String {
    let commit_msg = if auto_commit { " and committed" } else { "" };
    format_success(&format!(
        "Surface {} added to layer {}{}",
        surface_id, layer_id, commit_msg
    ))
}

/// Format success message for layer remove surface operation
pub fn format_layer_remove_surface_success(
    layer_id: u32,
    surface_id: u32,
    auto_commit: bool,
) -> String {
    let commit_msg = if auto_commit { " and committed" } else { "" };
    format_success(&format!(
        "Surface {} removed from layer {}{}",
        surface_id, layer_id, commit_msg
    ))
}

/// Format a success message for commit operation
pub fn format_commit_success() -> String {
    format_success("Changes committed")
}

type HierarchicalScene = Vec<(IviScreen, Vec<(IviLayer, Vec<IviSurface>)>)>;

/// Format hierarchical scene showing screens -> layers -> surfaces
///
/// # Arguments
/// * `hierarchy` - Vector of (screen, layers) where layers is Vec<(layer, surfaces)>
///
/// # Returns
/// A formatted string with tree structure and indentation
pub fn format_hierarchical_scene(hierarchy: &HierarchicalScene) -> String {
    if hierarchy.is_empty() {
        return "No screens available".to_string();
    }

    let mut output = String::new();
    let layer_indent = " ".repeat(8);
    let layer_prop_indent = " ".repeat(12);
    let surface_indent = " ".repeat(18);
    let surface_prop_indent = " ".repeat(22);

    for (screen, layers) in hierarchy.iter() {
        // Screen header
        output.push_str(&format!("Screen: {}\n", screen.name));

        // Screen properties (2-space indent)
        output.push_str(&format!(
            "  Resolution: {}x{}\n",
            screen.width, screen.height
        ));
        output.push_str(&format!("  Position: ({:.0}, {:.0})\n", screen.x, screen.y));
        output.push_str(&format!("  Transform: {}\n", screen.transform));
        output.push_str(&format!("  Enabled: {}\n", screen.enabled));
        output.push_str(&format!("  Scale: {}\n", screen.scale));

        if layers.is_empty() {
            output.push_str(&format!("{}No layers assigned\n", layer_indent));
        } else {
            for (layer, surfaces) in layers.iter() {
                //output.push('\n');

                // Layer header (2-space indent)
                output.push_str(&format!("{}Layer {}:\n", layer_indent, layer.id));

                // Layer properties (4-space indent)
                output.push_str(&format!(
                    "{}SrcRect: {}\n",
                    layer_prop_indent, layer.src_rect
                ));
                output.push_str(&format!(
                    "{}DestRect: {}\n",
                    layer_prop_indent, layer.dest_rect
                ));
                output.push_str(&format!(
                    "{}Visibility: {}\n",
                    layer_prop_indent, layer.visibility
                ));
                output.push_str(&format!(
                    "{}Opacity: {:.2}\n",
                    layer_prop_indent, layer.opacity
                ));
                output.push_str(&format!(
                    "{}Orientation: {}\n",
                    layer_prop_indent, layer.orientation
                ));

                if surfaces.is_empty() {
                    output.push_str(&format!("{}No surfaces assigned\n", surface_indent));
                } else {
                    for surface in surfaces.iter() {
                        //output.push('\n');

                        // Surface header (6-space indent)
                        output.push_str(&format!("{}Surface {}:\n", surface_indent, surface.id));

                        // Surface properties (8-space indent)
                        output.push_str(&format!(
                            "{}OrigSize: {}\n",
                            surface_prop_indent, surface.orig_size
                        ));
                        output.push_str(&format!(
                            "{}SrcRect: {}\n",
                            surface_prop_indent, surface.src_rect
                        ));
                        output.push_str(&format!(
                            "{}DestRect: {}\n",
                            surface_prop_indent, surface.dest_rect
                        ));
                        output.push_str(&format!(
                            "{}Visibility: {}\n",
                            surface_prop_indent, surface.visibility
                        ));
                        output.push_str(&format!(
                            "{}Opacity: {:.2}\n",
                            surface_prop_indent, surface.opacity
                        ));
                        output.push_str(&format!(
                            "{}Orientation: {}\n",
                            surface_prop_indent, surface.orientation
                        ));
                        output.push_str(&format!(
                            "{}Z-Order: {}\n",
                            surface_prop_indent, surface.z_order
                        ));
                    }
                }
            }
        }

        output.push_str("---------------------------------------------\n");
    }

    output
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    fn test_format_success() {
        assert_eq!(format_success("Test message"), "✓ Test message");
        assert_eq!(
            format_success("Operation completed successfully"),
            "✓ Operation completed successfully"
        );
    }

    #[test]
    fn test_format_error() {
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let formatted = format_error(&error);
        assert!(formatted.starts_with("✗ Error:"));
        assert!(formatted.contains("File not found"));
    }

    #[test]
    fn test_format_surface_visibility_success() {
        assert_eq!(
            format_surface_visibility_success(1000, true),
            "✓ Surface 1000 visibility set to true"
        );
        assert_eq!(
            format_surface_visibility_success(1001, false),
            "✓ Surface 1001 visibility set to false"
        );
    }

    #[test]
    fn test_format_surface_opacity_success() {
        assert_eq!(
            format_surface_opacity_success(1000, 1.0),
            "✓ Surface 1000 opacity set to 1.00"
        );
        assert_eq!(
            format_surface_opacity_success(1001, 0.5),
            "✓ Surface 1001 opacity set to 0.50"
        );
        assert_eq!(
            format_surface_opacity_success(1002, 0.123456),
            "✓ Surface 1002 opacity set to 0.12"
        );
    }

    #[test]
    fn test_format_surface_source_rect_success() {
        assert_eq!(
            format_surface_source_rect_success(1000, 0, 0, 1920, 1080),
            "✓ Surface 1000 source rectangle set to position (0, 0) and size 1920x1080"
        );
        assert_eq!(
            format_surface_source_rect_success(1001, 100, 200, 800, 600),
            "✓ Surface 1001 source rectangle set to position (100, 200) and size 800x600"
        );
    }

    #[test]
    fn test_format_surface_dest_rect_success() {
        assert_eq!(
            format_surface_dest_rect_success(1000, 100, 200, 1920, 1080),
            "✓ Surface 1000 destination rectangle set to position (100, 200) and size 1920x1080"
        );
        assert_eq!(
            format_surface_dest_rect_success(1001, -50, -100, 800, 600),
            "✓ Surface 1001 destination rectangle set to position (-50, -100) and size 800x600"
        );
    }

    #[test]
    fn test_format_surface_z_order_success() {
        assert_eq!(
            format_surface_z_order_success(1000, 0),
            "✓ Surface 1000 z-order set to 0"
        );
        assert_eq!(
            format_surface_z_order_success(1001, -5),
            "✓ Surface 1001 z-order set to -5"
        );
        assert_eq!(
            format_surface_z_order_success(1002, 10),
            "✓ Surface 1002 z-order set to 10"
        );
    }

    #[test]
    fn test_format_surface_focus_success() {
        assert_eq!(
            format_surface_focus_success(1000),
            "✓ Surface 1000 focus set"
        );
    }

    #[test]
    fn test_format_layer_visibility_success() {
        assert_eq!(
            format_layer_visibility_success(2000, true),
            "✓ Layer 2000 visibility set to true"
        );
        assert_eq!(
            format_layer_visibility_success(2001, false),
            "✓ Layer 2001 visibility set to false"
        );
    }

    #[test]
    fn test_format_layer_opacity_success() {
        assert_eq!(
            format_layer_opacity_success(2000, 1.0),
            "✓ Layer 2000 opacity set to 1.00"
        );
        assert_eq!(
            format_layer_opacity_success(2001, 0.75),
            "✓ Layer 2001 opacity set to 0.75"
        );
    }

    #[test]
    fn test_format_layer_source_rect_success() {
        assert_eq!(
            format_layer_source_rect_success(2000, 0, 0, 1920, 1080),
            "✓ Layer 2000 source rectangle set to position (0, 0) and size 1920x1080"
        );
        assert_eq!(
            format_layer_source_rect_success(2001, 100, 200, 800, 600),
            "✓ Layer 2001 source rectangle set to position (100, 200) and size 800x600"
        );
    }

    #[test]
    fn test_format_layer_dest_rect_success() {
        assert_eq!(
            format_layer_dest_rect_success(2000, 100, 200, 1920, 1080),
            "✓ Layer 2000 destination rectangle set to position (100, 200) and size 1920x1080"
        );
        assert_eq!(
            format_layer_dest_rect_success(2001, -50, -100, 800, 600),
            "✓ Layer 2001 destination rectangle set to position (-50, -100) and size 800x600"
        );
    }

    #[test]
    fn test_format_commit_success() {
        assert_eq!(format_commit_success(), "✓ Changes committed");
    }
}
