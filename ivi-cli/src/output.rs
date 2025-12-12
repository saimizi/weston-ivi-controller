//! Output formatting utilities for the CLI
//!
//! This module provides functions to format CLI output in a consistent,
//! human-readable manner.

use ivi_client::{IviLayer, IviSurface};

/// Format a list of surface IDs as a comma-separated string
///
/// # Arguments
/// * `surfaces` - Vector of surfaces to format
///
/// # Returns
/// A formatted string with surface IDs, or a message if no surfaces exist
///
/// # Examples
/// ```
/// let surfaces = vec![
///     Surface { id: 1000, ... },
///     Surface { id: 1001, ... },
/// ];
/// let output = format_surface_list(&surfaces);
/// assert_eq!(output, "Surface IDs: 1000, 1001");
/// ```
pub fn format_surface_list(surfaces: &[IviSurface]) -> String {
    if surfaces.is_empty() {
        "No surfaces available".to_string()
    } else {
        let ids: Vec<String> = surfaces.iter().map(|s| s.id.to_string()).collect();
        format!("Surface IDs: {}", ids.join(", "))
    }
}

/// Format a list of layer IDs as a comma-separated string
///
/// # Arguments
/// * `layers` - Vector of layers to format
///
/// # Returns
/// A formatted string with layer IDs, or a message if no layers exist
///
/// # Examples
/// ```
/// let layers = vec![
///     Layer { id: 2000, ... },
///     Layer { id: 2001, ... },
/// ];
/// let output = format_layer_list(&layers);
/// assert_eq!(output, "Layer IDs: 2000, 2001");
/// ```
pub fn format_layer_list(layers: &[IviLayer]) -> String {
    if layers.is_empty() {
        "No layers available".to_string()
    } else {
        let ids: Vec<String> = layers.iter().map(|l| l.id.to_string()).collect();
        format!("Layer IDs: {}", ids.join(", "))
    }
}

pub fn format_layer_create_success(id: u32) -> String {
    format_success(&format!("Layer {} created", id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ivi_client::{IviOrientation, IviSize, Rectangle};

    #[test]
    fn test_format_surface_list_empty() {
        let surfaces = vec![];
        assert_eq!(format_surface_list(&surfaces), "No surfaces available");
    }

    #[test]
    fn test_format_surface_list_single() {
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
        assert_eq!(format_surface_list(&surfaces), "Surface IDs: 1000");
    }

    #[test]
    fn test_format_surface_list_multiple() {
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
        assert_eq!(
            format_surface_list(&surfaces),
            "Surface IDs: 1000, 1001, 1002"
        );
    }

    #[test]
    fn test_format_layer_list_empty() {
        let layers = vec![];
        assert_eq!(format_layer_list(&layers), "No layers available");
    }

    #[test]
    fn test_format_layer_list_single() {
        let layers = vec![IviLayer {
            id: 2000,
            visibility: true,
            opacity: 1.0,
        }];
        assert_eq!(format_layer_list(&layers), "Layer IDs: 2000");
    }

    #[test]
    fn test_format_layer_list_multiple() {
        let layers = vec![
            IviLayer {
                id: 2000,
                visibility: true,
                opacity: 1.0,
            },
            IviLayer {
                id: 2001,
                visibility: false,
                opacity: 0.5,
            },
            IviLayer {
                id: 2002,
                visibility: true,
                opacity: 0.75,
            },
        ];
        assert_eq!(format_layer_list(&layers), "Layer IDs: 2000, 2001, 2002");
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
        "Layer {}:\n  Visibility: {}\n  Opacity: {:.2}",
        layer.id, layer.visibility, layer.opacity
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
        assert!(output.contains("Orientation: Normal"));
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
        assert!(output.contains("Orientation: Rotate90"));
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
        };

        let output = format_layer_properties(&layer);
        // Should format with 2 decimal places
        assert!(output.contains("Opacity: 0.99"));
    }
}

/// Format a success message with a checkmark
///
/// # Arguments
/// * `message` - The success message to format
///
/// # Returns
/// A formatted success message with a checkmark prefix
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

/// Format a success message for commit operation
pub fn format_commit_success() -> String {
    format_success("Changes committed")
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
