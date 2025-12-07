/**
 * C API Example for IVI Client Library
 *
 * This example demonstrates how to use the IVI client library from C to:
 * - Connect to the IVI controller
 * - List and query surfaces and layers
 * - Modify surface and layer properties
 * - Handle errors and manage memory properly
 *
 * Compilation:
 *   gcc -o c_example c_example.c -L../target/release -livi_client -lpthread -ldl -lm
 *
 * Usage:
 *   export LD_LIBRARY_PATH=../target/release:$LD_LIBRARY_PATH
 *   ./c_example
 */

#include "../include/ivi_client.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Function prototypes
int demonstrate_surface_operations(IviClient* client);
int demonstrate_layer_operations(IviClient* client);
void demonstrate_error_handling(IviClient* client);
const char* orientation_to_string(IviOrientation orientation);

int main(void) {
    char error_buf[256];
    int exit_code = 0;

    printf("=== IVI Client Library - C Example ===\n\n");

    // Get socket path from environment or use default
    const char* socket_path = getenv("IVI_SOCKET");
    if (socket_path == NULL) {
        socket_path = "/tmp/weston-ivi-controller.sock";
    }

    // Connect to the IVI controller
    printf("Connecting to IVI controller...\n");
    IviClient* client = ivi_client_connect(socket_path, error_buf, sizeof(error_buf));
    
    if (client == NULL) {
        fprintf(stderr, "✗ Connection failed: %s\n", error_buf);
        fprintf(stderr, "\nMake sure the Weston IVI controller is running and listening on:\n");
        fprintf(stderr, "  %s\n", socket_path);
        return 1;
    }
    
    printf("✓ Connected to %s\n\n", socket_path);

    // Demonstrate surface operations
    if (demonstrate_surface_operations(client) != 0) {
        exit_code = 1;
        goto cleanup;
    }

    // Demonstrate layer operations
    if (demonstrate_layer_operations(client) != 0) {
        exit_code = 1;
        goto cleanup;
    }

    // Demonstrate error handling
    demonstrate_error_handling(client);

    printf("\n=== Example completed successfully ===\n");

cleanup:
    // Clean up connection
    ivi_client_disconnect(client);
    return exit_code;
}

