# Stage 5B.3 - Wall-Mounted Object MVP

Stage 5B.3 turns the wall-mounted preview path into the first real wall-mounted object flow.

The scope is intentionally narrow: wall decor, a visual-only window, and a
basic wall door are buildable, saved, loaded, selectable, inspectable, and
deletable. Navigation portals, wall cutouts, and general wall-mounted move are
not part of this stage.

## Runtime Model

Wall-mounted objects are real `StoreObject` entities. Wall visuals and wall surfaces are not.
Being a real `StoreObject` does not mean the object is movable by the floor
`MoveTool`.

```rust
pub enum ObjectPlacement {
    Floor {
        world_pos: Vec2,
        rotation_index: Option<usize>,
    },
    WallMounted {
        attachment: WallAttachmentPoint,
    },
}
```

`WallAttachmentPoint` stores a stable wall address:

```rust
pub struct WallAttachmentPoint {
    pub segment_key: WallSegmentKey,
    pub offset_along_segment: f32,
    pub height_on_wall: f32,
}
```

It never stores `Entity`. `Entity` is runtime-only and is only valid inside the current Bevy `World`.

## Build Flow

```text
Ribbon
-> selected wall-mounted prototype
-> BuildTool wall preview
-> clean world click
-> BuildObjectRequested { placement: ObjectPlacement::WallMounted }
-> DomainCommand::BuildObject
-> wall placement validation
-> spawn_store_object_from_prototype(...)
```

Floor build remains unchanged, except that the request and command payload now carry `ObjectPlacement`.

## Validation

Wall-mounted build validation checks:

- prototype exists;
- prototype uses `PlacementKind::WallMounted`;
- prototype has `WallMountedSpec`;
- `WallSegmentKey` resolves to a current `WallSurface`;
- side is allowed by the prototype spec;
- offset is inside the surface bounds;
- height is inside the surface bounds;
- occupied wall rectangle does not overlap an existing `WallMounted` object on the same segment.

Rejected wall-mounted builds do not spawn objects, emit domain events, or partially mutate the world.

## Factory Behavior

`spawn_store_object_from_prototype(...)` accepts `ObjectPlacement`.

For floor placement it spawns the normal floor object path:

- `WorldPos`;
- `Footprint`;
- `BlocksPlacement` if the prototype blocks placement;
- `Movable`;
- floor-oriented render/sort data.

For wall-mounted placement it spawns:

- `StoreObject`;
- `ObjectStableId`;
- `ObjectPrototypeId`;
- `ObjectPlacementComponent`;
- `WallMountedPlacement`;
- `WallMounted`;
- `Wallprint`;
- `Selectable`;
- `Inspectable`;
- `Interactive`;
- `InteractionRole::WorldObject`;
- render components for presentation.

Wall-mounted objects do not get `Movable` in Stage 5B.3 and do not become floor placement blockers.
They also do not receive a floor `Footprint`. `StoreObject` no longer implies "floor-placed object".

`WallWindow` is an additional semantic component for visual-only windows:

```rust
pub struct WallWindow {
    pub glass_alpha: f32,
}
```

It is presentation semantics only. It does not cut a hole in the wall and does not alter collision, placement, or navigation.

## Geometry Hardening

Stage 5B.3.1 removes the MVP compatibility leak where wall-mounted objects still had a floor footprint.

Current rule:

```text
Floor object:
  ObjectPlacement::Floor
  WorldPos
  Footprint
  optional BlocksPlacement
  Movable

Wall-mounted object:
  ObjectPlacement::WallMounted
  WallMounted
  WallMountedBounds
  no floor Footprint
  no BlocksPlacement
  no Movable
```

`Wallprint` is the wall occupancy geometry:

```rust
pub struct Wallprint {
    pub rects: Vec<WallprintRect>,
}

pub struct WallprintRect {
    pub segment_key: WallSegmentKey,
    pub offset_min: f32,
    pub offset_max: f32,
    pub height_min: f32,
    pub height_max: f32,
    pub occupancy_kind: WallOccupancyKind,
}
```

`Wallprint` answers "what wall area does this object occupy?". It is derived from
`WallMountedPlacement + WallMountedSpec`, uses `WallSegmentKey`, and is not save
authority. Save/load stores `ObjectPlacement::WallMounted`; the factory derives
the wallprint again.

`WallMountedBounds` is a narrow runtime compatibility cache for current
selection/inspection paths:

