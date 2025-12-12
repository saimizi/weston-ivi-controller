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
    /// Set surface source rectangle (which part of buffer to display)
    SetSourceRect {
        /// Surface ID
        id: u32,
        /// X coordinate in buffer
        x: i32,
        /// Y coordinate in buffer
        y: i32,
        /// Width in pixels
        width: i32,
        /// Height in pixels
        height: i32,
    },
    /// Set surface destination rectangle (where and how to display on screen)
    SetDestRect {
        /// Surface ID
        id: u32,
        /// X coordinate on screen
        x: i32,
        /// Y coordinate on screen
        y: i32,
        /// Width in pixels
        width: i32,
        /// Height in pixels
        height: i32,
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
    CreateLayer {
        /// Layer ID
        id: u32,
        /// Width in pixels
        width: i32,
        /// Height in pixels
        height: i32,
    },
    /// Set layer source rectangle
    SetSourceRect {
        /// Layer ID
        id: u32,
        /// X coordinate
        x: i32,
        /// Y coordinate
        y: i32,
        /// Width in pixels
        width: i32,
        /// Height in pixels
        height: i32,
    },
    /// Set layer destination rectangle
    SetDestRect {
        /// Layer ID
        id: u32,
        /// X coordinate
        x: i32,
        /// Y coordinate
        y: i32,
        /// Width in pixels
        width: i32,
        /// Height in pixels
        height: i32,
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
fn handle_surface_set_source_rect(
    socket_path: &str,
    id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_surface_source_rectangle(id, x, y, width, height)?;
    Ok(output::format_surface_source_rect_success(
        id, x, y, width, height,
    ))
}

fn handle_surface_set_dest_rect(
    socket_path: &str,
    id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_surface_destination_rectangle(id, x, y, width, height)?;
    Ok(output::format_surface_dest_rect_success(
        id, x, y, width, height,
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

/// Handle layer create-layer command
fn handle_layer_create_layer(
    socket_path: &str,
    id: u32,
    width: i32,
    height: i32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.create_layer(id, width, height, true)?;
    Ok(output::format_layer_create_success(id))
}

/// Handle layer set-source-rect command
fn handle_layer_set_source_rect(
    socket_path: &str,
    id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_layer_source_rectangle(id, x, y, width, height)?;
    Ok(output::format_layer_source_rect_success(
        id, x, y, width, height,
    ))
}

/// Handle layer set-dest-rect command
fn handle_layer_set_dest_rect(
    socket_path: &str,
    id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_layer_destination_rectangle(id, x, y, width, height)?;
    Ok(output::format_layer_dest_rect_success(
        id, x, y, width, height,
    ))
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
            SurfaceCommands::SetSourceRect {
                id,
                x,
                y,
                width,
                height,
            } => handle_surface_set_source_rect(&cli.socket, id, x, y, width, height),
            SurfaceCommands::SetDestRect {
                id,
                x,
                y,
                width,
                height,
            } => handle_surface_set_dest_rect(&cli.socket, id, x, y, width, height),
            SurfaceCommands::SetZOrder { id, z_order } => {
                handle_surface_set_z_order(&cli.socket, id, z_order)
            }
            SurfaceCommands::SetFocus { id } => handle_surface_set_focus(&cli.socket, id),
        },
        Commands::Layer { command } => match command {
            LayerCommands::List => handle_layer_list(&cli.socket),
            LayerCommands::GetProperties { id } => handle_layer_get_properties(&cli.socket, id),
            LayerCommands::CreateLayer { id, width, height } => {
                handle_layer_create_layer(&cli.socket, id, width, height)
            }
            LayerCommands::SetSourceRect {
                id,
                x,
                y,
                width,
                height,
            } => handle_layer_set_source_rect(&cli.socket, id, x, y, width, height),
            LayerCommands::SetDestRect {
                id,
                x,
                y,
                width,
                height,
            } => handle_layer_set_dest_rect(&cli.socket, id, x, y, width, height),
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
}
