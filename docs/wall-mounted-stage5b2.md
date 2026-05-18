# Stage 5B.2 - BuildTool Wall Surface Preview Integration

Stage 5B.2 integrates wall surfaces into `BuildTool` as a preview-only placement strategy.
It does not create real wall-mounted objects yet.

Status: superseded by [wall-mounted-stage5b3.md](/home/danilasar/data/projects/mallgame/docs/wall-mounted-stage5b3.md). This document remains as historical notes for the preview-only slice.

## Scope

- `PlacementKind` supports `Floor` and `WallMounted`
- `BuildTool` selects its placement strategy from the selected prototype
- floor placement behavior remains unchanged
- wall-mounted placement uses `PointerTargets.wall_surface`
- wall-mounted placement computes a runtime `WallAttachmentPoint`
- wall-mounted placement spawns a transient preview entity
- wall-mounted commit is disabled in Stage 5B.2
- the Ribbon exposes a visible `Walls` tab for the wall-mounted dev prototype
- the startup scene spawns one real `StoreObject` instance of the wall prototype for Move/Delete smoke testing

## What Stage 5B.2 Does Not Do

- no real wall-mounted object spawn
- no `BuildObjectRequested` for wall-mounted preview clicks
- no `DomainCommand::BuildObject` for wall-mounted placement
- no save/load for wall-mounted placement
- no doors
- no windows
- no wall occupancy
- no door clearance
- no navigation portals
- no window transparency or cutouts

## Runtime Types

### Placement kind

```rust
pub enum PlacementKind {
    Floor,
    WallMounted,
}
```

### Wall attachment

```rust
pub struct WallAttachmentPoint {
    pub segment_key: WallSegmentKey,
    pub offset_along_segment: f32,
    pub height_on_wall: f32,
}
```

### Wall hit result

```rust
pub struct WallSurfaceHit {
    pub entity: Entity,
    pub key: WallSegmentKey,
    pub world_pos: Vec2,
    pub offset_along_segment: f32,
    pub height_on_wall: f32,
    pub normal: Vec2,
}
```

## BuildTool Behavior

- floor prototypes continue to spawn `BuildObjectRequested` on valid click
- wall-mounted prototypes create a wall-attached preview only
- wall preview uses `ToolPreview`, `RuntimeOwned`, and `WallMountedPreview`
- wall preview is removed on cancel, mode switch, or load/reset
- wall preview is not a `StoreObject`

## Picking Rules

- `PointerTargets.world_object` stays for store objects
- `PointerTargets.wall_surface` is used for wall-mounted preview targeting
- `PointerTargets.exterior` remains separate
- move/delete tools ignore wall surfaces

## Input Safety

- wall preview respects `ToolInputGate`
- ribbon/UI selection still uses the fresh-click guard
- same-release selection must not create or commit anything

## Test Targets

- wall hit detection projects and clamps to wall bounds
- wall attachment clamps to segment bounds
- floor preview regression remains intact
- wall preview does not emit build requests or domain commands
- wall preview is transient and not saved
