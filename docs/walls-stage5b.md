# Stage 5B.1 - Store Boundary Walls & Exterior Layer Foundation

Stage 5B.1 is the foundation for store boundary walls and a proper exterior layer inside `WorldBounds`.
It does not implement doors, windows, wall-mounted placement, or wall occupancy.

## Core Idea

The world now has three distinct gameplay layers:

- `StoreArea`: buyable 4x4 store chunks and freeform store placement authority
- `WorldBounds`: the larger world container that also contains exterior entities
- `Boundary Walls`: derived visuals/metadata for locked top/right store edges

Exterior is no longer just background decor. The code currently adds the exterior role/component foundation and picking slot separation; authored exterior content can be added later without touching store authority.
Exterior entities may eventually be pickable, inspectable, stateful, animated, or scripted.
They are still not `StoreObject` and must not share store placement/save authority.

## What Stage 5B.1 Does

- Derives locked top/right store boundary segments from the outer top-right owned chunk of `StoreArea + expansion policy`
- Spawns wall visuals with visible thickness
- Attaches addressable wall metadata (`WallSurface`)
- Adds a separate exterior entity layer inside `WorldBounds`
- Keeps picking separated between `WorldObject`, `WallSurface`, and `Exterior`
- Keeps walls derived, not saved
- Keeps exterior separate from `StoreObject` save authority

## What Stage 5B.1 Does Not Do

- No doors
- No windows
- No wall-mounted placement
- No wall occupancy for store objects
- No exterior editor tools
- No NPC pathfinding
- No wall persistence in `SaveGame`

## Component Model

### Boundary identifiers

```rust
pub enum StoreBoundarySide {
    Top,
    Right,
}
```

```rust
pub struct WallSegmentKey {
    pub chunk: StoreChunkCoord,
    pub side: StoreBoundarySide,
}
```

```rust
pub struct StoreBoundarySegment {
    pub key: WallSegmentKey,
    pub start: WorldPos,
    pub end: WorldPos,
    pub normal: Vec2,
    pub length: f32,
    pub height: f32,
}
```

### Wall entities

```rust
#[derive(Component)]
pub struct StoreWallSegment {
    pub key: WallSegmentKey,
}

#[derive(Component)]
pub struct WallSurface {
    pub key: WallSegmentKey,
    pub start: Vec2,
    pub end: Vec2,
    pub length: f32,
    pub height: f32,
    pub thickness: f32,
    pub normal: Vec2,
}

#[derive(Component)]
pub struct WallVisual;
```

### Exterior entities

```rust
#[derive(Component)]
pub struct ExteriorObject;

#[derive(Component)]
pub struct ExteriorStateful;

#[derive(Component)]
pub struct ExteriorInspectable;

#[derive(Component)]
pub struct ExteriorInteractive;

#[derive(Component)]
pub struct ExteriorVisual;
```

Exterior entities are separate from store entities. They may have state and picking, but they are not `StoreObject`, not `BlocksPlacement`, and not part of store expansion/build authority.

## Picking And Interaction

Current input runtime already separates world objects, widgets, previews, overlays, and debug targets.
Stage 5B.1 adds explicit separation for walls and exterior:

- `WorldObject` for store objects
- `WallSurface` for wall-facing tools and future wall-mounted flows
- `Exterior` for pickable exterior entities, once authored content is added

Rule of thumb:

- `Cursor`, `Move`, `Delete` tools look at `WorldObject`
- future wall tools look at `WallSurface`
- exterior interaction uses a separate exterior target path

## Boundary Generation

Boundary walls are derived from owned chunks, not from a rectangular store outline.
In the current MVP they start at the outer top-right owned chunk and extend along the top row and right column only while the run remains contiguous.

MVP generation rule:

- if the outer top-right owned chunk exists and the `Top` side is locked, emit top boundary segments across the contiguous top-row run
- if the outer top-right owned chunk exists and the `Right` side is locked, emit right boundary segments across the contiguous right-column run
- wall height is at least `1.5x` the chunk height
- stop the run at the first missing chunk

Do not emit walls:

- between owned chunks
- on unlocked sides
- on inner shared chunk edges
- on a second or third row past the first gap in the run
- as a substitute for store ownership logic

## Boundary Policy

Walls reflect expansion policy. They do not enforce it by existence.

Correct model:

```text
expansion policy -> locked boundary sides -> rendered walls
```

Incorrect model:

```text
wall entity exists -> expansion is forbidden
```

## Wall Sync

Wall visuals must be cache/delta synced:

1. read final `StoreArea`
2. compute expected `WallSegmentKey` set
3. diff against cache
4. spawn missing wall entities
5. despawn stale wall entities
6. update changed wall visuals when needed

Walls should not be rebuilt every frame.

## Exterior Layer

Exterior lives inside `WorldBounds` and can be part of the world layer.
The current codebase has the component and pointer-target foundation; authored exterior content is a follow-up step.

Persistence model for exterior is intentionally separate from store save authority:

- static derived exterior: rebuilt from world config/seed
- runtime-only exterior: not saved
- persistent exterior: separate save section, not `StoreObject` save

If exterior state is not needed yet, do not invent a mixed save path for it.

## Sorting And Visuals

Walls and exterior need explicit presentation layering so they do not collide with store objects or overlays.

Suggested layers:

- exterior back
- wall face
- wall top cap
- wall-mounted future layer
- store objects
- overlays
- UI

Wall thickness is a presentation concern, not a placement blocker.

## Scheduling

Wall sync should run after store mutations and after deferred commands are applied, so presentation sees final wall entities.

## Save/Load Rules

- Do not save derived wall entities.
- Do not save wall caches.
- Do not merge exterior state into `StoreObject` save data.
- Rebuild walls from `StoreArea` on load.
- If exterior gets persistence later, use a separate save section.

## Test Targets

- boundary generation on initial store shape
- contiguous run generation from the outer corner
- outer-corner anchoring
- shared edges not generating walls
- locked top/right sides generating walls
- wall cache diffing
- walls not being `StoreObject`
- exterior not being picked as store objects
- save/load excluding derived walls

## Definition Of Done

Stage 5B.1 is done when:

- wall boundary segments are derived from `StoreArea + expansion policy`
- wall visuals render with thickness
- wall surfaces carry addressable metadata
- walls are cache/delta synced
- exterior exists as a separate world layer
- picking separation is explicit
- derived walls are not saved
- exterior is not mixed into store save authority
- tests and docs are updated
