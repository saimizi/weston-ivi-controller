// Input validation for IVI controller operations

use thiserror::Error;

/// Validation errors for IVI controller operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ValidationError {
    #[error("Invalid position: {param} = {value}, {reason}")]
    InvalidPosition {
        param: String,
        value: String,
        reason: String,
    },

    #[error("Invalid size: {param} = {value}, must be positive non-zero")]
    InvalidSize { param: String, value: i32 },

    #[error("Invalid opacity: {value}, must be in range [0.0, 1.0]")]
    InvalidOpacity { value: f32 },

    #[error("Invalid orientation: {value} degrees, must be a multiple of 90 (0, 90, 180, or 270)")]
    InvalidOrientation { value: i32 },

    #[error("Invalid z-order: {value}, must be in range [{min}, {max}]")]
    InvalidZOrder { value: i32, min: i32, max: i32 },
}

/// Validate position coordinates
///
/// Position coordinates can be negative (for off-screen positioning),
/// but should be within reasonable bounds to prevent overflow issues.
pub fn validate_position(x: i32, y: i32) -> Result<(), ValidationError> {
    // Check for reasonable bounds to prevent overflow
    // Using i32::MIN/2 and i32::MAX/2 as reasonable limits
    const MIN_COORD: i32 = i32::MIN / 2;
    const MAX_COORD: i32 = i32::MAX / 2;

    if x < MIN_COORD || x > MAX_COORD {
        return Err(ValidationError::InvalidPosition {
            param: "x".to_string(),
            value: x.to_string(),
            reason: format!("must be in range [{}, {}]", MIN_COORD, MAX_COORD),
        });
    }

    if y < MIN_COORD || y > MAX_COORD {
        return Err(ValidationError::InvalidPosition {
            param: "y".to_string(),
            value: y.to_string(),
            reason: format!("must be in range [{}, {}]", MIN_COORD, MAX_COORD),
        });
    }

    Ok(())
}

/// Validate size dimensions
///
/// Size dimensions must be positive non-zero values.
pub fn validate_size(width: i32, height: i32) -> Result<(), ValidationError> {
    if width <= 0 {
        return Err(ValidationError::InvalidSize {
            param: "width".to_string(),
            value: width,
        });
    }

    if height <= 0 {
        return Err(ValidationError::InvalidSize {
            param: "height".to_string(),
            value: height,
        });
    }

    Ok(())
}

/// Validate opacity value
///
/// Opacity must be in the range [0.0, 1.0] where:
/// - 0.0 = fully transparent
/// - 1.0 = fully opaque
pub fn validate_opacity(opacity: f32) -> Result<(), ValidationError> {
    if opacity < 0.0 || opacity > 1.0 || opacity.is_nan() {
        return Err(ValidationError::InvalidOpacity { value: opacity });
    }

    Ok(())
}

/// Validate orientation value
///
/// Orientation must be a multiple of 90 degrees (0, 90, 180, 270, etc.)
pub fn validate_orientation(degrees: i32) -> Result<(), ValidationError> {
    if degrees % 90 != 0 {
        return Err(ValidationError::InvalidOrientation { value: degrees });
    }

    Ok(())
}

/// Validate z-order value
///
/// Z-order must be within the valid range for the layer.
/// The range is typically [0, max_surfaces] but can vary.
pub fn validate_z_order(z_order: i32, min: i32, max: i32) -> Result<(), ValidationError> {
    if z_order < min || z_order > max {
        return Err(ValidationError::InvalidZOrder {
            value: z_order,
            min,
            max,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_position_valid() {
        assert!(validate_position(0, 0).is_ok());
        assert!(validate_position(100, 200).is_ok());
        assert!(validate_position(-100, -200).is_ok());
        assert!(validate_position(1000000, 1000000).is_ok());
    }

    #[test]
    fn test_validate_position_invalid() {
        assert!(validate_position(i32::MAX, 0).is_err());
        assert!(validate_position(0, i32::MAX).is_err());
        assert!(validate_position(i32::MIN, 0).is_err());
        assert!(validate_position(0, i32::MIN).is_err());
    }

    #[test]
    fn test_validate_size_valid() {
        assert!(validate_size(1, 1).is_ok());
        assert!(validate_size(100, 200).is_ok());
        assert!(validate_size(1920, 1080).is_ok());
    }

    #[test]
    fn test_validate_size_invalid() {
        assert!(validate_size(0, 100).is_err());
        assert!(validate_size(100, 0).is_err());
        assert!(validate_size(-1, 100).is_err());
        assert!(validate_size(100, -1).is_err());
        assert!(validate_size(0, 0).is_err());
    }

    #[test]
    fn test_validate_opacity_valid() {
        assert!(validate_opacity(0.0).is_ok());
        assert!(validate_opacity(0.5).is_ok());
        assert!(validate_opacity(1.0).is_ok());
    }

    #[test]
    fn test_validate_opacity_invalid() {
        assert!(validate_opacity(-0.1).is_err());
        assert!(validate_opacity(1.1).is_err());
        assert!(validate_opacity(f32::NAN).is_err());
    }

    #[test]
    fn test_validate_orientation_valid() {
        assert!(validate_orientation(0).is_ok());
        assert!(validate_orientation(90).is_ok());
        assert!(validate_orientation(180).is_ok());
        assert!(validate_orientation(270).is_ok());
        assert!(validate_orientation(360).is_ok());
        assert!(validate_orientation(-90).is_ok());
    }

    #[test]
    fn test_validate_orientation_invalid() {
        assert!(validate_orientation(45).is_err());
        assert!(validate_orientation(91).is_err());
        assert!(validate_orientation(1).is_err());
        assert!(validate_orientation(89).is_err());
    }

    #[test]
    fn test_validate_z_order_valid() {
        assert!(validate_z_order(0, 0, 10).is_ok());
        assert!(validate_z_order(5, 0, 10).is_ok());
        assert!(validate_z_order(10, 0, 10).is_ok());
    }

    #[test]
    fn test_validate_z_order_invalid() {
        assert!(validate_z_order(-1, 0, 10).is_err());
        assert!(validate_z_order(11, 0, 10).is_err());
    }
}
