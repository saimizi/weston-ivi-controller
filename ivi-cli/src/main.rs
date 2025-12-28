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
    /// Screen management commands
    Screen {
        #[command(subcommand)]
        command: ScreenCommands,
    },
    /// Display complete scene hierarchy
    Scene,
    /// Commit pending changes atomically
    Commit,
}

/// Surface management commands
#[derive(Subcommand)]
enum SurfaceCommands {
    /// List all available surfaces
    List,
    /// Get properties of a specific surface
    GetProps {
        /// Surface ID
        id: u32,
    },
    /// Set surface visibility
    SetVisibility {
        /// Surface ID
        id: u32,
        /// Visibility (true or false)
        #[arg(action = clap::ArgAction::Set)]
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
    GetProps {
        /// Layer ID
        id: u32,
    },
    /// Create a new layer with specified dimensions
    Create {
        /// Layer ID
        id: u32,
        /// Width in pixels
        width: i32,
        /// Height in pixels
        height: i32,
    },
    /// Destroy an existing layer
    Destroy {
        /// Layer ID
        id: u32,
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
        #[arg(action = clap::ArgAction::Set)]
        visible: bool,
    },
    /// Set layer opacity
    SetOpacity {
        /// Layer ID
        id: u32,
        /// Opacity value (0.0 to 1.0)
        opacity: f32,
    },
    /// Set surfaces on a layer (replaces all existing surfaces)
    SetSurfaces {
        /// Layer ID
        layer_id: u32,
        /// Comma-separated list of surface IDs (first=bottommost, last=topmost)
        #[arg(value_delimiter = ',')]
        surface_ids: Vec<u32>,
    },
    /// Add a single surface to a layer as topmost
    AddSurface {
        /// Layer ID
        layer_id: u32,
        /// Surface ID to add
        surface_id: u32,
    },
    /// Remove a surface from a layer
    RemoveSurface {
        /// Layer ID
        layer_id: u32,
        /// Surface ID to remove
        surface_id: u32,
    },
    /// List surfaces on a layer
    GetSurfaces {
        /// Layer ID
        layer_id: u32,
    },
}

/// Screen management commands
#[derive(Subcommand)]
enum ScreenCommands {
    /// List all available screens
    List,
    /// Get properties of a specific screen
    GetProps {
        /// Screen name (e.g., "HDMI-A-1")
        name: String,
    },
    /// Get layers assigned to a screen
    GetLayers {
        /// Screen name
        name: String,
    },
    /// Get screens assigned to a layer
    GetScreensForLayer {
        /// Layer ID
        layer_id: u32,
    },
    /// Set layers on a screen (replaces all existing layers)
    SetLayers {
        /// Screen name
        name: String,
        /// Comma-separated list of layer IDs
        #[arg(value_delimiter = ',')]
        layer_ids: Vec<u32>,
    },
    /// Remove a layer from a screen
    RemoveLayer {
        /// Screen name
        name: String,
        /// Layer ID to remove
        layer_id: u32,
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
    if !(0.0..=1.0).contains(&opacity) {
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

/// Handle surface get-props command
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
    client.set_surface_visibility(id, visible, true)?;
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
    client.set_surface_opacity(id, opacity, true)?;
    Ok(output::format_surface_opacity_success(id, opacity))
}

/// Handle surface set-source-rect command
fn handle_surface_set_source_rect(
    socket_path: &str,
    id: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_surface_source_rectangle(id, x, y, width, height, true)?;
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
    client.set_surface_destination_rectangle(id, x, y, width, height, true)?;
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
    client.set_surface_z_order(id, z_order, true)?;
    Ok(output::format_surface_z_order_success(id, z_order))
}

/// Handle surface set-focus command
fn handle_surface_set_focus(
    socket_path: &str,
    id: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_surface_focus(id, true)?;
    Ok(output::format_surface_focus_success(id))
}

/// Handle layer list command
fn handle_layer_list(socket_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    let layers = client.list_layers()?;
    Ok(output::format_layer_list(&layers))
}

/// Handle layer get-props command
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

/// Handle layer destroy command
fn handle_layer_destroy(socket_path: &str, id: u32) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.destroy_layer(id, true)?;
    Ok(output::format_layer_destroy_success(id))
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
    client.set_layer_source_rectangle(id, x, y, width, height, true)?;
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
    client.set_layer_destination_rectangle(id, x, y, width, height, true)?;
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
    client.set_layer_visibility(id, visible, true)?;
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
    client.set_layer_opacity(id, opacity, true)?;
    Ok(output::format_layer_opacity_success(id, opacity))
}

/// Handle layer set surfaces command
fn handle_layer_set_surfaces(
    socket_path: &str,
    layer_id: u32,
    surface_ids: &[u32],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.set_surfaces_on_layer(layer_id, surface_ids, true)?;
    Ok(output::format_layer_set_surfaces_success(
        layer_id,
        surface_ids,
        true,
    ))
}

/// Handle layer add surface command
fn handle_layer_add_surface(
    socket_path: &str,
    layer_id: u32,
    surface_id: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.add_surface_to_layer(layer_id, surface_id, true)?;
    Ok(output::format_layer_add_surface_success(
        layer_id, surface_id, true,
    ))
}

/// Handle layer remove surface command
fn handle_layer_remove_surface(
    socket_path: &str,
    layer_id: u32,
    surface_id: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.remove_surface_from_layer(layer_id, surface_id, true)?;
    Ok(output::format_layer_remove_surface_success(
        layer_id, surface_id, true,
    ))
}

/// Handle layer get surfaces command
fn handle_layer_get_surfaces(
    socket_path: &str,
    layer_id: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    let surface_ids = client.get_layer_surfaces(layer_id)?;
    Ok(output::format_layer_surfaces(layer_id, &surface_ids))
}

/// Handle screen list command
fn handle_screen_list(socket_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    let screens = client.list_screens()?;
    Ok(output::format_screen_list(&screens))
}

/// Handle screen get properties command
fn handle_screen_get_properties(
    socket_path: &str,
    name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    let screen = client.get_screen(name)?;
    Ok(output::format_screen_properties(&screen))
}

/// Handle screen get layers command
fn handle_screen_get_layers(
    socket_path: &str,
    name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    let layer_ids = client.get_screen_layers(name)?;
    Ok(output::format_screen_layers(name, &layer_ids))
}

/// Handle get screens for layer command
fn handle_screen_get_screens_for_layer(
    socket_path: &str,
    layer_id: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    let screen_names = client.get_layer_screens(layer_id)?;
    Ok(output::format_layer_screens(layer_id, &screen_names))
}

/// Handle screen set layers command
fn handle_screen_set_layers(
    socket_path: &str,
    name: &str,
    layer_ids: &[u32],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.add_layers_to_screen(name, layer_ids, true)?;
    Ok(output::format_screen_set_layers_success(
        name, layer_ids, true,
    ))
}

/// Handle screen remove layer command
fn handle_screen_remove_layer(
    socket_path: &str,
    name: &str,
    layer_id: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;
    client.remove_layer_from_screen(name, layer_id, true)?;
    Ok(output::format_screen_remove_layer_success(
        name, layer_id, true,
    ))
}

/// Handle hierarchical scene command
fn handle_scene(socket_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ivi_client::IviClient::connect(socket_path)?;

    // Get all screens
    let screens = client.list_screens()?;
    let mut hierarchy = Vec::new();

    // Build hierarchical structure: screens -> layers -> surfaces
    for screen in screens {
        let layer_ids = client.get_screen_layers(&screen.name)?;
        let mut layers_data = Vec::new();

        for layer_id in layer_ids {
            let layer = client.get_layer(layer_id)?;
            let surface_ids = client.get_layer_surfaces(layer_id)?;
            let mut surfaces_data = Vec::new();

            for surface_id in surface_ids {
                let surface = client.get_surface(surface_id)?;
                surfaces_data.push(surface);
            }

            layers_data.push((layer, surfaces_data));
        }

        hierarchy.push((screen, layers_data));
    }

    Ok(output::format_hierarchical_scene(&hierarchy))
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
            SurfaceCommands::GetProps { id } => handle_surface_get_properties(&cli.socket, id),
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
            LayerCommands::GetProps { id } => handle_layer_get_properties(&cli.socket, id),
            LayerCommands::Create { id, width, height } => {
                handle_layer_create_layer(&cli.socket, id, width, height)
            }
            LayerCommands::Destroy { id } => handle_layer_destroy(&cli.socket, id),
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
            LayerCommands::SetSurfaces {
                layer_id,
                surface_ids,
            } => handle_layer_set_surfaces(&cli.socket, layer_id, &surface_ids),
            LayerCommands::AddSurface {
                layer_id,
                surface_id,
            } => handle_layer_add_surface(&cli.socket, layer_id, surface_id),
            LayerCommands::RemoveSurface {
                layer_id,
                surface_id,
            } => handle_layer_remove_surface(&cli.socket, layer_id, surface_id),
            LayerCommands::GetSurfaces { layer_id } => {
                handle_layer_get_surfaces(&cli.socket, layer_id)
            }
        },
        Commands::Screen { command } => match command {
            ScreenCommands::List => handle_screen_list(&cli.socket),
            ScreenCommands::GetProps { name } => handle_screen_get_properties(&cli.socket, &name),
            ScreenCommands::GetLayers { name } => handle_screen_get_layers(&cli.socket, &name),
            ScreenCommands::GetScreensForLayer { layer_id } => {
                handle_screen_get_screens_for_layer(&cli.socket, layer_id)
            }
            ScreenCommands::SetLayers { name, layer_ids } => {
                handle_screen_set_layers(&cli.socket, &name, &layer_ids)
            }
            ScreenCommands::RemoveLayer { name, layer_id } => {
                handle_screen_remove_layer(&cli.socket, &name, layer_id)
            }
        },
        Commands::Scene => handle_scene(&cli.socket),
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
