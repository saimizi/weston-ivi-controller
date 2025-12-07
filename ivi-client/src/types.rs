//! Data types for IVI surfaces and layers
//!
//! This module provides the core data structures used to represent IVI compositor
//! objects such as surfaces and layers, along with their properties.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A 2D position with x and y coordinates.
///
/// Positions are used to specify the location of surfaces on the screen.
/// Coordinates can be negative to position surfaces off-screen or partially visible.
///
/// # Examples
///
/// ```
/// use ivi_client::Position;
///
/// let pos = Position { x: 100, y: 200 };
/// println!("Position: {}", pos); // Prints: Position: (100, 200)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// X coordinate (horizontal position)
    pub x: i32,
    /// Y coordinate (vertical position)
    pub y: i32,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Dimensions representing width and height.
///
/// Size is used to specify the dimensions of surfaces and other rectangular areas.
/// Both width and height must be positive values.
///
/// # Examples
///
/// ```
/// use ivi_client::Size;
///
/// let size = Size { width: 1920, height: 1080 };
/// println!("Size: {}", size); // Prints: Size: 1920x1080
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

/// Rotation orientation for surfaces.
///
/// Surfaces can be rotated in 90-degree increments to support different
/// display configurations and orientations.
///
/// # Examples
///
/// ```
/// use ivi_client::Orientation;
///
/// let orientation = Orientation::Rotate90;
/// println!("Orientation: {}", orientation); // Prints: Orientation: Rotate90
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    /// No rotation (0 degrees)
    Normal,
    /// Rotated 90 degrees clockwise
    Rotate90,
    /// Rotated 180 degrees
    Rotate180,
    /// Rotated 270 degrees clockwise (90 degrees counter-clockwise)
    Rotate270,
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Orientation::Normal => write!(f, "Normal"),
            Orientation::Rotate90 => write!(f, "Rotate90"),
            Orientation::Rotate180 => write!(f, "Rotate180"),
            Orientation::Rotate270 => write!(f, "Rotate270"),
        }
    }
}

/// An IVI surface representing an application window.
///
/// Surfaces are the visual elements that represent application windows in the
/// IVI compositor. Each surface has properties that control its appearance and
/// behavior on the screen.
///
/// # Examples
///
/// ```
/// use ivi_client::{Surface, Position, Size, Orientation};
///
/// let surface = Surface {
///     id: 1000,
///     position: Position { x: 0, y: 0 },
///     size: Size { width: 1920, height: 1080 },
///     visibility: true,
///     opacity: 1.0,
///     orientation: Orientation::Normal,
///     z_order: 0,
/// };
///
/// println!("Surface ID: {}", surface.id);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Surface {
    /// Unique identifier for the surface
    pub id: u32,
    /// Position on the screen
    pub position: Position,
    /// Dimensions of the surface
    pub size: Size,
    /// Whether the surface is visible
    pub visibility: bool,
    /// Opacity level (0.0 = transparent, 1.0 = opaque)
    pub opacity: f32,
    /// Rotation orientation
    pub orientation: Orientation,
    /// Stacking order (higher values appear on top)
    pub z_order: i32,
}

impl fmt::Display for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Surface {{\n  ID: {}\n  Position: {}\n  Size: {}\n  Visibility: {}\n  Opacity: {:.2}\n  Orientation: {}\n  Z-Order: {}\n}}",
            self.id, self.position, self.size, self.visibility, self.opacity, self.orientation, self.z_order
        )
    }
}

/// An IVI layer that groups and organizes surfaces.
///
/// Layers are containers that group multiple surfaces together and control
/// their collective rendering. Layers can be used to organize surfaces into
/// logical groups (e.g., background, application, overlay).
///
/// # Examples
///
/// ```
/// use ivi_client::Layer;
///
/// let layer = Layer {
///     id: 2000,
///     visibility: true,
///     opacity: 1.0,
/// };
///
/// println!("Layer ID: {}", layer.id);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    /// Unique identifier for the layer
    pub id: u32,
    /// Whether the layer is visible
    pub visibility: bool,
    /// Opacity level (0.0 = transparent, 1.0 = opaque)
    pub opacity: f32,
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Layer {{\n  ID: {}\n  Visibility: {}\n  Opacity: {:.2}\n}}",
            self.id, self.visibility, self.opacity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_display() {
        let pos = Position { x: 100, y: 200 };
        assert_eq!(pos.to_string(), "(100, 200)");
    }

    #[test]
    fn test_size_display() {
        let size = Size {
            width: 1920,
            height: 1080,
        };
        assert_eq!(size.to_string(), "1920x1080");
    }

    #[test]
    fn test_orientation_display() {
        assert_eq!(Orientation::Normal.to_string(), "Normal");
        assert_eq!(Orientation::Rotate90.to_string(), "Rotate90");
        assert_eq!(Orientation::Rotate180.to_string(), "Rotate180");
        assert_eq!(Orientation::Rotate270.to_string(), "Rotate270");
    }

    #[test]
    fn test_surface_serialization() {
        let surface = Surface {
            id: 1000,
            position: Position { x: 100, y: 200 },
            size: Size {
                width: 1920,
                height: 1080,
            },
            visibility: true,
            opacity: 1.0,
            orientation: Orientation::Normal,
            z_order: 0,
        };

        let json = serde_json::to_string(&surface).unwrap();
        let deserialized: Surface = serde_json::from_str(&json).unwrap();
        assert_eq!(surface, deserialized);
    }

    #[test]
    fn test_layer_serialization() {
        let layer = Layer {
            id: 2000,
            visibility: true,
            opacity: 0.75,
        };

        let json = serde_json::to_string(&layer).unwrap();
        let deserialized: Layer = serde_json::from_str(&json).unwrap();
        assert_eq!(layer, deserialized);
    }

    #[test]
    fn test_orientation_serialization() {
        let orientations = vec![
            Orientation::Normal,
            Orientation::Rotate90,
            Orientation::Rotate180,
            Orientation::Rotate270,
        ];

        for orientation in orientations {
            let json = serde_json::to_string(&orientation).unwrap();
            let deserialized: Orientation = serde_json::from_str(&json).unwrap();
            assert_eq!(orientation, deserialized);
        }
    }
}
