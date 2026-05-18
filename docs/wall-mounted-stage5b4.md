# Stage 5B.4: Wall-Mounted Move

## Overview
Stage 5B.4 introduces the ability to move wall-mounted objects along wall surfaces without utilizing the `Movable` floor-movement logic or `WorldPos` placement authority.

## Core Mechanics

### Explicit Capability
Wall-mounted objects do not receive the `Movable` component, which is strictly reserved for floor placement. Instead, they must opt-in via a `movable: true` flag in their `WallMountedSpec`, which grants the `WallMovable` marker component.

### Tool Session Separation
`MoveToolSession` is split into an enum: `Floor` and `WallMounted`.
When a `WallMovable` object is clicked, a `WallMoveSession` starts. It uses `PointerTargets.wall_surface` instead of floor raycasting.

### Attachment and Occupancy Validation
- The session computes a new `WallAttachmentPoint` dynamically as the cursor moves over a valid wall surface.
- Validation checks for conflicts via `Wallprint` overlaps.
- The validation engine employs self-ignore: a moving object does not collide with its own prior `Wallprint`.

### Domain Mutation
`DomainCommand::MoveObject` no longer relies on purely `Vec2` positions. Instead, it accepts an explicit `ObjectPlacement` payload.
For wall-mounted objects, it updates `WallMountedPlacement`, regenerates the runtime `Wallprint`, updates internal bounding caches, and emits an `ObjectMoved` domain event on success.

### Persistence
`ObjectPlacementSave::WallMounted` already captures `WallAttachmentPoint`, so no new save schema was necessary. Load logic restores `WallMovable` correctly.

## Limitations
- Floor → WallMounted conversion is strictly rejected.
- WallMounted → Floor conversion is strictly rejected.
- No doors, cutouts, clearance, or navigation portals are implemented in this stage.