int demonstrate_surface_operations(IviClient* client) {
    char error_buf[256];
    IviSurface* surfaces = NULL;
    size_t surface_count = 0;
    IviErrorCode result;

    printf("--- Surface Operations ---\n\n");

    // List all surfaces
    printf("Listing all surfaces...\n");
    result = ivi_list_surfaces(client, &surfaces, &surface_count, error_buf, sizeof(error_buf));
    
    if (result != IVI_OK) {
        fprintf(stderr, "✗ Failed to list surfaces: %s\n", error_buf);
        return 1;
    }

    if (surface_count == 0) {
        printf("  No surfaces found\n");
        return 0;
    }

    printf("  Found %zu surface(s):\n", surface_count);
    for (size_t i = 0; i < surface_count; i++) {
        printf("    Surface ID: %u\n", surfaces[i].id);
        printf("      Position: (%d, %d)\n", surfaces[i].position.x, surfaces[i].position.y);
        printf("      Size: %ux%u\n", surfaces[i].size.width, surfaces[i].size.height);
        printf("      Visibility: %s\n", surfaces[i].visibility ? "true" : "false");
        printf("      Opacity: %.2f\n", surfaces[i].opacity);
        printf("      Orientation: %s\n", orientation_to_string(surfaces[i].orientation));
        printf("      Z-Order: %d\n", surfaces[i].z_order);
        printf("\n");
    }

    // Get properties of the first surface
    if (surface_count > 0) {
        uint32_t surface_id = surfaces[0].id;
        IviSurface surface;

        printf("Getting properties for surface %u...\n", surface_id);
        result = ivi_get_surface(client, surface_id, &surface, error_buf, sizeof(error_buf));
        
        if (result != IVI_OK) {
            fprintf(stderr, "✗ Failed to get surface: %s\n", error_buf);
            ivi_free_surfaces(surfaces);
            return 1;
        }

        printf("  ✓ Retrieved surface %u\n", surface.id);
        printf("    Current opacity: %.2f\n", surface.opacity);
        printf("    Current visibility: %s\n", surface.visibility ? "true" : "false");

        // Modify surface properties
        printf("\nModifying surface %u properties...\n", surface_id);

        // Set position
        printf("  Setting position to (100, 100)...\n");
        result = ivi_set_surface_position(client, surface_id, 100, 100, error_buf, sizeof(error_buf));
        if (result != IVI_OK) {
            fprintf(stderr, "✗ Failed to set position: %s\n", error_buf);
            ivi_free_surfaces(surfaces);
            return 1;
        }
        printf("    ✓ Position updated\n");

        // Set size
        printf("  Setting size to 800x600...\n");
        result = ivi_set_surface_size(client, surface_id, 800, 600, error_buf, sizeof(error_buf));
        if (result != IVI_OK) {
            fprintf(stderr, "✗ Failed to set size: %s\n", error_buf);
            ivi_free_surfaces(surfaces);
            return 1;
        }
        printf("    ✓ Size updated\n");

        // Set visibility
        printf("  Setting visibility to true...\n");
        result = ivi_set_surface_visibility(client, surface_id, true, error_buf, sizeof(error_buf));
        if (result != IVI_OK) {
            fprintf(stderr, "✗ Failed to set visibility: %s\n", error_buf);
            ivi_free_surfaces(surfaces);
            return 1;
        }
        printf("    ✓ Visibility updated\n");

        // Set opacity
        printf("  Setting opacity to 0.8...\n");
        result = ivi_set_surface_opacity(client, surface_id, 0.8f, error_buf, sizeof(error_buf));
        if (result != IVI_OK) {
            fprintf(stderr, "✗ Failed to set opacity: %s\n", error_buf);
            ivi_free_surfaces(surfaces);
            return 1;
        }
        printf("    ✓ Opacity updated\n");

        // Set orientation
        printf("  Setting orientation to Normal...\n");
        result = ivi_set_surface_orientation(client, surface_id, IVI_ORIENTATION_NORMAL, error_buf, sizeof(error_buf));
        if (result != IVI_OK) {
            fprintf(stderr, "✗ Failed to set orientation: %s\n", error_buf);
            ivi_free_surfaces(surfaces);
            return 1;
        }
        printf("    ✓ Orientation updated\n");

        // Set z-order
        printf("  Setting z-order to 10...\n");
        result = ivi_set_surface_z_order(client, surface_id, 10, error_buf, sizeof(error_buf));
        if (result != IVI_OK) {
            fprintf(stderr, "✗ Failed to set z-order: %s\n", error_buf);
            ivi_free_surfaces(surfaces);
            return 1;
        }
        printf("    ✓ Z-order updated\n");

        // Commit all changes atomically
        printf("\nCommitting changes...\n");
        result = ivi_commit(client, error_buf, sizeof(error_buf));
        if (result != IVI_OK) {
            fprintf(stderr, "✗ Failed to commit: %s\n", error_buf);
            ivi_free_surfaces(surfaces);
            return 1;
        }
        printf("  ✓ All changes committed successfully\n");

        // Verify changes
        printf("\nVerifying changes...\n");
        result = ivi_get_surface(client, surface_id, &surface, error_buf, sizeof(error_buf));
        if (result == IVI_OK) {
            printf("  Position: (%d, %d)\n", surface.position.x, surface.position.y);
            printf("  Size: %ux%u\n", surface.size.width, surface.size.height);
            printf("  Opacity: %.2f\n", surface.opacity);
            printf("  Visibility: %s\n", surface.visibility ? "true" : "false");
        }
    }

    // Free allocated memory
    ivi_free_surfaces(surfaces);

    printf("\n");
    return 0;
}

