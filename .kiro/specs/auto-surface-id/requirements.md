# Requirements Document

## Introduction

This document specifies the requirements for automatic surface ID assignment functionality in the Weston IVI Controller. When Wayland applications create IVI surfaces without specifying a surface ID, the compositor assigns the invalid ID `0xFFFFFFFF`. This feature will detect such cases and automatically assign valid, unique surface IDs to ensure proper surface management.

## Glossary

- **IVI Surface**: A Wayland surface managed by the IVI shell protocol with a unique numeric identifier
- **Surface ID**: A 32-bit unsigned integer that uniquely identifies an IVI surface within the compositor
- **IVI_ID_INVALID**: The invalid surface ID value `0xFFFFFFFF` (4294967295) assigned by the compositor when no ID is specified
- **Auto-Assignment Range**: The range of surface IDs from `0x10000000` (268435456) to `0xFFFFFFFE` (4294967294) used for automatic assignment
- **ID Pool**: The collection of available surface IDs that can be assigned to new surfaces
- **ID Wraparound**: The process of returning to the start of the auto-assignment range when the maximum ID is reached
- **Controller Module**: The Weston IVI Controller plugin that manages surface operations
- **Surface Registry**: The internal data structure tracking active surface IDs and their assignment status

## Requirements

### Requirement 1

**User Story:** As a system integrator, I want the controller to detect surfaces with invalid IDs, so that applications that don't specify surface IDs can still be managed properly.

#### Acceptance Criteria

1. WHEN an IVI Surface is created with ID `0xFFFFFFFF`, THE Controller Module SHALL detect this as an invalid surface ID
2. WHEN an invalid surface ID is detected, THE Controller Module SHALL trigger the automatic ID assignment process
3. THE Controller Module SHALL monitor all surface creation events for invalid ID detection
4. WHEN a surface with a valid ID is created, THE Controller Module SHALL not trigger automatic assignment
5. THE Controller Module SHALL log detection of invalid surface IDs for debugging purposes

### Requirement 2

**User Story:** As a system integrator, I want automatic surface ID assignment to use a dedicated range, so that auto-assigned IDs don't conflict with manually specified IDs.

#### Acceptance Criteria

1. THE Controller Module SHALL assign surface IDs starting from `0x10000000` (268435456)
2. THE Controller Module SHALL increment the surface ID by 1 for each new surface requiring automatic assignment
3. THE Controller Module SHALL only use IDs within the range `0x10000000` to `0xFFFFFFFE` for automatic assignment
4. THE Controller Module SHALL not assign the invalid ID `0xFFFFFFFF` to any surface
5. WHEN the maximum ID `0xFFFFFFFE` is reached, THE Controller Module SHALL wrap around to `0x10000000`

### Requirement 3

**User Story:** As a system integrator, I want ID wraparound to skip already-used IDs, so that each surface maintains a unique identifier even after wraparound occurs.

#### Acceptance Criteria

1. WHEN ID assignment wraps around to `0x10000000`, THE Controller Module SHALL check if that ID is already in use
2. WHEN a candidate ID is already in use, THE Controller Module SHALL increment to the next ID and check again
3. THE Controller Module SHALL continue searching for an available ID until one is found
4. THE Controller Module SHALL maintain a registry of all active surface IDs to enable conflict detection
5. WHEN no available IDs exist in the auto-assignment range, THE Controller Module SHALL return an error

### Requirement 4

**User Story:** As a system integrator, I want surface ID reuse when surfaces are destroyed, so that the ID space doesn't become exhausted over time.

#### Acceptance Criteria

1. WHEN an IVI Surface with an auto-assigned ID is destroyed, THE Controller Module SHALL mark that ID as available for reuse
2. THE Controller Module SHALL remove destroyed surface IDs from the active registry
3. WHEN wraparound occurs, THE Controller Module SHALL be able to reuse previously assigned but now available IDs
4. THE Controller Module SHALL prioritize sequential assignment over immediate reuse for predictable behavior
5. THE Controller Module SHALL handle surface destruction events to maintain accurate ID availability

### Requirement 5

**User Story:** As a system integrator, I want the auto-assigned surface ID to replace the invalid ID, so that the surface can be managed through standard IVI operations.

#### Acceptance Criteria

1. WHEN a valid ID is assigned to a surface, THE Controller Module SHALL update the surface's ID in the IVI compositor
2. THE Controller Module SHALL ensure the surface is accessible using the new auto-assigned ID
3. WHEN the ID replacement is complete, THE Controller Module SHALL make the surface available for standard IVI operations
4. THE Controller Module SHALL verify that the ID replacement was successful before proceeding
5. WHEN ID replacement fails, THE Controller Module SHALL log the error and attempt recovery

### Requirement 6

**User Story:** As an application developer, I want auto-assigned surface IDs to be persistent during the surface lifetime, so that I can reliably reference surfaces once they are created.

#### Acceptance Criteria

1. WHEN a surface receives an auto-assigned ID, THE Controller Module SHALL maintain that ID until the surface is destroyed
2. THE Controller Module SHALL not reassign or change auto-assigned IDs during surface lifetime
3. WHEN queried for surface information, THE Controller Module SHALL return the auto-assigned ID consistently
4. THE Controller Module SHALL ensure auto-assigned IDs are included in surface state notifications
5. THE Controller Module SHALL treat auto-assigned IDs identically to manually specified IDs for all operations

### Requirement 7

**User Story:** As a system administrator, I want comprehensive logging of ID assignment operations, so that I can monitor and debug surface ID management.

#### Acceptance Criteria

1. WHEN an invalid surface ID is detected, THE Controller Module SHALL log the detection event with surface details
2. WHEN a new ID is auto-assigned, THE Controller Module SHALL log the assignment with both old and new ID values
3. WHEN ID wraparound occurs, THE Controller Module SHALL log the wraparound event
4. WHEN a surface ID is released due to destruction, THE Controller Module SHALL log the release event
5. WHEN ID assignment fails, THE Controller Module SHALL log detailed error information including the reason for failure

### Requirement 8

**User Story:** As a system integrator, I want thread-safe ID assignment, so that concurrent surface creation doesn't result in ID conflicts.

#### Acceptance Criteria

1. THE Controller Module SHALL ensure ID assignment operations are atomic and thread-safe
2. WHEN multiple surfaces are created concurrently, THE Controller Module SHALL assign unique IDs to each surface
3. THE Controller Module SHALL use appropriate synchronization mechanisms to prevent race conditions in ID assignment
4. THE Controller Module SHALL maintain consistency of the Surface Registry under concurrent access
5. WHEN concurrent ID assignment occurs, THE Controller Module SHALL ensure no duplicate IDs are assigned