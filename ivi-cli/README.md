# IVI CLI

A command-line interface tool for controlling the Weston IVI (In-Vehicle Infotainment) compositor. This tool provides an easy way to interact with IVI surfaces and layers for testing, debugging, and managing compositor layouts.

## Features

- List and query surfaces and layers
- Control surface visibility, opacity, position, size, orientation, and z-order
- Control layer visibility and opacity
- Atomic commits for consistent updates
- Clear error messages and help text
- Configurable socket path

## Installation

### From Source

```bash
cd ivi-cli
cargo build --release
```

The compiled binary will be at `target/release/ivi_cli`.

### Install to System

```bash
cargo install --path .
```

This installs `ivi_cli` to `~/.cargo/bin/` (ensure it's in your PATH).

## Usage

```
ivi_cli [OPTIONS] <COMMAND>
```

### Global Options

- `--socket <PATH>` - Custom socket path (default: `/tmp/weston-ivi-controller.sock`)
- `--help` - Display help information
- `--version` - Display version information

### Commands

The CLI is organized into resource-based commands:

- `surface` - Surface management commands
- `layer` - Layer management commands
- `commit` - Commit pending changes

## Surface Commands

### List Surfaces

List all available surfaces:

```bash
ivi_cli surface list
```

Example output:
```
Surface IDs: 1000, 1001, 1002
```

### Get Surface Properties

Display detailed properties of a specific surface:

```bash
ivi_cli surface get-properties <SURFACE_ID>
```

Example:
```bash
ivi_cli surface get-properties 1000
```

Example output:
```
Surface 1000:
  Position: (100, 200)
  Size: 1920x1080
  Visibility: true
  Opacity: 1.0
  Orientation: Normal
  Z-Order: 0
```

### Set Surface Visibility

Show or hide a surface:

```bash
ivi_cli surface set-visibility <SURFACE_ID> <true|false>
```

Examples:
```bash
ivi_cli surface set-visibility 1000 true
ivi_cli surface set-visibility 1000 false
```

### Set Surface Opacity

Adjust surface transparency (0.0 = transparent, 1.0 = opaque):

```bash
ivi_cli surface set-opacity <SURFACE_ID> <OPACITY>
```

Examples:
```bash
ivi_cli surface set-opacity 1000 1.0
ivi_cli surface set-opacity 1000 0.5
ivi_cli surface set-opacity 1000 0.0
```

### Set Surface Destination Rectangle

Set the position and size of a surface:

```bash
ivi_cli surface set-dest-rect <SURFACE_ID> <X> <Y> <WIDTH> <HEIGHT>
```

Example:
```bash
ivi_cli surface set-dest-rect 1000 100 200 1920 1080
```

### Set Surface Orientation

Rotate a surface:

```bash
ivi_cli surface set-orientation <SURFACE_ID> <ORIENTATION>
```

Orientations: `normal`, `rotate90`, `rotate180`, `rotate270`

Example:
```bash
ivi_cli surface set-orientation 1000 rotate90
```

### Set Surface Z-Order

Control the stacking order of surfaces:

```bash
ivi_cli surface set-z-order <SURFACE_ID> <Z_ORDER>
```

Example:
```bash
ivi_cli surface set-z-order 1000 10
```

### Set Surface Focus

Route keyboard and pointer input to a surface:

```bash
ivi_cli surface set-focus <SURFACE_ID>
```

Example:
```bash
ivi_cli surface set-focus 1000
```

## Layer Commands

### List Layers

List all available layers:

```bash
ivi_cli layer list
```

Example output:
```
Layer IDs: 2000, 2001
```

### Get Layer Properties

Display detailed properties of a specific layer:

```bash
ivi_cli layer get-properties <LAYER_ID>
```

Example:
```bash
ivi_cli layer get-properties 2000
```

Example output:
```
Layer 2000:
  Visibility: true
  Opacity: 1.0
```

### Set Layer Visibility

Show or hide a layer:

```bash
ivi_cli layer set-visibility <LAYER_ID> <true|false>
```

Examples:
```bash
ivi_cli layer set-visibility 2000 true
ivi_cli layer set-visibility 2000 false
```

### Set Layer Opacity

Adjust layer transparency (0.0 = transparent, 1.0 = opaque):

```bash
ivi_cli layer set-opacity <LAYER_ID> <OPACITY>
```

Examples:
```bash
ivi_cli layer set-opacity 2000 1.0
ivi_cli layer set-opacity 2000 0.8
```

## Commit Command

Apply all pending changes atomically:

```bash
ivi_cli commit
```

This ensures that multiple modifications are applied simultaneously without visual artifacts.

### Auto-Commit Flag

Most modification commands support the `--commit` flag to automatically commit changes:

```bash
ivi_cli surface set-visibility 1000 true --commit
ivi_cli surface set-opacity 1000 0.8 --commit
```

## Examples

### Basic Workflow

```bash
# List all surfaces
ivi_cli surface list

# Get properties of surface 1000
ivi_cli surface get-properties 1000

# Make surface visible and set opacity
ivi_cli surface set-visibility 1000 true
ivi_cli surface set-opacity 1000 0.9

# Commit changes
ivi_cli commit
```

### Batch Operations

```bash
# Configure multiple properties
ivi_cli surface set-dest-rect 1000 0 0 1920 1080
ivi_cli surface set-visibility 1000 true
ivi_cli surface set-opacity 1000 1.0
ivi_cli surface set-z-order 1000 10

# Apply all changes at once
ivi_cli commit
```

### Using Auto-Commit

```bash
# Each command commits immediately
ivi_cli surface set-visibility 1000 true --commit
ivi_cli layer set-opacity 2000 0.8 --commit
```

### Custom Socket Path

```bash
# Use a different socket path
ivi_cli --socket /var/run/ivi-controller.sock surface list
```

### Layer Management

```bash
# List all layers
ivi_cli layer list

# Configure layer
ivi_cli layer set-visibility 2000 true
ivi_cli layer set-opacity 2000 0.9
ivi_cli commit
```

## Error Handling

The CLI provides clear error messages for common issues:

### Connection Failed

```
✗ Error: Connection failed: No such file or directory (os error 2)
```

**Solution**: Ensure the Weston IVI controller is running and the socket path is correct.

### Surface Not Found

```
✗ Error: Request failed (code -32000): Surface not found: 1234
```

**Solution**: Verify the surface ID exists using `ivi_cli surface list`.

### Invalid Parameter

```
✗ Error: Opacity must be between 0.0 and 1.0
```

**Solution**: Check the parameter constraints in the help text.

## Exit Codes

- `0` - Success
- `1` - Error (connection failed, invalid parameters, request failed)

## Environment Variables

- `IVI_SOCKET` - Default socket path (overrides built-in default)

Example:
```bash
export IVI_SOCKET=/var/run/ivi-controller.sock
ivi_cli surface list
```

## Requirements

- Weston compositor with IVI shell support
- IVI controller plugin running and listening on UNIX socket
- Rust 1.70 or later (for building from source)

## Help

Get help for any command:

```bash
# General help
ivi_cli --help

# Surface commands help
ivi_cli surface --help

# Layer commands help
ivi_cli layer --help

# Specific command help
ivi_cli surface set-visibility --help
```

## Troubleshooting

### Command Not Found

Ensure `~/.cargo/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Permission Denied

The socket file may have restricted permissions. Check the socket permissions:

```bash
ls -l /tmp/weston-ivi-controller.sock
```

### Controller Not Running

Start the Weston compositor with the IVI controller plugin:

```bash
weston --config=/path/to/weston.ini
```

Ensure `weston.ini` includes:

```ini
[core]
shell=ivi-shell.so
modules=weston-ivi-controller.so
```

## License

MIT

## See Also

- [IVI Client Library](../ivi-client/README.md) - The underlying library used by this CLI
- [Weston IVI Controller](../README.md) - The controller plugin documentation
