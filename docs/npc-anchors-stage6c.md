# NPC Anchor Model (Stage 6C)

## Overview
Stage 6C introduces a shared semantic target system called `NpcAnchor`. Anchors are stable target points that NPCs can navigate to.

## Key Concepts
- **NpcAnchor**: Represents a semantic point (e.g., "browse here", "stand at station"). It has a position, facing, and role filtering.
- **Stable IDs**: Anchor IDs are semantic and stable (e.g., `StoreObject(stable_id) + "browse"`), not based on runtime `Entity` IDs.
- **Anchor Cache**: A runtime-derived cache (`NpcAnchorCache`) that stores all active anchors.
- **Lifecycle**: Anchors are automatically re-derived when their owner (object or doorway) is built, moved, or deleted.

## Usage
Anchors are managed by the `NpcPlugin`. Systems can resolve anchors by ID or owner.

### Resolving an Anchor
```rust
if let Ok(anchor) = resolve_anchor_for_npc(role, target, &anchor_cache) {
    let target_pos = anchor.point.pos;
}
```

## Debugging
Enable `show_anchors` in `NavigationDebugSettings` to see anchor points in the world.
