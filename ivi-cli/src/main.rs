//! IVI CLI - Command-line interface for Weston IVI Controller
//!
//! This tool provides a command-line interface to interact with the Weston IVI
//! Controller, allowing users to manage surfaces and layers from the terminal.

mod output;

use clap::{Parser, Subcommand};

/// Command-line interface for Weston IVI Controller
#[derive(Parser)]
#[command(name = "ivi_cli")]
#[command(version = "0.1.0")]
#[command(about = "Command-line interface for Weston IVI Controller", long_about = None)]
#[command(author = "Weston IVI Controller Project")]
struct Cli {
    /// Path to the UNIX domain socket
    #[arg(long, default_value = "/tmp/weston-ivi-controller.sock")]
    socket: String,

    #[command(subcommand)]
    command: Commands,
}

/// Available commands
#[derive(Subcommand)]
enum Commands {
    /// Surface management commands
    Surface {
        #[command(subcommand)]
        command: SurfaceCommands,
    },
    /// Layer management commands
    Layer {
        #[command(subcommand)]
        command: LayerCommands,
    },
    /// Commit pending changes atomically
    Commit,
}

/// Surface management commands
#[derive(Subcommand)]
enum SurfaceCommands {
    /// List all available surfaces
    List,
    /// Get properties of a specific surface
    GetProperties {
        /// Surface ID
        id: u32,
    },
    /// Set surface visibility
    SetVisibility {
        /// Surface ID
        id: u32,
        /// Visibility (true or false)
        visible: bool,
    },
    /// Set surface opacity
    SetOpacity {
        /// Surface ID
        id: u32,
        /// Opacity value (0.0 to 1.0)
        opacity: f32,
    },
    /// Set surface destination rectangle (position and size)
    SetDestRect {
        /// Surface ID
        id: u32,
        /// X coordinate
        x: i32,
        /// Y coordinate
        y: i32,
        /// Width in pixels
        width: u32,
        /// Height in pixels
        height: u32,
    },
    /// Set surface orientation
    SetOrientation {
        /// Surface ID
        id: u32,
        /// Orientation (normal, rotate90, rotate180, rotate270)
        orientation: String,
    },
    /// Set surface z-order
    SetZOrder {
        /// Surface ID
        id: u32,
        /// Z-order value
        z_order: i32,
    },
    /// Set surface focus
    SetFocus {
        /// Surface ID
        id: u32,
    },
}

/// Layer management commands
#[derive(Subcommand)]
enum LayerCommands {
    /// List all available layers
    List,
    /// Get properties of a specific layer
    GetProperties {
        /// Layer ID
        id: u32,
    },
    /// Set layer visibility
    SetVisibility {
        /// Layer ID
        id: u32,
        /// Visibility (true or false)
        visible: bool,
    },
    /// Set layer opacity
    SetOpacity {
        /// Layer ID
        id: u32,
        /// Opacity value (0.0 to 1.0)
        opacity: f32,
    },
}

