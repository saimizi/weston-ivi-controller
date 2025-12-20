# Weston IVI Controller Configuration

This document describes the configuration options available for the Weston IVI Controller plugin, including the new automatic surface ID assignment feature.

## Command-Line Arguments

The plugin accepts configuration through command-line arguments passed to Weston. All arguments are optional and have sensible defaults.

### Transport Configuration

- `--socket-path=<path>`: Path to the UNIX domain socket (default: `/tmp/weston-ivi-controller.sock`)
- `--max-connections=<num>`: Maximum number of client connections (default: `10`)

### ID Assignment Configuration

The automatic surface ID assignment feature can be configured with the following arguments:

- `--id-start=<id>`: Starting ID for auto-assignment range (default: `0x10000000`)
  - Supports both decimal and hexadecimal (with `0x` prefix) values
  - Example: `--id-start=268435456` or `--id-start=0x10000000`

- `--id-max=<id>`: Maximum ID for auto-assignment range (default: `0xFFFFFFFE`)
  - Supports both decimal and hexadecimal (with `0x` prefix) values
  - Example: `--id-max=4294967294` or `--id-max=0xFFFFFFFE`

- `--id-invalid=<id>`: Invalid ID that triggers assignment (default: `0xFFFFFFFF`)
  - This is the ID value that the IVI compositor assigns to surfaces without explicit IDs
  - Example: `--id-invalid=0xFFFFFFFF`

- `--id-lock-timeout=<ms>`: Lock acquisition timeout in milliseconds (default: `5000`)
  - Controls how long the system waits to acquire locks during concurrent operations
  - Example: `--id-lock-timeout=10000`

- `--id-max-concurrent=<num>`: Maximum concurrent assignments (default: `10`)
  - Limits the number of simultaneous ID assignment operations
  - Example: `--id-max-concurrent=5`

- `--id-assignment-timeout=<ms>`: Assignment operation timeout in milliseconds (default: `10000`)
  - Maximum time allowed for a single ID assignment operation
  - Example: `--id-assignment-timeout=15000`

## Environment Variables

Configuration can also be set via environment variables. Environment variables are overridden by command-line arguments but take precedence over defaults.

All environment variables are prefixed with `WESTON_IVI_` to avoid conflicts:

- `WESTON_IVI_SOCKET_PATH`: Socket path
- `WESTON_IVI_MAX_CONNECTIONS`: Maximum connections
- `WESTON_IVI_ID_START`: ID assignment start ID
- `WESTON_IVI_ID_MAX`: ID assignment max ID
- `WESTON_IVI_ID_INVALID`: Invalid ID value
- `WESTON_IVI_ID_LOCK_TIMEOUT`: Lock timeout in milliseconds
- `WESTON_IVI_ID_MAX_CONCURRENT`: Maximum concurrent assignments
- `WESTON_IVI_ID_ASSIGNMENT_TIMEOUT`: Assignment timeout in milliseconds

## Configuration Examples

### Basic Usage

Load the plugin with default settings:

```ini
[core]
modules=ivi-controller.so,libweston_ivi_controller.so
```

### Custom Socket Path

```bash
weston --modules=libweston_ivi_controller.so -- --socket-path=/var/run/ivi-controller.sock
```

### Custom ID Assignment Range

```bash
weston --modules=libweston_ivi_controller.so -- \
  --id-start=0x20000000 \
  --id-max=0x2FFFFFFF \
  --id-lock-timeout=10000
```

### Environment Variable Configuration

```bash
export WESTON_IVI_SOCKET_PATH=/tmp/custom-ivi.sock
export WESTON_IVI_ID_START=0x30000000
export WESTON_IVI_ID_MAX=0x3FFFFFFF
export WESTON_IVI_ID_MAX_CONCURRENT=5

weston --modules=libweston_ivi_controller.so
```

### Production Configuration

For production environments with high concurrency:

```bash
weston --modules=libweston_ivi_controller.so -- \
  --socket-path=/var/run/ivi-controller.sock \
  --max-connections=50 \
  --id-start=0x10000000 \
  --id-max=0xEFFFFFFF \
  --id-lock-timeout=2000 \
  --id-max-concurrent=20 \
  --id-assignment-timeout=5000
```

## Configuration Validation

The plugin validates all configuration parameters at startup and will fail to load if invalid values are provided. Common validation errors include:

- Socket directory does not exist
- `max_connections` is 0 or exceeds 1000
- ID assignment range is invalid (start >= max)
- Invalid ID is within the assignment range
- Timeout values are 0 or excessively large
- Concurrency limits are 0 or exceed reasonable bounds

## Monitoring and Debugging

The plugin logs comprehensive information about configuration and ID assignment operations:

- Configuration values are logged at startup
- ID assignment operations are logged with timing information
- Statistics are logged during shutdown
- Error conditions are logged with detailed context

Use `RUST_LOG=weston_ivi_controller=debug` to enable detailed logging for troubleshooting configuration issues.