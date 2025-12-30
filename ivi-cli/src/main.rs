//! IVI CLI - Command-line interface for Weston IVI Controller
//!
//! This tool provides a command-line interface to interact with the Weston IVI
//! Controller, allowing users to manage surfaces and layers from the terminal.

mod output;

use clap::{Parser, Subcommand};
use ivi_client::{IviClient, IviError, Result};
use std::result::Result as StdResult;

/// Command-line interface for Weston IVI Controller
#[derive(Parser)]
#[command(name = "ivi_cli")]
#[command(version = "0.1.0")]
#[command(about = "Command-line interface for Weston IVI Controller", long_about = None)]
#[command(author = "Weston IVI Controller Project")]
struct Cli {
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

impl From<ValidationError> for IviError {
    fn from(err: ValidationError) -> Self {
        IviError::RequestFailed {
            code: -32602, // Invalid params
            message: err.message,
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Validate opacity value is in range [0.0, 1.0]
fn validate_opacity(opacity: f32) -> StdResult<(), ValidationError> {
    if !(0.0..=1.0).contains(&opacity) {
        Err(ValidationError {
            message: format!("Opacity must be between 0.0 and 1.0, got: {}", opacity),
        })
    } else {
        Ok(())
    }
}

struct IviCli {
    client: IviClient,
}

impl IviCli {
    fn new(remote: Option<&str>) -> Result<Self> {
        Ok(IviCli {
            client: IviClient::new(remote)?,
        })
    }
    /// Handle surface list command
    fn handle_surface_list(&mut self) -> Result<String> {
        let surfaces = self.client.list_surfaces()?;
        Ok(output::format_surface_list(&surfaces))
    }

    /// Handle surface get-props command
    fn handle_surface_get_properties(&mut self, id: u32) -> Result<String> {
        let surface = self.client.get_surface(id)?;
        Ok(output::format_surface_properties(&surface))
    }

    /// Handle surface set-visibility command
    fn handle_surface_set_visibility(&mut self, id: u32, visible: bool) -> Result<String> {
        self.client.set_surface_visibility(id, visible, true)?;
        Ok(output::format_surface_visibility_success(id, visible))
    }

    /// Handle surface set-opacity command
    fn handle_surface_set_opacity(&mut self, id: u32, opacity: f32) -> Result<String> {
        validate_opacity(opacity)?;

        self.client.set_surface_opacity(id, opacity, true)?;
        Ok(output::format_surface_opacity_success(id, opacity))
    }

    /// Handle surface set-source-rect command
    fn handle_surface_set_source_rect(
        &mut self,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<String> {
        self.client
            .set_surface_source_rectangle(id, x, y, width, height, true)?;
        Ok(output::format_surface_source_rect_success(
            id, x, y, width, height,
        ))
    }

    fn handle_surface_set_dest_rect(
        &mut self,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<String> {
        self.client
            .set_surface_destination_rectangle(id, x, y, width, height, true)?;
        Ok(output::format_surface_dest_rect_success(
            id, x, y, width, height,
        ))
    }

    /// Handle surface set-z-order command
    fn handle_surface_set_z_order(&mut self, id: u32, z_order: i32) -> Result<String> {
        self.client.set_surface_z_order(id, z_order, true)?;
        Ok(output::format_surface_z_order_success(id, z_order))
    }

    /// Handle surface set-focus command
    fn handle_surface_set_focus(&mut self, id: u32) -> Result<String> {
        self.client.set_surface_focus(id, true)?;
        Ok(output::format_surface_focus_success(id))
    }

    /// Handle layer list command
    fn handle_layer_list(&mut self) -> Result<String> {
        let layers = self.client.list_layers()?;
        Ok(output::format_layer_list(&layers))
    }

    /// Handle layer get-props command
    fn handle_layer_get_properties(&mut self, id: u32) -> Result<String> {
        let layer = self.client.get_layer(id)?;
        Ok(output::format_layer_properties(&layer))
    }

    /// Handle layer create-layer command
    fn handle_layer_create_layer(&mut self, id: u32, width: i32, height: i32) -> Result<String> {
        self.client.create_layer(id, width, height, true)?;
        Ok(output::format_layer_create_success(id))
    }

    /// Handle layer destroy command
    fn handle_layer_destroy(&mut self, id: u32) -> Result<String> {
        self.client.destroy_layer(id, true)?;
        Ok(output::format_layer_destroy_success(id))
    }

    /// Handle layer set-source-rect command
    fn handle_layer_set_source_rect(
        &mut self,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<String> {
        self.client
            .set_layer_source_rectangle(id, x, y, width, height, true)?;
        Ok(output::format_layer_source_rect_success(
            id, x, y, width, height,
        ))
    }

    /// Handle layer set-dest-rect command
    fn handle_layer_set_dest_rect(
        &mut self,
        id: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<String> {
        self.client
            .set_layer_destination_rectangle(id, x, y, width, height, true)?;
        Ok(output::format_layer_dest_rect_success(
            id, x, y, width, height,
        ))
    }

    /// Handle layer set-visibility command
    fn handle_layer_set_visibility(&mut self, id: u32, visible: bool) -> Result<String> {
        self.client.set_layer_visibility(id, visible, true)?;
        Ok(output::format_layer_visibility_success(id, visible))
    }

    /// Handle layer set-opacity command
    fn handle_layer_set_opacity(&mut self, id: u32, opacity: f32) -> Result<String> {
        validate_opacity(opacity)?;

        self.client.set_layer_opacity(id, opacity, true)?;
        Ok(output::format_layer_opacity_success(id, opacity))
    }

    /// Handle layer set surfaces command
    fn handle_layer_set_surfaces(&mut self, layer_id: u32, surface_ids: &[u32]) -> Result<String> {
        self.client
            .set_surfaces_on_layer(layer_id, surface_ids, true)?;
        Ok(output::format_layer_set_surfaces_success(
            layer_id,
            surface_ids,
            true,
        ))
    }

    /// Handle layer add surface command
    fn handle_layer_add_surface(&mut self, layer_id: u32, surface_id: u32) -> Result<String> {
        self.client
            .add_surface_to_layer(layer_id, surface_id, true)?;
        Ok(output::format_layer_add_surface_success(
            layer_id, surface_id, true,
        ))
    }

    /// Handle layer remove surface command
    fn handle_layer_remove_surface(&mut self, layer_id: u32, surface_id: u32) -> Result<String> {
        self.client
            .remove_surface_from_layer(layer_id, surface_id, true)?;
        Ok(output::format_layer_remove_surface_success(
            layer_id, surface_id, true,
        ))
    }

    /// Handle layer get surfaces command
    fn handle_layer_get_surfaces(&mut self, layer_id: u32) -> Result<String> {
        let surface_ids = self.client.get_layer_surfaces(layer_id)?;
        Ok(output::format_layer_surfaces(layer_id, &surface_ids))
    }

    /// Handle screen list command
    fn handle_screen_list(&mut self) -> Result<String> {
        let screens = self.client.list_screens()?;
        Ok(output::format_screen_list(&screens))
    }

    /// Handle screen get properties command
    fn handle_screen_get_properties(&mut self, name: &str) -> Result<String> {
        let screen = self.client.get_screen(name)?;
        Ok(output::format_screen_properties(&screen))
    }

    /// Handle screen get layers command
    fn handle_screen_get_layers(&mut self, name: &str) -> Result<String> {
        let layer_ids = self.client.get_screen_layers(name)?;
        Ok(output::format_screen_layers(name, &layer_ids))
    }

    /// Handle get screens for layer command
    fn handle_screen_get_screens_for_layer(&mut self, layer_id: u32) -> Result<String> {
        let screen_names = self.client.get_layer_screens(layer_id)?;
        Ok(output::format_layer_screens(layer_id, &screen_names))
    }

    /// Handle screen set layers command
    fn handle_screen_set_layers(&mut self, name: &str, layer_ids: &[u32]) -> Result<String> {
        self.client.add_layers_to_screen(name, layer_ids, true)?;
        Ok(output::format_screen_set_layers_success(
            name, layer_ids, true,
        ))
    }

    /// Handle screen remove layer command
    fn handle_screen_remove_layer(&mut self, name: &str, layer_id: u32) -> Result<String> {
        self.client.remove_layer_from_screen(name, layer_id, true)?;
        Ok(output::format_screen_remove_layer_success(
            name, layer_id, true,
        ))
    }

    /// Handle hierarchical scene command
    fn handle_scene(&mut self) -> Result<String> {
        // Get all screens
        let screens = self.client.list_screens()?;
        let mut hierarchy = Vec::new();

        // Build hierarchical structure: screens -> layers -> surfaces
        for screen in screens {
            let layer_ids = self.client.get_screen_layers(&screen.name)?;
            let mut layers_data = Vec::new();

            for layer_id in layer_ids {
                let layer = self.client.get_layer(layer_id)?;
                let surface_ids = self.client.get_layer_surfaces(layer_id)?;
                let mut surfaces_data = Vec::new();

                for surface_id in surface_ids {
                    let surface = self.client.get_surface(surface_id)?;
                    surfaces_data.push(surface);
                }

                layers_data.push((layer, surfaces_data));
            }

            hierarchy.push((screen, layers_data));
        }

        Ok(output::format_hierarchical_scene(&hierarchy))
    }

    /// Handle commit command
    fn handle_commit(&mut self) -> Result<String> {
        self.client.commit()?;
        Ok(output::format_commit_success())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut ivi_cli = IviCli::new(None)?;

    match cli.command {
        Commands::Surface { command } => match command {
            SurfaceCommands::List => ivi_cli.handle_surface_list(),
            SurfaceCommands::GetProps { id } => ivi_cli.handle_surface_get_properties(id),
            SurfaceCommands::SetVisibility { id, visible } => {
                ivi_cli.handle_surface_set_visibility(id, visible)
            }
            SurfaceCommands::SetOpacity { id, opacity } => {
                ivi_cli.handle_surface_set_opacity(id, opacity)
            }
            SurfaceCommands::SetSourceRect {
                id,
                x,
                y,
                width,
                height,
            } => ivi_cli.handle_surface_set_source_rect(id, x, y, width, height),
            SurfaceCommands::SetDestRect {
                id,
                x,
                y,
                width,
                height,
            } => ivi_cli.handle_surface_set_dest_rect(id, x, y, width, height),
            SurfaceCommands::SetZOrder { id, z_order } => {
                ivi_cli.handle_surface_set_z_order(id, z_order)
            }
            SurfaceCommands::SetFocus { id } => ivi_cli.handle_surface_set_focus(id),
        },
        Commands::Layer { command } => match command {
            LayerCommands::List => ivi_cli.handle_layer_list(),
            LayerCommands::GetProps { id } => ivi_cli.handle_layer_get_properties(id),
            LayerCommands::Create { id, width, height } => {
                ivi_cli.handle_layer_create_layer(id, width, height)
            }
            LayerCommands::Destroy { id } => ivi_cli.handle_layer_destroy(id),
            LayerCommands::SetSourceRect {
                id,
                x,
                y,
                width,
                height,
            } => ivi_cli.handle_layer_set_source_rect(id, x, y, width, height),
            LayerCommands::SetDestRect {
                id,
                x,
                y,
                width,
                height,
            } => ivi_cli.handle_layer_set_dest_rect(id, x, y, width, height),
            LayerCommands::SetVisibility { id, visible } => {
                ivi_cli.handle_layer_set_visibility(id, visible)
            }
            LayerCommands::SetOpacity { id, opacity } => {
                ivi_cli.handle_layer_set_opacity(id, opacity)
            }
            LayerCommands::SetSurfaces {
                layer_id,
                surface_ids,
            } => ivi_cli.handle_layer_set_surfaces(layer_id, &surface_ids),
            LayerCommands::AddSurface {
                layer_id,
                surface_id,
            } => ivi_cli.handle_layer_add_surface(layer_id, surface_id),
            LayerCommands::RemoveSurface {
                layer_id,
                surface_id,
            } => ivi_cli.handle_layer_remove_surface(layer_id, surface_id),
            LayerCommands::GetSurfaces { layer_id } => ivi_cli.handle_layer_get_surfaces(layer_id),
        },
        Commands::Screen { command } => match command {
            ScreenCommands::List => ivi_cli.handle_screen_list(),
            ScreenCommands::GetProps { name } => ivi_cli.handle_screen_get_properties(&name),
            ScreenCommands::GetLayers { name } => ivi_cli.handle_screen_get_layers(&name),
            ScreenCommands::GetScreensForLayer { layer_id } => {
                ivi_cli.handle_screen_get_screens_for_layer(layer_id)
            }
            ScreenCommands::SetLayers { name, layer_ids } => {
                ivi_cli.handle_screen_set_layers(&name, &layer_ids)
            }
            ScreenCommands::RemoveLayer { name, layer_id } => {
                ivi_cli.handle_screen_remove_layer(&name, layer_id)
            }
        },
        Commands::Scene => ivi_cli.handle_scene(),
        Commands::Commit => ivi_cli.handle_commit(),
    }
    .map(|r| println!("{}", r))
    .map_err(|e| {
        eprintln!("{}", output::format_error(&e));

        // Avoid printing error twice
        std::process::exit(1);
    })
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