/// Validation error type
#[derive(Debug)]
struct ValidationError {
    message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Validate opacity value is in range [0.0, 1.0]
fn validate_opacity(opacity: f32) -> Result<(), ValidationError> {
    if opacity < 0.0 || opacity > 1.0 {
        Err(ValidationError {
            message: format!("Opacity must be between 0.0 and 1.0, got: {}", opacity),
        })
    } else {
        Ok(())
    }
}

/// Validate width is positive
fn validate_width(width: u32) -> Result<(), ValidationError> {
    if width == 0 {
        Err(ValidationError {
            message: "Width must be a positive integer".to_string(),
        })
    } else {
        Ok(())
    }
}

/// Validate height is positive
fn validate_height(height: u32) -> Result<(), ValidationError> {
    if height == 0 {
        Err(ValidationError {
            message: "Height must be a positive integer".to_string(),
        })
    } else {
        Ok(())
    }
}

/// Validate orientation string and convert to Orientation enum
fn validate_orientation(orientation: &str) -> Result<ivi_client::Orientation, ValidationError> {
    match orientation.to_lowercase().as_str() {
        "normal" => Ok(ivi_client::Orientation::Normal),
        "rotate90" => Ok(ivi_client::Orientation::Rotate90),
        "rotate180" => Ok(ivi_client::Orientation::Rotate180),
        "rotate270" => Ok(ivi_client::Orientation::Rotate270),
        _ => Err(ValidationError {
            message: format!(
                "Invalid orientation '{}'. Valid values are: normal, rotate90, rotate180, rotate270",
                orientation
            ),
        }),
    }
}

/// Handle surface list command
fn handle_surface_list(socket_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    let surfaces = client.list_surfaces()?;
    Ok(output::format_surface_list(&surfaces))
}

/// Handle surface get-properties command
fn handle_surface_get_properties(
    socket_path: &str,
    id: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    let surface = client.get_surface(id)?;
    Ok(output::format_surface_properties(&surface))
}

/// Handle surface set-visibility command
fn handle_surface_set_visibility(
    socket_path: &str,
    id: u32,
    visible: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_surface_visibility(id, visible)?;
    Ok(output::format_surface_visibility_success(id, visible))
}

/// Handle surface set-opacity command
fn handle_surface_set_opacity(
    socket_path: &str,
    id: u32,
    opacity: f32,
) -> Result<String, Box<dyn std::error::Error>> {
    validate_opacity(opacity)?;

    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_surface_opacity(id, opacity)?;
    Ok(output::format_surface_opacity_success(id, opacity))
}

/// Handle surface set-dest-rect command
fn handle_surface_set_dest_rect(
    socket_path: &str,
    id: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    validate_width(width)?;
    validate_height(height)?;

    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_surface_position(id, x, y)?;
    client.set_surface_size(id, width, height)?;
    Ok(output::format_surface_dest_rect_success(
        id, x, y, width, height,
    ))
}

/// Handle surface set-orientation command
fn handle_surface_set_orientation(
    socket_path: &str,
    id: u32,
    orientation: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let orientation_enum = validate_orientation(orientation)?;

    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_surface_orientation(id, orientation_enum)?;
    Ok(output::format_surface_orientation_success(
        id,
        &orientation_enum.to_string(),
    ))
}

/// Handle surface set-z-order command
fn handle_surface_set_z_order(
    socket_path: &str,
    id: u32,
    z_order: i32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_surface_z_order(id, z_order)?;
    Ok(output::format_surface_z_order_success(id, z_order))
}

/// Handle surface set-focus command
fn handle_surface_set_focus(
    socket_path: &str,
    id: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_surface_focus(id)?;
    Ok(output::format_surface_focus_success(id))
}

/// Handle layer list command
fn handle_layer_list(socket_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    let layers = client.list_layers()?;
    Ok(output::format_layer_list(&layers))
}

/// Handle layer get-properties command
fn handle_layer_get_properties(
    socket_path: &str,
    id: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    let layer = client.get_layer(id)?;
    Ok(output::format_layer_properties(&layer))
}

/// Handle layer set-visibility command
fn handle_layer_set_visibility(
    socket_path: &str,
    id: u32,
    visible: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_layer_visibility(id, visible)?;
    Ok(output::format_layer_visibility_success(id, visible))
}

/// Handle layer set-opacity command
fn handle_layer_set_opacity(
    socket_path: &str,
    id: u32,
    opacity: f32,
) -> Result<String, Box<dyn std::error::Error>> {
    validate_opacity(opacity)?;

    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_layer_opacity(id, opacity)?;
    Ok(output::format_layer_opacity_success(id, opacity))
}

/// Handle commit command
fn handle_commit(socket_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.commit()?;
    Ok(output::format_commit_success())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Surface { command } => match command {
            SurfaceCommands::List => handle_surface_list(&cli.socket),
            SurfaceCommands::GetProperties { id } => handle_surface_get_properties(&cli.socket, id),
            SurfaceCommands::SetVisibility { id, visible } => {
                handle_surface_set_visibility(&cli.socket, id, visible)
            }
            SurfaceCommands::SetOpacity { id, opacity } => {
                handle_surface_set_opacity(&cli.socket, id, opacity)
            }
            SurfaceCommands::SetDestRect {
                id,
                x,
                y,
                width,
                height,
            } => handle_surface_set_dest_rect(&cli.socket, id, x, y, width, height),
            SurfaceCommands::SetOrientation { id, orientation } => {
                handle_surface_set_orientation(&cli.socket, id, &orientation)
            }
            SurfaceCommands::SetZOrder { id, z_order } => {
                handle_surface_set_z_order(&cli.socket, id, z_order)
            }
            SurfaceCommands::SetFocus { id } => handle_surface_set_focus(&cli.socket, id),
        },
        Commands::Layer { command } => match command {
            LayerCommands::List => handle_layer_list(&cli.socket),
            LayerCommands::GetProperties { id } => handle_layer_get_properties(&cli.socket, id),
            LayerCommands::SetVisibility { id, visible } => {
                handle_layer_set_visibility(&cli.socket, id, visible)
            }
            LayerCommands::SetOpacity { id, opacity } => {
                handle_layer_set_opacity(&cli.socket, id, opacity)
            }
        },
        Commands::Commit => handle_commit(&cli.socket),
    };

    match result {
        Ok(output) => {
            println!("{}", output);
            std::process::exit(0);
        }
        Err(err) => {
            eprintln!("{}", output::format_error(err.as_ref()));
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(validate_opacity(-1.0).is_err());
        assert!(validate_opacity(2.0).is_err());
    }

    #[test]
    fn test_validate_width_valid() {
        assert!(validate_width(1).is_ok());
        assert!(validate_width(1920).is_ok());
        assert!(validate_width(u32::MAX).is_ok());
    }

    #[test]
    fn test_validate_width_invalid() {
        assert!(validate_width(0).is_err());
    }

    #[test]
    fn test_validate_height_valid() {
        assert!(validate_height(1).is_ok());
        assert!(validate_height(1080).is_ok());
        assert!(validate_height(u32::MAX).is_ok());
    }

    #[test]
    fn test_validate_height_invalid() {
        assert!(validate_height(0).is_err());
    }

    #[test]
    fn test_validate_orientation_valid() {
        assert!(validate_orientation("normal").is_ok());
        assert!(validate_orientation("Normal").is_ok());
        assert!(validate_orientation("NORMAL").is_ok());
        assert!(validate_orientation("rotate90").is_ok());
        assert!(validate_orientation("Rotate90").is_ok());
        assert!(validate_orientation("rotate180").is_ok());
        assert!(validate_orientation("rotate270").is_ok());
    }

    #[test]
    fn test_validate_orientation_invalid() {
        assert!(validate_orientation("invalid").is_err());
        assert!(validate_orientation("rotate45").is_err());
        assert!(validate_orientation("").is_err());
    }

    #[test]
    fn test_validate_orientation_returns_correct_enum() {
        assert_eq!(
            validate_orientation("normal").unwrap(),
            ivi_client::Orientation::Normal
        );
        assert_eq!(
            validate_orientation("rotate90").unwrap(),
            ivi_client::Orientation::Rotate90
        );
        assert_eq!(
            validate_orientation("rotate180").unwrap(),
            ivi_client::Orientation::Rotate180
        );
        assert_eq!(
            validate_orientation("rotate270").unwrap(),
            ivi_client::Orientation::Rotate270
        );
    }
}