int demonstrate_layer_operations(IviClient* client) {
    char error_buf[256];
    IviLayer* layers = NULL;
    size_t layer_count = 0;
    IviErrorCode result;

    printf("--- Layer Operations ---\n\n");

    // List all layers
    printf("Listing all layers...\n");
    result = ivi_list_layers(client, &layers, &layer_count, error_buf, sizeof(error_buf));
    
    if (result != IVI_OK) {
        fprintf(stderr, "✗ Failed to list layers: %s\n", error_buf);
        return 1;
    }

    if (layer_count == 0) {
        printf("  No layers found\n");
        return 0;
    }

    printf("  Found %zu layer(s):\n", layer_count);
    for (size_t i = 0; i < layer_count; i++) {
        printf("    Layer ID: %u\n", layers[i].id);
        printf("      Visibility: %s\n", layers[i].visibility ? "true" : "false");
        printf("      Opacity: %.2f\n", layers[i].opacity);
        printf("\n");
    }

    // Get properties of the first layer
    if (layer_count > 0) {
        uint32_t layer_id = layers[0].id;
        IviLayer layer;

        printf("Getting properties for layer %u...\n", layer_id);
        result = ivi_get_layer(client, layer_id, &layer, error_buf, sizeof(error_buf));
        
        if (result != IVI_OK) {
            fprintf(stderr, "✗ Failed to get layer: %s\n", error_buf);
            ivi_free_layers(layers);
            return 1;
        }

        printf("  ✓ Retrieved layer %u\n", layer.id);
        printf("    Current opacity: %.2f\n", layer.opacity);
        printf("    Current visibility: %s\n", layer.visibility ? "true" : "false");

        // Modify layer properties
        printf("\nModifying layer %u properties...\n", layer_id);

        // Set visibility
        printf("  Setting visibility to true...\n");
        result = ivi_set_layer_visibility(client, layer_id, true, error_buf, sizeof(error_buf));
        if (result != IVI_OK) {
            fprintf(stderr, "✗ Failed to set visibility: %s\n", error_buf);
            ivi_free_layers(layers);
            return 1;
        }
        printf("    ✓ Visibility updated\n");

        // Set opacity
        printf("  Setting opacity to 0.9...\n");
        result = ivi_set_layer_opacity(client, layer_id, 0.9f, error_buf, sizeof(error_buf));
        if (result != IVI_OK) {
            fprintf(stderr, "✗ Failed to set opacity: %s\n", error_buf);
            ivi_free_layers(layers);
            return 1;
        }
        printf("    ✓ Opacity updated\n");

        // Commit changes
        printf("\nCommitting changes...\n");
        result = ivi_commit(client, error_buf, sizeof(error_buf));
        if (result != IVI_OK) {
            fprintf(stderr, "✗ Failed to commit: %s\n", error_buf);
            ivi_free_layers(layers);
            return 1;
        }
        printf("  ✓ All changes committed successfully\n");

        // Verify changes
        printf("\nVerifying changes...\n");
        result = ivi_get_layer(client, layer_id, &layer, error_buf, sizeof(error_buf));
        if (result == IVI_OK) {
            printf("  Opacity: %.2f\n", layer.opacity);
            printf("  Visibility: %s\n", layer.visibility ? "true" : "false");
        }
    }

    // Free allocated memory
    ivi_free_layers(layers);

    printf("\n");
    return 0;
}

void demonstrate_error_handling(IviClient* client) {
    char error_buf[256];
    IviSurface surface;
    IviLayer layer;
    IviErrorCode result;

    printf("--- Error Handling ---\n\n");

    // Try to get a non-existent surface
    printf("Attempting to get non-existent surface (ID: 99999)...\n");
    result = ivi_get_surface(client, 99999, &surface, error_buf, sizeof(error_buf));
    
    if (result != IVI_OK) {
        printf("  ✓ Correctly handled error:\n");
        printf("    Error code: %d\n", result);
        printf("    Error message: %s\n", error_buf);
    } else {
        printf("  Unexpected success\n");
    }

    printf("\n");

    // Try to get a non-existent layer
    printf("Attempting to get non-existent layer (ID: 99999)...\n");
    result = ivi_get_layer(client, 99999, &layer, error_buf, sizeof(error_buf));
    
    if (result != IVI_OK) {
        printf("  ✓ Correctly handled error:\n");
        printf("    Error code: %d\n", result);
        printf("    Error message: %s\n", error_buf);
    } else {
        printf("  Unexpected success\n");
    }

    printf("\n");
}

const char* orientation_to_string(IviOrientation orientation) {
    switch (orientation) {
        case IVI_ORIENTATION_NORMAL:
            return "Normal";
        case IVI_ORIENTATION_ROTATE90:
            return "Rotate90";
        case IVI_ORIENTATION_ROTATE180:
            return "Rotate180";
        case IVI_ORIENTATION_ROTATE270:
            return "Rotate270";
        default:
            return "Unknown";
    }
}
