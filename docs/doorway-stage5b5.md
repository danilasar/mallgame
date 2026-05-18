# Stage 5B.5 / 5B.5.1: Doorway Access Object Hardening

This stage stabilizes the basic doorway object. It does not add navigation,
wall cutouts, open/close state, animation, NPC traversal, exterior access
zones, or door economy.

## Model

`wall.door.basic_customer` is a real `StoreObject`, but it is not a floor
object. Its persistent placement authority is:

```rust
ObjectPlacement::WallMounted {
    attachment: WallAttachmentPoint {
        segment_key,
        offset_along_segment,
        height_on_wall,
    }
}
```

The `height_on_wall` value is normalized to the doorway base height, currently
`0.0`, before build, move, load restoration, and factory spawning. The stable
wall identity is `WallSegmentKey`, never `Entity`.

## Components

A built, moved, or loaded door should have:

- `StoreObject`;
- `ObjectStableId`;
- `ObjectPrototypeId`;
- `ObjectPlacementComponent`;
- `WallMountedPlacement`;
- `WallMounted`;
- `Wallprint`;
- `Doorway`;
- `InteriorAccessZone`;
- `DoorMovable`;
- `Selectable`;
- `Inspectable`;
- `Deletable`;
- `Interactive` / `InteractionRole::WorldObject`.

It should not have:

- floor `Footprint`;
- `BlocksPlacement`;
- floor `Movable`;
- generic `WallMovable`;
- navigation portal or wall-cutout components.

`InteriorAccessZone` is floor access reservation geometry. It is not a physical
floor footprint and is derived from the doorway prototype plus wall attachment.

## Move Routing

Move is one user-facing action, but internal routing is strategy-based:

```text
FloorPlacement + Movable
  -> FloorMoveSession

Doorway + DoorMovable
  -> DoorMoveSession

WallMountedPlacement + WallMovable
  -> WallMoveSession
```

The door branch must be checked before the generic wall-mounted branch. A door
must never start `WallMoveSession`, because it has to validate and update both
wall occupancy and interior access reservation.

## Door Move Validation

Door move validates a derived placement:

```text
WallMountedPlacement
+ DoorwaySpec
+ WallMountedSpec
+ WallSurface
-> DerivedDoorPlacement {
     Wallprint,
     InteriorAccessZone,
   }
```

The validator self-ignores only the moving door's old spatial claims:

- its previous `Wallprint`;
- its previous `InteriorAccessZone`.

It does not ignore other wall-mounted objects, other access zones, floor
blockers, store coverage, or missing wall segments.

## Preview Authority

Door build/move previews are transient:

- `ToolPreview`;
- `RuntimeOwned`;
- `NonInteractive`;
- not `StoreObject`;
- no `ObjectStableId`;
- not saved.

The access-zone preview uses `AccessZonePreviewShape`, not real
`InteriorAccessZone`. Real `InteriorAccessZone` exists only on the committed
door object and is domain authority for floor placement blocking.

## Lifecycle

After build:

- the door's `Wallprint` blocks overlapping wall placement;
- the door's `InteriorAccessZone` blocks floor placement.

After move:

- the old wall interval and old access zone are freed;
- the new wall interval and new access zone block placement.

After delete:

- wall occupancy and access reservation disappear with the door entity.

After load:

- save data restores `ObjectPlacement::WallMounted`;
- the factory derives `Wallprint` and `InteriorAccessZone` again;
- derived preview/runtime geometry is not loaded from save.

## Tests

Current hardening coverage includes:

- built doors have required components and lack forbidden floor/generic-wall
  movement components;
- moved doors preserve the same invariants;
- door move self-overlap is allowed for the old door wall/access claims;
- door access zones block floor placement after build;
- door move frees the old access zone and blocks the new one;
- door delete frees wall occupancy and access reservation;
- wall decor/window objects do not create `InteriorAccessZone`;
- `Doorway + DoorMovable` starts `DoorMoveSession`, not generic wall move;
- generic wall objects still start `WallMoveSession`;
- floor objects still start `FloorMoveSession`;
- save/load restores a doorway with derived `Wallprint` and
  `InteriorAccessZone`.

## Non-Goals

- no `NavigationPortal`;
- no `WallCutout`;
- no NPC pathfinding or traversal;
- no open/close or lock/unlock state;
- no door animation;
- no exterior access reservation;
- no generic spatial-claim framework.