```rust
pub struct WallMountedBounds {
    pub segment_key: WallSegmentKey,
    pub offset_min: f32,
    pub offset_max: f32,
    pub height_min: f32,
    pub height_max: f32,
}
```

Floor placement validation and footprint overlays ignore wall-mounted objects.
Wall validation uses `Wallprint` conflict policy, not floor `Footprint`.

Current spatial split:

```text
Placement:
  FloorPlacement / WallMountedPlacement

Occupancy:
  Footprint/FloorFootprint / Wallprint

Access:
  InteriorAccessZone

Selection:
  current sprite or wall runtime bounds strategy

Visual:
  VisualSpec + presentation sync
```

## Presentation

Wall-mounted object position is derived from:

```text
WallMounted.attachment
+ WallSurface geometry
+ wall visual thickness/height
-> WorldPos + VisualOffset
```

`WorldPos` exists on the entity for Bevy transform and picking compatibility, but the authority is the `WallMounted` attachment.

## Move Behavior

Wall-mounted move is intentionally not implemented in Stage 5B.3/5B.3.1.

Current behavior:

```text
Floor object:
  has Movable
  uses current MoveTool
  moves by previewing/committing a new floor WorldPos

Wall-mounted object:
  no Movable
  no floor Footprint
  move action is ignored/rejected
  remains selectable, inspectable, deletable, and saveable
```

This is expected. A wall-mounted move needs a separate placement strategy:

```text
hover WallSurface
-> compute new WallAttachmentPoint
-> validate Wallprint overlap
-> commit new WallMountedPlacement
```

It must not reuse the floor MoveTool path, because that path assumes floor
`WorldPos + Footprint` placement.

## Save / Load

`ObjectSave` now stores `placement`:

```rust
pub enum ObjectPlacementSave {
    Floor {
        world_pos: WorldPosSave,
        rotation_index: Option<usize>,
    },
    WallMounted {
        segment_key: WallSegmentKeySave,
        offset_along_segment: f32,
        height_on_wall: f32,
    },
}
```

Wall-mounted objects are saved as real store objects. Wall visuals and wall surfaces remain derived and are not saved.

## Current Sample

`wall.decor.placeholder` is the simple MVP wall-mounted decor prototype:

- `PlacementKind::WallMounted`;
- visible in the Ribbon `Walls` tab;
- `Decor`;
- `WallMountedSpec`;
- allowed on top and right wall segments.

It is a simple wall decor object. It does not imply windows, transparency, wall cutouts, doors, or navigation.

`wall.window.basic_visual` is the visual-only window prototype:

- `PlacementKind::WallMounted`;
- visible in the Ribbon `Walls` tab;
- `Decor`;
- `WallMountedSpec`;
- `Window { glass_alpha }`;
- allowed on top and right wall segments.

The window is rendered above the wall face as a translucent wall-mounted visual. Exterior visibility through the window is a presentation/layering concern: exterior objects are behind `WallFace`, and the window is drawn above the wall. No StoreArea hole, wall cutout, or navigation portal is created.

`wall.door.basic_customer` is the first basic door prototype:

- `PlacementKind::WallMounted`;
- visible in the Ribbon `Walls` tab;
- `WallMountedSpec`;
- `Doorway`;
- `DoorMovable`;
- creates an `InteriorAccessZone`.

Unlike generic wall-mounted decor/windows, a door is always clamped to the
floor line of the wall. Any incoming `WallAttachmentPoint.height_on_wall` is
normalized to the doorway spec base height, currently `0.0`, before build,
move, save/load restoration, or factory spawning. The door still does not
create a wall cutout or navigation portal yet.

## Explicit Non-Goals

- no navigation portals;
- no NPC pathfinding;
- no wall cutouts;
- no wall editor;
- no general wall-mounted move for decor/window objects;
- no exterior editing tools.

## Tests

Current coverage includes:

- floor build regression;
- wall-mounted build spawns a real `StoreObject`;
- wall-mounted build adds `WallMountedPlacement` and `Wallprint`;
- wall-mounted build does not add `BlocksPlacement` or `Movable`;
- wall-mounted build does not add floor `Footprint`;
- wall-mounted overlap is rejected through `Wallprint` conflict checks;
- door build clamps wall height to the floor line;
- wall-mounted load restores wall placement/occupancy components and does not add floor `Footprint`;
- wall surface hit and attachment clamping;
- save/load authority restoration for existing objects.
